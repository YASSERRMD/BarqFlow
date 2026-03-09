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

pub struct OnedriveNode {
    client: Client,
}

impl OnedriveNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for OnedriveNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for OnedriveNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(
            json!({"name":"OneDrive","description":"List and create OneDrive folders"}),
        )
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "listRoot").await;
            let base_url = get_string_param(
                context,
                "baseUrl",
                item_index,
                "https://graph.microsoft.com",
            )
            .await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = resolve_auth_token(
                context,
                "OneDrive",
                item_index,
                "oneDriveApi",
                &["accessToken"],
            )
            .await?;

            let headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "listRoot" => (
                    "GET".to_string(),
                    build_url(&base_url, "/v1.0/me/drive/root/children"),
                    None,
                ),
                "createFolder" => {
                    let name = ensure_required_string(
                        "OneDrive",
                        "Name",
                        get_optional_string_param(context, "name", item_index).await,
                        "Provide folder name.",
                    )?;
                    (
                        "POST".to_string(),
                        build_url(&base_url, "/v1.0/me/drive/root/children"),
                        Some(json!({
                            "name": name,
                            "folder": {},
                            "@microsoft.graph.conflictBehavior": "rename"
                        })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "OneDrive",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide Graph API path.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "OneDrive".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    })
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "OneDrive",
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
    async fn onedrive_list_root_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/v1.0/me/drive/root/children")
            .match_header("authorization", "Bearer od-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"value":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("OneDrive", "barqflow-nodes.onedrive");
        context.add_param("operation", json!("listRoot"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("od-token"));

        let result = OnedriveNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn onedrive_uses_bound_credential_when_auth_token_is_empty() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/v1.0/me/drive/root/children")
            .match_header("authorization", "Bearer od-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"value":[]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("OneDrive", "barqflow-nodes.onedrive");
        context.add_param("operation", json!("listRoot"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_credential("oneDriveApi", "accessToken", json!("od-token"));

        let result = OnedriveNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }
}
