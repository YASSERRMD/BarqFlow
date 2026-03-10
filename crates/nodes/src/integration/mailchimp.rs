use crate::integration::common::{
    build_standard_output, build_url, ensure_required_string, execute_prepared_request,
    get_optional_param, get_optional_string_param, get_string_param, get_u64_param, parse_body,
    parse_kv_pairs, resolve_api_key, run_count, PreparedRequest,
};
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use reqwest::Client;
use serde_json::json;

pub struct MailchimpNode {
    client: Client,
}

impl MailchimpNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for MailchimpNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for MailchimpNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Mailchimp",
            "description": "Manage Mailchimp audience members"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "addMember").await;
            let base_url = get_string_param(
                context,
                "baseUrl",
                item_index,
                "https://us1.api.mailchimp.com/3.0",
            )
            .await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let api_key = resolve_api_key(
                context,
                "Mailchimp",
                item_index,
                "mailchimpApi",
                &["apiKey"],
            )
            .await?;

            let basic_token = STANDARD.encode(format!("barqflow:{}", api_key));
            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push((
                "Authorization".to_string(),
                format!("Basic {}", basic_token),
            ));
            headers.push(("Content-Type".to_string(), "application/json".to_string()));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "addMember" => {
                    let list_id = ensure_required_string(
                        "Mailchimp",
                        "List ID",
                        get_optional_string_param(context, "listId", item_index).await,
                        "Set Mailchimp audience list ID.",
                    )?;
                    let email = ensure_required_string(
                        "Mailchimp",
                        "Email Address",
                        get_optional_string_param(context, "emailAddress", item_index).await,
                        "Set member email address.",
                    )?;
                    let status = get_optional_string_param(context, "memberStatus", item_index)
                        .await
                        .unwrap_or_else(|| "subscribed".to_string());
                    let merge_fields =
                        parse_body(get_optional_param(context, "mergeFields", item_index).await)
                            .unwrap_or_else(|| json!({}));

                    (
                        "POST".to_string(),
                        build_url(&base_url, &format!("/lists/{list_id}/members")),
                        Some(json!({
                            "email_address": email,
                            "status": status,
                            "merge_fields": merge_fields,
                        })),
                    )
                }
                "listMembers" => {
                    let list_id = ensure_required_string(
                        "Mailchimp",
                        "List ID",
                        get_optional_string_param(context, "listId", item_index).await,
                        "Set Mailchimp audience list ID.",
                    )?;
                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/lists/{list_id}/members")),
                        None,
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Mailchimp",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide resource path like /lists.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Mailchimp".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Mailchimp",
                PreparedRequest {
                    method,
                    url,
                    headers,
                    query,
                    body,
                    auth_token: None,
                    timeout_ms,
                },
            )
            .await?;

            output_items.push(build_standard_output(&operation, response));
        }

        Ok(vec![output_items])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::integration::common::test_utils::MockContext;
    use mockito::Matcher;
    use mockito::Server;

    #[tokio::test]
    async fn mailchimp_add_member_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/lists/list1/members")
            .match_header("authorization", Matcher::Regex("Basic .+".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"member1"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Mailchimp", "barqflow-nodes.mailchimp");
        context.add_param("operation", json!("addMember"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("apiKey", json!("mc-key"));
        context.add_param("listId", json!("list1"));
        context.add_param("emailAddress", json!("u@example.com"));

        let result = MailchimpNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn mailchimp_uses_bound_credential_when_api_key_is_empty() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/lists/list1/members")
            .match_header("authorization", Matcher::Regex("Basic .+".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"member1"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Mailchimp", "barqflow-nodes.mailchimp");
        context.add_param("operation", json!("addMember"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("listId", json!("list1"));
        context.add_param("emailAddress", json!("u@example.com"));
        context.add_credential("mailchimpApi", "apiKey", json!("mc-key"));

        let result = MailchimpNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn mailchimp_requires_api_key() {
        let mut context = MockContext::new("Mailchimp", "barqflow-nodes.mailchimp");
        context.add_param("operation", json!("listMembers"));
        context.add_param("listId", json!("list1"));

        let err = MailchimpNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("API Key"));
    }
}
