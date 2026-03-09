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

pub struct ClickupNode {
    client: Client,
}

impl ClickupNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for ClickupNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for ClickupNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "ClickUp",
            "description": "List spaces and create tasks in ClickUp"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "listSpaces").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://api.clickup.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = ensure_required_string(
                "ClickUp",
                "Auth Token",
                get_optional_string_param(context, "authToken", item_index).await,
                "Add a ClickUp API token in the node configuration.",
            )?;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("Authorization".to_string(), auth_token));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "listSpaces" => {
                    let team_id = ensure_required_string(
                        "ClickUp",
                        "Team ID",
                        get_optional_string_param(context, "teamId", item_index).await,
                        "Provide the workspace/team ID.",
                    )?;
                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/api/v2/team/{team_id}/space")),
                        None,
                    )
                }
                "createTask" => {
                    let list_id = ensure_required_string(
                        "ClickUp",
                        "List ID",
                        get_optional_string_param(context, "listId", item_index).await,
                        "Provide the list ID where task will be created.",
                    )?;
                    let name = ensure_required_string(
                        "ClickUp",
                        "Name",
                        get_optional_string_param(context, "name", item_index).await,
                        "Provide the task name.",
                    )?;
                    let description =
                        get_optional_string_param(context, "description", item_index).await;
                    (
                        "POST".to_string(),
                        build_url(&base_url, &format!("/api/v2/list/{list_id}/task")),
                        Some(json!({
                            "name": name,
                            "description": description,
                        })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "ClickUp",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide a ClickUp API path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "ClickUp".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "ClickUp",
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
    async fn clickup_list_spaces_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v2/team/11/space")
            .match_header("authorization", "token-1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"spaces":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("ClickUp", "barqflow-nodes.clickUp");
        context.add_param("operation", json!("listSpaces"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("token-1"));
        context.add_param("teamId", json!("11"));

        let result = ClickupNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn clickup_requires_auth_token() {
        let mut context = MockContext::new("ClickUp", "barqflow-nodes.clickUp");
        context.add_param("operation", json!("listSpaces"));
        context.add_param("teamId", json!("11"));

        let err = ClickupNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
