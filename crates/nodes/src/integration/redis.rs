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

pub struct RedisNode {
    client: Client,
}

impl RedisNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for RedisNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for RedisNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Redis",
            "description": "Execute Redis REST commands (Upstash-style)"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "get").await;
            let base_url = get_string_param(
                context,
                "baseUrl",
                item_index,
                "https://your-upstash-endpoint.upstash.io",
            )
            .await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;
            let auth_token = require_auth_token(
                "Redis",
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
                "get" => {
                    let key = ensure_required_string(
                        "Redis",
                        "Key",
                        get_optional_string_param(context, "key", item_index).await,
                        "Set the Redis key.",
                    )?;
                    (
                        "GET".to_string(),
                        build_url(&base_url, &format!("/get/{key}")),
                        None,
                    )
                }
                "set" => {
                    let key = ensure_required_string(
                        "Redis",
                        "Key",
                        get_optional_string_param(context, "key", item_index).await,
                        "Set the Redis key.",
                    )?;
                    let value = ensure_required_string(
                        "Redis",
                        "Value",
                        get_optional_string_param(context, "value", item_index).await,
                        "Set the Redis value.",
                    )?;
                    (
                        "POST".to_string(),
                        build_url(&base_url, &format!("/set/{key}/{value}")),
                        None,
                    )
                }
                "delete" => {
                    let key = ensure_required_string(
                        "Redis",
                        "Key",
                        get_optional_string_param(context, "key", item_index).await,
                        "Set the Redis key.",
                    )?;
                    (
                        "POST".to_string(),
                        build_url(&base_url, &format!("/del/{key}")),
                        None,
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Redis",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide command path like /ping.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (method, build_url(&base_url, &resource_path), body)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Redis".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Redis",
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
    async fn redis_get_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/get/myKey")
            .match_header("authorization", "Bearer redis-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"result":"value"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Redis", "barqflow-nodes.redis");
        context.add_param("operation", json!("get"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("redis-token"));
        context.add_param("key", json!("myKey"));

        let result = RedisNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn redis_requires_auth_token() {
        let mut context = MockContext::new("Redis", "barqflow-nodes.redis");
        context.add_param("operation", json!("get"));
        context.add_param("key", json!("myKey"));

        let err = RedisNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
