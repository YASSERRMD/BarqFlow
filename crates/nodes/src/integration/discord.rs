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

pub struct DiscordNode {
    client: Client,
}

impl DiscordNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for DiscordNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for DiscordNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Discord",
            "description": "Send Discord webhook messages and call Discord API"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let mut output_items = Vec::new();
        let run_count = run_count(context).await;

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "sendWebhook").await;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;

            let mut headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            headers.push(("Content-Type".to_string(), "application/json".to_string()));

            let mut query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body, auth_token) = match operation.as_str() {
                "sendWebhook" => {
                    let webhook_url = ensure_required_string(
                        "Discord",
                        "Webhook URL",
                        get_optional_string_param(context, "webhookUrl", item_index).await,
                        "Provide the Discord webhook URL.",
                    )?;
                    let content = ensure_required_string(
                        "Discord",
                        "Content",
                        get_optional_string_param(context, "content", item_index).await,
                        "Provide message content.",
                    )?;

                    (
                        "POST".to_string(),
                        webhook_url,
                        Some(json!({ "content": content })),
                        None,
                    )
                }
                "apiCall" => {
                    let base_url = get_string_param(
                        context,
                        "baseUrl",
                        item_index,
                        "https://discord.com/api/v10",
                    )
                    .await;
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Discord",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide an API path like /users/@me.",
                    )?;
                    query = get_optional_param(context, "queryParameters", item_index)
                        .await
                        .map(|v| parse_kv_pairs(&v))
                        .unwrap_or_default();
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    let token = resolve_auth_token(
                        context,
                        "Discord",
                        item_index,
                        "discordApi",
                        &["botToken", "accessToken"],
                    )
                    .await?;

                    (
                        method,
                        build_url(&base_url, &resource_path),
                        body,
                        Some(token),
                    )
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Discord".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Discord",
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
    async fn discord_send_webhook_executes_request() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/discord-webhook")
            .with_status(204)
            .create_async()
            .await;

        let mut context = MockContext::new("Discord", "barqflow-nodes.discord");
        context.add_param("operation", json!("sendWebhook"));
        context.add_param(
            "webhookUrl",
            json!(format!("{}/discord-webhook", server.url())),
        );
        context.add_param("content", json!("hello"));

        let result = DiscordNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;

        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(204)
        );
    }

    #[tokio::test]
    async fn discord_api_call_uses_bound_credential_when_auth_token_is_empty() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/users/@me")
            .match_header("authorization", "Bearer discord-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"id":"u1"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Discord", "barqflow-nodes.discord");
        context.add_param("operation", json!("apiCall"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("resourcePath", json!("/users/@me"));
        context.add_credential("discordApi", "botToken", json!("discord-token"));

        let result = DiscordNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn discord_api_call_requires_token() {
        let mut context = MockContext::new("Discord", "barqflow-nodes.discord");
        context.add_param("operation", json!("apiCall"));
        context.add_param("resourcePath", json!("/users/@me"));

        let err = DiscordNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
