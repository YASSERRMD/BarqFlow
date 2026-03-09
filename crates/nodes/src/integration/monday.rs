use crate::integration::common::{
    build_standard_output, build_url, ensure_required_string, execute_prepared_request,
    get_optional_param, get_optional_string_param, get_string_param, get_u64_param, parse_body,
    parse_kv_pairs, run_count, PreparedRequest,
};
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use reqwest::Client;
use serde_json::json;

pub struct MondayNode {
    client: Client,
}

impl MondayNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for MondayNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for MondayNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Monday.com",
            "description": "List boards and create items in Monday.com"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "listBoards").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://api.monday.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = ensure_required_string(
                "Monday.com",
                "Auth Token",
                get_optional_string_param(context, "authToken", item_index).await,
                "Add a Monday.com API token in the node configuration.",
            )?;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("Authorization".to_string(), auth_token));
            headers.push(("Content-Type".to_string(), "application/json".to_string()));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "listBoards" => (
                    "POST".to_string(),
                    build_url(&base_url, "/v2"),
                    Some(json!({
                        "query": "query { boards(limit: 25) { id name } }"
                    })),
                ),
                "createItem" => {
                    let board_id = ensure_required_string(
                        "Monday.com",
                        "Board ID",
                        get_optional_string_param(context, "boardId", item_index).await,
                        "Provide the target board ID.",
                    )?;
                    let item_name = ensure_required_string(
                        "Monday.com",
                        "Item Name",
                        get_optional_string_param(context, "itemName", item_index).await,
                        "Provide the item name.",
                    )?;
                    (
                        "POST".to_string(),
                        build_url(&base_url, "/v2"),
                        Some(json!({
                            "query": "mutation($boardId: ID!, $itemName: String!) { create_item(board_id: $boardId, item_name: $itemName) { id } }",
                            "variables": {
                                "boardId": board_id,
                                "itemName": item_name,
                            }
                        })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Monday.com",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide a Monday.com API path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Monday.com".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Monday.com",
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
    use mockito::Server;

    #[tokio::test]
    async fn monday_list_boards_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/v2")
            .match_header("authorization", "monday-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data":{"boards":[]}}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Monday.com", "barqflow-nodes.monday");
        context.add_param("operation", json!("listBoards"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("monday-token"));

        let result = MondayNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn monday_requires_auth_token() {
        let mut context = MockContext::new("Monday.com", "barqflow-nodes.monday");
        context.add_param("operation", json!("listBoards"));

        let err = MondayNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
