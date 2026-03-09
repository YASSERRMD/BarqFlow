use crate::integration::common::{
    build_standard_output, build_url, ensure_required_string, execute_prepared_request,
    get_optional_param, get_optional_string_param, get_string_param, get_u64_param, parse_body,
    parse_kv_pairs, resolve_auth_token, run_count, PreparedRequest,
};
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use reqwest::Client;
use serde_json::json;

pub struct NotionNode {
    client: Client,
}

impl NotionNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for NotionNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for NotionNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Notion",
            "description": "Interact with Notion API"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let mut output_items = Vec::new();
        let run_count = run_count(context).await;

        for item_index in 0..run_count {
            let operation =
                get_string_param(context, "operation", item_index, "queryDatabase").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://api.notion.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token =
                resolve_auth_token(context, "Notion", item_index, "notionApi", &["accessToken"])
                    .await?;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("Notion-Version".to_string(), "2022-06-28".to_string()));
            headers.push(("Content-Type".to_string(), "application/json".to_string()));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "queryDatabase" => {
                    let database_id = ensure_required_string(
                        "Notion",
                        "Database ID",
                        get_optional_string_param(context, "databaseId", item_index).await,
                        "Provide the Notion database ID.",
                    )?;
                    let filter_body =
                        parse_body(get_optional_param(context, "filter", item_index).await)
                            .unwrap_or_else(|| json!({}));
                    (
                        "POST".to_string(),
                        build_url(&base_url, &format!("/v1/databases/{database_id}/query")),
                        Some(filter_body),
                    )
                }
                "createPage" => {
                    let database_id = ensure_required_string(
                        "Notion",
                        "Database ID",
                        get_optional_string_param(context, "databaseId", item_index).await,
                        "Provide the Notion database ID.",
                    )?;
                    let properties =
                        parse_body(get_optional_param(context, "properties", item_index).await)
                            .ok_or_else(|| {
                                BarqError::NodeOperationError {
                        node_name: "Notion".to_string(),
                        message:
                            "Missing Properties. Provide a JSON object for the new page properties."
                                .to_string(),
                    }
                            })?;

                    (
                        "POST".to_string(),
                        build_url(&base_url, "/v1/pages"),
                        Some(json!({
                            "parent": { "database_id": database_id },
                            "properties": properties,
                        })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Notion",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide an API path like /v1/users.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Notion".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Notion",
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
    async fn notion_query_database_executes_request() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/databases/db123/query")
            .match_header("authorization", "Bearer nt_secret")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Notion", "barqflow-nodes.notion");
        context.add_param("operation", json!("queryDatabase"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("nt_secret"));
        context.add_param("databaseId", json!("db123"));

        let result = NotionNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;

        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn notion_uses_bound_credential_when_auth_token_is_empty() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v1/databases/db123/query")
            .match_header("authorization", "Bearer nt_secret")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"results":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Notion", "barqflow-nodes.notion");
        context.add_param("operation", json!("queryDatabase"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("databaseId", json!("db123"));
        context.add_credential("notionApi", "accessToken", json!("nt_secret"));

        let result = NotionNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn notion_requires_auth_token() {
        let mut context = MockContext::new("Notion", "barqflow-nodes.notion");
        context.add_param("operation", json!("queryDatabase"));
        context.add_param("databaseId", json!("db123"));

        let err = NotionNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
