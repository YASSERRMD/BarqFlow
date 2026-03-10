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

pub struct TrelloNode {
    client: Client,
}

impl TrelloNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for TrelloNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for TrelloNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({"name":"Trello","description":"Use Trello boards and cards"}))
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
                get_string_param(context, "baseUrl", item_index, "https://api.trello.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token =
                resolve_auth_token(context, "Trello", item_index, "trelloApi", &["accessToken"])
                    .await?;

            let headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            let mut query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "listBoards" => (
                    "GET".to_string(),
                    build_url(&base_url, "/1/members/me/boards"),
                    None,
                ),
                "createCard" => {
                    let list_id = ensure_required_string(
                        "Trello",
                        "List ID",
                        get_optional_string_param(context, "listId", item_index).await,
                        "Set Trello list ID.",
                    )?;
                    let name = ensure_required_string(
                        "Trello",
                        "Name",
                        get_optional_string_param(context, "name", item_index).await,
                        "Set card name.",
                    )?;
                    let desc = get_optional_string_param(context, "description", item_index).await;
                    query.push(("idList".to_string(), list_id));
                    query.push(("name".to_string(), name));
                    if let Some(desc) = desc {
                        query.push(("desc".to_string(), desc));
                    }
                    ("POST".to_string(), build_url(&base_url, "/1/cards"), None)
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Trello",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide Trello resource path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Trello".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    })
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Trello",
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
    async fn trello_list_boards_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/1/members/me/boards")
            .match_header("authorization", "Bearer tr-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let mut context = MockContext::new("Trello", "barqflow-nodes.trello");
        context.add_param("operation", json!("listBoards"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("tr-token"));

        let result = TrelloNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn trello_uses_bound_credential_when_auth_token_is_empty() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/1/members/me/boards")
            .match_header("authorization", "Bearer tr-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body("[]")
            .create_async()
            .await;

        let mut context = MockContext::new("Trello", "barqflow-nodes.trello");
        context.add_param("operation", json!("listBoards"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_credential("trelloApi", "accessToken", json!("tr-token"));

        let result = TrelloNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }
}
