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

pub struct GoogleDriveNode {
    client: Client,
}

impl GoogleDriveNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for GoogleDriveNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for GoogleDriveNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Google Drive",
            "description": "Read metadata and list files in Google Drive"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "getFile").await;
            let base_url = get_string_param(
                context,
                "baseUrl",
                item_index,
                "https://www.googleapis.com/drive/v3",
            )
            .await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = resolve_auth_token(
                context,
                "Google Drive",
                item_index,
                "googleDriveApi",
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
                "getFile" => {
                    let file_id = ensure_required_string(
                        "Google Drive",
                        "File ID",
                        get_optional_string_param(context, "fileId", item_index).await,
                        "Set the Google Drive file ID.",
                    )?;
                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/files/{file_id}")),
                        None,
                    )
                }
                "listFiles" => ("GET".to_string(), build_url(&base_url, "/files"), None),
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Google Drive",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide an API path like /files.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Google Drive".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Google Drive",
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
    async fn drive_get_file_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/files/file123")
            .match_header("authorization", "Bearer gd-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"file123"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Google Drive", "barqflow-nodes.googleDrive");
        context.add_param("operation", json!("getFile"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("gd-token"));
        context.add_param("fileId", json!("file123"));

        let result = GoogleDriveNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn drive_uses_bound_credential_when_auth_token_is_empty() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/files/file123")
            .match_header("authorization", "Bearer gd-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"file123"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Google Drive", "barqflow-nodes.googleDrive");
        context.add_param("operation", json!("getFile"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("fileId", json!("file123"));
        context.add_credential("googleDriveApi", "accessToken", json!("gd-token"));

        let result = GoogleDriveNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn drive_requires_auth_token() {
        let mut context = MockContext::new("Google Drive", "barqflow-nodes.googleDrive");
        context.add_param("operation", json!("getFile"));
        context.add_param("fileId", json!("file123"));

        let err = GoogleDriveNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
