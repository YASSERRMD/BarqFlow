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

pub struct BitbucketNode {
    client: Client,
}

impl BitbucketNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for BitbucketNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for BitbucketNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(
            json!({"name":"Bitbucket","description":"Use Bitbucket repositories and issues"}),
        )
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation =
                get_string_param(context, "operation", item_index, "listRepositories").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://api.bitbucket.org").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = require_auth_token(
                "Bitbucket",
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
                "listRepositories" => {
                    let workspace = ensure_required_string(
                        "Bitbucket",
                        "Workspace",
                        get_optional_string_param(context, "workspace", item_index).await,
                        "Set Bitbucket workspace slug.",
                    )?;
                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/2.0/repositories/{workspace}")),
                        None,
                    )
                }
                "createIssue" => {
                    let workspace = ensure_required_string(
                        "Bitbucket",
                        "Workspace",
                        get_optional_string_param(context, "workspace", item_index).await,
                        "Set Bitbucket workspace slug.",
                    )?;
                    let repo = ensure_required_string(
                        "Bitbucket",
                        "Repository",
                        get_optional_string_param(context, "repoSlug", item_index).await,
                        "Set repository slug.",
                    )?;
                    let title = ensure_required_string(
                        "Bitbucket",
                        "Title",
                        get_optional_string_param(context, "title", item_index).await,
                        "Set issue title.",
                    )?;
                    (
                        "POST".to_string(),
                        build_url(
                            &base_url,
                            &format!("/2.0/repositories/{workspace}/{repo}/issues"),
                        ),
                        Some(json!({"title": title})),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Bitbucket",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide Bitbucket API path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Bitbucket".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    })
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Bitbucket",
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
    async fn bitbucket_list_repositories_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/2.0/repositories/workspace1")
            .match_header("authorization", "Bearer bb-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"values":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Bitbucket", "barqflow-nodes.bitbucket");
        context.add_param("operation", json!("listRepositories"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("bb-token"));
        context.add_param("workspace", json!("workspace1"));

        let result = BitbucketNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }
}
