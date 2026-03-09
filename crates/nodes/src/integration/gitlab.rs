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

pub struct GitlabNode {
    client: Client,
}

impl GitlabNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for GitlabNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for GitlabNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(
            json!({"name":"GitLab","description":"Get projects and create GitLab issues"}),
        )
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "getProject").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://gitlab.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let token = require_auth_token(
                "GitLab",
                get_optional_string_param(context, "authToken", item_index).await,
            )?;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("PRIVATE-TOKEN".to_string(), token));

            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "getProject" => {
                    let project_id = ensure_required_string(
                        "GitLab",
                        "Project ID",
                        get_optional_string_param(context, "projectId", item_index).await,
                        "Set GitLab project ID.",
                    )?;
                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/api/v4/projects/{project_id}")),
                        None,
                    )
                }
                "createIssue" => {
                    let project_id = ensure_required_string(
                        "GitLab",
                        "Project ID",
                        get_optional_string_param(context, "projectId", item_index).await,
                        "Set GitLab project ID.",
                    )?;
                    let title = ensure_required_string(
                        "GitLab",
                        "Title",
                        get_optional_string_param(context, "title", item_index).await,
                        "Set issue title.",
                    )?;
                    let description = get_optional_string_param(context, "description", item_index)
                        .await
                        .unwrap_or_default();
                    (
                        "POST".to_string(),
                        build_url(&base_url, &format!("/api/v4/projects/{project_id}/issues")),
                        Some(json!({"title": title, "description": description})),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "GitLab",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide GitLab API path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "GitLab".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "GitLab",
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
    async fn gitlab_get_project_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/api/v4/projects/123")
            .match_header("private-token", "glpat")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":123}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("GitLab", "barqflow-nodes.gitlab");
        context.add_param("operation", json!("getProject"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("glpat"));
        context.add_param("projectId", json!("123"));

        let result = GitlabNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }
}
