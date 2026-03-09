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

pub struct AwsS3Node {
    client: Client,
}

impl AwsS3Node {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for AwsS3Node {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for AwsS3Node {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "AWS S3",
            "description": "Get or put S3 objects via pre-signed URL or gateway"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "getObject").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://s3.amazonaws.com").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let presigned_url =
                get_optional_string_param(context, "preSignedUrl", item_index).await;

            let headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body, auth_token) = if let Some(pre_signed) = presigned_url {
                let method = match operation.as_str() {
                    "putObject" => "PUT",
                    _ => "GET",
                }
                .to_string();
                let body = if operation == "putObject" {
                    parse_body(get_optional_param(context, "body", item_index).await)
                } else {
                    None
                };
                (method, pre_signed, body, None)
            } else {
                let auth_token = require_auth_token(
                    "AWS S3",
                    get_optional_string_param(context, "authToken", item_index).await,
                )?;
                let bucket = ensure_required_string(
                    "AWS S3",
                    "Bucket Name",
                    get_optional_string_param(context, "bucketName", item_index).await,
                    "Set bucket name or provide pre-signed URL.",
                )?;
                let object_key = ensure_required_string(
                    "AWS S3",
                    "Object Key",
                    get_optional_string_param(context, "objectKey", item_index).await,
                    "Set object key or provide pre-signed URL.",
                )?;
                let object_path = format!("/{bucket}/{object_key}");

                match operation.as_str() {
                    "getObject" => (
                        "GET".to_string(),
                        build_url(&base_url, &object_path),
                        None,
                        Some(auth_token),
                    ),
                    "putObject" => {
                        let body =
                            parse_body(get_optional_param(context, "body", item_index).await)
                                .ok_or_else(|| {
                                    BarqError::NodeOperationError {
                            node_name: "AWS S3".to_string(),
                            message:
                                "Missing Body. Provide object content for putObject operation."
                                    .to_string(),
                        }
                                })?;
                        (
                            "PUT".to_string(),
                            build_url(&base_url, &object_path),
                            Some(body),
                            Some(auth_token),
                        )
                    }
                    "apiCall" => {
                        let method = get_string_param(context, "method", item_index, "GET").await;
                        let resource_path = ensure_required_string(
                            "AWS S3",
                            "Resource Path",
                            get_optional_string_param(context, "resourcePath", item_index).await,
                            "Provide resource path like /bucket/key.",
                        )?;
                        let body =
                            parse_body(get_optional_param(context, "body", item_index).await);
                        (
                            method,
                            build_url(&base_url, &resource_path),
                            body,
                            Some(auth_token),
                        )
                    }
                    _ => {
                        return Err(BarqError::NodeOperationError {
                            node_name: "AWS S3".to_string(),
                            message: format!("Operation '{}' is not supported", operation),
                        });
                    }
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "AWS S3",
                PreparedRequest {
                    method,
                    url,
                    headers,
                    query,
                    body,
                    auth_token,
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
    async fn s3_get_object_with_gateway_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/bucket-a/file.txt")
            .match_header("authorization", "Bearer s3-token")
            .with_status(200)
            .with_header("content-type", "text/plain")
            .with_body("ok")
            .create_async()
            .await;

        let mut context = MockContext::new("AWS S3", "barqflow-nodes.awsS3");
        context.add_param("operation", json!("getObject"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("s3-token"));
        context.add_param("bucketName", json!("bucket-a"));
        context.add_param("objectKey", json!("file.txt"));

        let result = AwsS3Node::new().execute(&context).await.unwrap();
        mock.assert_async().await;

        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn s3_requires_auth_without_presigned_url() {
        let mut context = MockContext::new("AWS S3", "barqflow-nodes.awsS3");
        context.add_param("operation", json!("getObject"));
        context.add_param("bucketName", json!("bucket-a"));
        context.add_param("objectKey", json!("file.txt"));

        let err = AwsS3Node::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
