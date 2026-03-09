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

pub struct GithubNode {
    client: Client,
}

impl GithubNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for GithubNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for GithubNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "GitHub",
            "description": "Interact with GitHub API"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let mut output_items = Vec::new();
        let run_count = run_count(context).await;

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "getRepo").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://api.github.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;

            let owner = get_optional_string_param(context, "owner", item_index).await;
            let repo = get_optional_string_param(context, "repo", item_index).await;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push((
                "Accept".to_string(),
                "application/vnd.github+json".to_string(),
            ));
            headers.push(("X-GitHub-Api-Version".to_string(), "2022-11-28".to_string()));
            headers.push(("User-Agent".to_string(), "BarqFlow".to_string()));

            let mut query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "getRepo" => {
                    let owner = ensure_required_string(
                        "GitHub",
                        "Owner",
                        owner.clone(),
                        "Set the repository owner.",
                    )?;
                    let repo = ensure_required_string(
                        "GitHub",
                        "Repo",
                        repo.clone(),
                        "Set the repository name.",
                    )?;

                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/repos/{owner}/{repo}")),
                        None,
                    )
                }
                "listIssues" => {
                    let owner = ensure_required_string(
                        "GitHub",
                        "Owner",
                        owner.clone(),
                        "Set the repository owner.",
                    )?;
                    let repo = ensure_required_string(
                        "GitHub",
                        "Repo",
                        repo.clone(),
                        "Set the repository name.",
                    )?;
                    query.extend(
                        get_optional_param(context, "queryParameters", item_index)
                            .await
                            .map(|v| parse_kv_pairs(&v))
                            .unwrap_or_default(),
                    );
                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/repos/{owner}/{repo}/issues")),
                        None,
                    )
                }
                "createIssue" => {
                    let owner = ensure_required_string(
                        "GitHub",
                        "Owner",
                        owner.clone(),
                        "Set the repository owner.",
                    )?;
                    let repo = ensure_required_string(
                        "GitHub",
                        "Repo",
                        repo.clone(),
                        "Set the repository name.",
                    )?;
                    let title = ensure_required_string(
                        "GitHub",
                        "Issue Title",
                        get_optional_string_param(context, "issueTitle", item_index).await,
                        "Set the issue title.",
                    )?;
                    let issue_body = get_optional_string_param(context, "issueBody", item_index)
                        .await
                        .unwrap_or_default();
                    (
                        "POST".to_string(),
                        build_url(&base_url, &format!("/repos/{owner}/{repo}/issues")),
                        Some(json!({
                            "title": title,
                            "body": issue_body,
                        })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "GitHub",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide an API path like /user/repos.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "GitHub".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let auth_token = require_auth_token(
                "GitHub",
                get_optional_string_param(context, "authToken", item_index).await,
            )?;

            let response = execute_prepared_request(
                &self.client,
                "GitHub",
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
    async fn github_get_repo_returns_payload() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/repos/octo/repo")
            .match_header("authorization", "Bearer ghp_xxx")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":1,"full_name":"octo/repo"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("GitHub", "barqflow-nodes.github");
        context.add_param("operation", json!("getRepo"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("ghp_xxx"));
        context.add_param("owner", json!("octo"));
        context.add_param("repo", json!("repo"));

        let result = GithubNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;

        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn github_requires_auth_token() {
        let mut context = MockContext::new("GitHub", "barqflow-nodes.github");
        context.add_param("operation", json!("getRepo"));
        context.add_param("owner", json!("octo"));
        context.add_param("repo", json!("repo"));

        let err = GithubNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
