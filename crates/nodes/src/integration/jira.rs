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

pub struct JiraNode {
    client: Client,
}

impl JiraNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for JiraNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for JiraNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Jira",
            "description": "Manage Jira issues"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let mut output_items = Vec::new();
        let run_count = run_count(context).await;

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "getIssue").await;
            let base_url = get_string_param(
                context,
                "baseUrl",
                item_index,
                "https://your-domain.atlassian.net",
            )
            .await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = require_auth_token(
                "Jira",
                get_optional_string_param(context, "authToken", item_index).await,
            )?;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("Accept".to_string(), "application/json".to_string()));
            headers.push(("Content-Type".to_string(), "application/json".to_string()));

            let mut query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "getIssue" => {
                    let issue_key = ensure_required_string(
                        "Jira",
                        "Issue Key",
                        get_optional_string_param(context, "issueKey", item_index).await,
                        "Provide a Jira issue key such as PROJ-123.",
                    )?;
                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/rest/api/3/issue/{issue_key}")),
                        None,
                    )
                }
                "searchIssues" => {
                    if let Some(jql) = get_optional_string_param(context, "jql", item_index).await {
                        query.push(("jql".to_string(), jql));
                    }
                    (
                        "GET".to_string(),
                        build_url(&base_url, "/rest/api/3/search"),
                        None,
                    )
                }
                "createIssue" => {
                    let fields =
                        parse_body(get_optional_param(context, "issueFields", item_index).await)
                            .ok_or_else(|| BarqError::NodeOperationError {
                                node_name: "Jira".to_string(),
                                message:
                                    "Missing Issue Fields. Provide a JSON object for Jira fields."
                                        .to_string(),
                            })?;
                    (
                        "POST".to_string(),
                        build_url(&base_url, "/rest/api/3/issue"),
                        Some(json!({ "fields": fields })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Jira",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide an API path like /rest/api/3/project.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Jira".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Jira",
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
    async fn jira_get_issue_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/rest/api/3/issue/PROJ-1")
            .match_header("authorization", "Bearer jira-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"10000"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Jira", "barqflow-nodes.jira");
        context.add_param("operation", json!("getIssue"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("jira-token"));
        context.add_param("issueKey", json!("PROJ-1"));

        let result = JiraNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn jira_requires_auth_token() {
        let mut context = MockContext::new("Jira", "barqflow-nodes.jira");
        context.add_param("operation", json!("getIssue"));
        context.add_param("issueKey", json!("PROJ-1"));

        let err = JiraNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
