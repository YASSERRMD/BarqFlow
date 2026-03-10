use crate::integration::common::{
    build_standard_output, build_url, ensure_required_string, execute_prepared_request,
    get_optional_param, get_optional_string_param, get_string_param, get_u64_param, parse_body,
    parse_kv_pairs, resolve_parameter_from_node_or_credentials, run_count, PreparedRequest,
};
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use reqwest::Client;
use serde_json::json;

pub struct PipedriveNode {
    client: Client,
}

impl PipedriveNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for PipedriveNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for PipedriveNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Pipedrive",
            "description": "List and create deals in Pipedrive"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "listDeals").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://api.pipedrive.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let api_token = resolve_parameter_from_node_or_credentials(
                context,
                "Pipedrive",
                "apiToken",
                "API Token",
                item_index,
                "pipedriveApi",
                &["apiToken"],
            )
            .await?;

            let headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            let mut query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            query.push(("api_token".to_string(), api_token));

            let (method, url, body) = match operation.as_str() {
                "listDeals" => (
                    "GET".to_string(),
                    build_url(&base_url, "/api/v1/deals"),
                    None,
                ),
                "createDeal" => {
                    let title = ensure_required_string(
                        "Pipedrive",
                        "Title",
                        get_optional_string_param(context, "title", item_index).await,
                        "Provide deal title.",
                    )?;
                    (
                        "POST".to_string(),
                        build_url(&base_url, "/api/v1/deals"),
                        Some(json!({"title": title})),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Pipedrive",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide a Pipedrive API path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Pipedrive".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Pipedrive",
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
    async fn pipedrive_list_deals_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/deals")
            .match_query(Matcher::UrlEncoded("api_token".into(), "pd-token".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Pipedrive", "barqflow-nodes.pipedrive");
        context.add_param("operation", json!("listDeals"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("apiToken", json!("pd-token"));

        let result = PipedriveNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn pipedrive_uses_bound_credential_when_token_is_missing() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v1/deals")
            .match_query(Matcher::UrlEncoded("api_token".into(), "pd-token".into()))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Pipedrive", "barqflow-nodes.pipedrive");
        context.add_param("operation", json!("listDeals"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_credential("pipedriveApi", "apiToken", json!("pd-token"));

        let result = PipedriveNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }
}
