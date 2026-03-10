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

pub struct FreshdeskNode {
    client: Client,
}

impl FreshdeskNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for FreshdeskNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for FreshdeskNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Freshdesk",
            "description": "List and create Freshdesk tickets"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "listTickets").await;
            let base_url = get_string_param(
                context,
                "baseUrl",
                item_index,
                "https://example.freshdesk.com",
            )
            .await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let api_key = resolve_api_key(
                context,
                "Freshdesk",
                item_index,
                "freshdeskApi",
                &["apiKey"],
            )
            .await?;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            let basic = STANDARD.encode(format!("{}:X", api_key));
            headers.push(("Authorization".to_string(), format!("Basic {}", basic)));
            headers.push(("Content-Type".to_string(), "application/json".to_string()));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "listTickets" => (
                    "GET".to_string(),
                    build_url(&base_url, "/api/v2/tickets"),
                    None,
                ),
                "createTicket" => {
                    let subject = ensure_required_string(
                        "Freshdesk",
                        "Subject",
                        get_optional_string_param(context, "subject", item_index).await,
                        "Provide ticket subject.",
                    )?;
                    let email = ensure_required_string(
                        "Freshdesk",
                        "Email",
                        get_optional_string_param(context, "email", item_index).await,
                        "Provide requester email.",
                    )?;
                    let description = ensure_required_string(
                        "Freshdesk",
                        "Description",
                        get_optional_string_param(context, "description", item_index).await,
                        "Provide ticket description.",
                    )?;
                    (
                        "POST".to_string(),
                        build_url(&base_url, "/api/v2/tickets"),
                        Some(json!({
                            "subject": subject,
                            "email": email,
                            "description": description,
                            "status": 2,
                            "priority": 1
                        })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Freshdesk",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide a Freshdesk API path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Freshdesk".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Freshdesk",
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
    use mockito::{Matcher, Server};

    #[tokio::test]
    async fn freshdesk_list_tickets_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v2/tickets")
            .match_header("authorization", Matcher::Regex("Basic .+".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let mut context = MockContext::new("Freshdesk", "barqflow-nodes.freshdesk");
        context.add_param("operation", json!("listTickets"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("apiKey", json!("fd-key"));

        let result = FreshdeskNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn freshdesk_uses_bound_credential_when_api_key_is_empty() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v2/tickets")
            .match_header("authorization", Matcher::Regex("Basic .+".to_string()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let mut context = MockContext::new("Freshdesk", "barqflow-nodes.freshdesk");
        context.add_param("operation", json!("listTickets"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_credential("freshdeskApi", "apiKey", json!("fd-key"));

        let result = FreshdeskNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }
}
