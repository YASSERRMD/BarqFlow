use crate::integration::common::{
    build_standard_output, build_url, ensure_required_string, execute_prepared_request,
    get_optional_param, get_optional_string_param, get_string_param, get_u64_param, parse_body,
    parse_kv_pairs, require_auth_token, run_count, PreparedRequest,
};
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use reqwest::Client;
use serde_json::json;

pub struct OutlookNode {
    client: Client,
}

impl OutlookNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for OutlookNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for OutlookNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Outlook",
            "description": "Send email and read mailbox via Microsoft Graph"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "sendMail").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://graph.microsoft.com/v1.0").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = require_auth_token(
                "Outlook",
                get_optional_string_param(context, "authToken", item_index).await,
            )?;

            let headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "sendMail" => {
                    let to_email = ensure_required_string(
                        "Outlook",
                        "Recipient Email",
                        get_optional_string_param(context, "toEmail", item_index).await,
                        "Set recipient email.",
                    )?;
                    let subject = ensure_required_string(
                        "Outlook",
                        "Subject",
                        get_optional_string_param(context, "subject", item_index).await,
                        "Set email subject.",
                    )?;
                    let content = ensure_required_string(
                        "Outlook",
                        "Content",
                        get_optional_string_param(context, "content", item_index).await,
                        "Set email body content.",
                    )?;
                    (
                        "POST".to_string(),
                        build_url(&base_url, "/me/sendMail"),
                        Some(json!({
                            "message": {
                                "subject": subject,
                                "body": {
                                    "contentType": "Text",
                                    "content": content,
                                },
                                "toRecipients": [{
                                    "emailAddress": { "address": to_email }
                                }]
                            },
                            "saveToSentItems": true
                        })),
                    )
                }
                "listMessages" => ("GET".to_string(), build_url(&base_url, "/me/messages"), None),
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Outlook",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide API path like /me/events.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Outlook".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Outlook",
                PreparedRequest {
                    method,
                    url,
                    headers,
                    query,
                    body,
                    auth_token: Some(auth_token),
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
    use mockito::Server;

    #[tokio::test]
    async fn outlook_send_mail_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/me/sendMail")
            .match_header("authorization", "Bearer out-token")
            .with_status(202)
            .create_async()
            .await;

        let mut context = MockContext::new("Outlook", "barqflow-nodes.outlook");
        context.add_param("operation", json!("sendMail"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("out-token"));
        context.add_param("toEmail", json!("a@example.com"));
        context.add_param("subject", json!("Hello"));
        context.add_param("content", json!("Body"));

        let result = OutlookNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;

        assert_eq!(result[0][0].json.0.get("status").and_then(|v| v.as_u64()), Some(202));
    }

    #[tokio::test]
    async fn outlook_requires_auth_token() {
        let mut context = MockContext::new("Outlook", "barqflow-nodes.outlook");
        context.add_param("operation", json!("listMessages"));

        let err = OutlookNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
