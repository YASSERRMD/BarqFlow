use crate::integration::common::{
    build_standard_output, ensure_required_string, execute_prepared_request, get_optional_param,
    get_optional_string_param, get_string_param, get_u64_param, parse_body, parse_kv_pairs,
    run_count, PreparedRequest,
};
use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use reqwest::Client;
use serde_json::json;

pub struct TelegramNode {
    client: Client,
}

impl TelegramNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn build_bot_endpoint(base_url: &str, bot_token: &str, resource: &str) -> String {
        let base = base_url.trim().trim_end_matches('/');
        let clean = resource.trim().trim_start_matches('/');
        format!("{}/bot{}/{}", base, bot_token, clean)
    }
}

impl Default for TelegramNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for TelegramNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Telegram",
            "description": "Send Telegram messages and call Telegram Bot API"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = run_count(context).await;
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "sendMessage").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://api.telegram.org").await;
            let bot_token = ensure_required_string(
                "Telegram",
                "Bot Token",
                get_optional_string_param(context, "botToken", item_index).await,
                "Set the Telegram bot token in the node.",
            )?;
            let timeout_ms = get_u64_param(context, "timeout", item_index, 60_000).await;

            let headers = get_optional_param(context, "headers", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();
            let query = get_optional_param(context, "queryParameters", item_index)
                .await
                .map(|v| parse_kv_pairs(&v))
                .unwrap_or_default();

            let (method, url, body) = match operation.as_str() {
                "sendMessage" => {
                    let chat_id = ensure_required_string(
                        "Telegram",
                        "Chat ID",
                        get_optional_string_param(context, "chatId", item_index).await,
                        "Set the target chat ID.",
                    )?;
                    let text = ensure_required_string(
                        "Telegram",
                        "Text",
                        get_optional_string_param(context, "text", item_index).await,
                        "Set the message text.",
                    )?;

                    (
                        "POST".to_string(),
                        Self::build_bot_endpoint(&base_url, &bot_token, "sendMessage"),
                        Some(json!({
                            "chat_id": chat_id,
                            "text": text,
                        })),
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Telegram",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide a method path like getMe or getUpdates.",
                    )?;
                    let body = parse_body(get_optional_param(context, "body", item_index).await);
                    (
                        method,
                        Self::build_bot_endpoint(&base_url, &bot_token, &resource_path),
                        body,
                    )
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Telegram".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    });
                }
            };

            let response = execute_prepared_request(
                &self.client,
                "Telegram",
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
    async fn telegram_send_message_works() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/botbot123/sendMessage")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Telegram", "barqflow-nodes.telegram");
        context.add_param("operation", json!("sendMessage"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("botToken", json!("bot123"));
        context.add_param("chatId", json!("12345"));
        context.add_param("text", json!("hi"));

        let result = TelegramNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;

        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn telegram_requires_bot_token() {
        let mut context = MockContext::new("Telegram", "barqflow-nodes.telegram");
        context.add_param("operation", json!("sendMessage"));
        context.add_param("chatId", json!("12345"));
        context.add_param("text", json!("hi"));

        let err = TelegramNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Bot Token"));
    }
}
