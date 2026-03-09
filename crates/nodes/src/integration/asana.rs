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

pub struct AsanaNode {
    client: Client,
}

impl AsanaNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for AsanaNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for AsanaNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Asana",
            "description": "Manage tasks in Asana"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let mut output_items = Vec::new();
        let run_count = run_count(context).await;

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "createTask").await;
            let base_url = get_string_param(
                context,
                "baseUrl",
                item_index,
                "https://app.asana.com/api/1.0",
            )
            .await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token =
                resolve_auth_token(context, "Asana", item_index, "asanaApi", &["accessToken"])
                    .await?;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("Content-Type".to_string(), "application/json".to_string()));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "createTask" => {
                    let workspace = ensure_required_string(
                        "Asana",
                        "Workspace ID",
                        get_optional_string_param(context, "workspace", item_index).await,
                        "Set the Asana workspace ID.",
                    )?;
                    let project = ensure_required_string(
                        "Asana",
                        "Project ID",
                        get_optional_string_param(context, "project", item_index).await,
                        "Set the Asana project ID.",
                    )?;
                    let task_name = ensure_required_string(
                        "Asana",
                        "Task Name",
                        get_optional_string_param(context, "name", item_index).await,
                        "Set the task name.",
                    )?;
                    let notes = get_optional_string_param(context, "notes", item_index).await;

                    (
                        "POST".to_string(),
                        build_url(&base_url, "/tasks"),
                        Some(json!({
                            "data": {
                                "name": task_name,
                                "workspace": workspace,
                                "projects": [project],
                                "notes": notes.unwrap_or_default(),
                            }
                        })),
                    )
                }
                "listProjectTasks" => {
                    let project = ensure_required_string(
                        "Asana",
                        "Project ID",
                        get_optional_string_param(context, "project", item_index).await,
                        "Set the Asana project ID.",
                    )?;
                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/projects/{project}/tasks")),
                        None,
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Asana",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide an API path like /tasks.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Asana".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Asana",
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
    async fn asana_create_task_executes_request() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/tasks")
            .match_header("authorization", "Bearer asana-token")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data":{"gid":"123"}}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Asana", "barqflow-nodes.asana");
        context.add_param("operation", json!("createTask"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("asana-token"));
        context.add_param("workspace", json!("ws1"));
        context.add_param("project", json!("pr1"));
        context.add_param("name", json!("Task A"));

        let result = AsanaNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;

        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(201)
        );
    }

    #[tokio::test]
    async fn asana_uses_bound_credential_when_auth_token_is_empty() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/tasks")
            .match_header("authorization", "Bearer asana-token")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(r#"{"data":{"gid":"123"}}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Asana", "barqflow-nodes.asana");
        context.add_param("operation", json!("createTask"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("workspace", json!("ws1"));
        context.add_param("project", json!("pr1"));
        context.add_param("name", json!("Task A"));
        context.add_credential("asanaApi", "accessToken", json!("asana-token"));

        let result = AsanaNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(201)
        );
    }

    #[tokio::test]
    async fn asana_requires_auth_token() {
        let mut context = MockContext::new("Asana", "barqflow-nodes.asana");
        context.add_param("operation", json!("createTask"));
        context.add_param("workspace", json!("ws1"));
        context.add_param("project", json!("pr1"));
        context.add_param("name", json!("Task A"));

        let err = AsanaNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
