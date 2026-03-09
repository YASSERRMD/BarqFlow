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

pub struct SlackNode {
    client: Client,
}

impl SlackNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl Default for SlackNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for SlackNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Slack",
            "description": "Send Slack messages and call Slack API"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let mut output_items = Vec::new();
        let run_count = run_count(context).await;

        for item_index in 0..run_count {
            let operation = get_string_param(context, "operation", item_index, "postMessage").await;
            let base_url =
                get_string_param(context, "baseUrl", item_index, "https://slack.com").await;
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

            let (method, url, body, needs_auth) = match operation.as_str() {
                "postMessage" => {
                    let channel = ensure_required_string(
                        "Slack",
                        "Channel",
                        get_optional_string_param(context, "channel", item_index).await,
                        "Set the target Slack channel ID or name.",
                    )?;
                    let text = ensure_required_string(
                        "Slack",
                        "Text",
                        get_optional_string_param(context, "text", item_index).await,
                        "Set the message content.",
                    )?;

                    (
                        "POST".to_string(),
                        build_url(&base_url, "/api/chat.postMessage"),
                        Some(json!({
                            "channel": channel,
                            "text": text,
                        })),
                        true,
                    )
                }
                "apiCall" => {
                    let method = get_string_param(context, "method", item_index, "GET").await;
                    let resource_path = ensure_required_string(
                        "Slack",
                        "Resource Path",
                        get_optional_string_param(context, "resourcePath", item_index).await,
                        "Provide a Slack API path like /api/users.list.",
                    )?;
                    query = get_optional_param(context, "queryParameters", item_index)
                        .await
                        .map(|v| parse_kv_pairs(&v))
                        .unwrap_or_default();
                    let body = parse_body(get_optional_param(context, "body", item_index).await);

                    (method, build_url(&base_url, &resource_path), body, true)
                }
                _ => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Slack".to_string(),
                        message: format!("Operation '{}' is not supported", operation),
                    })
                }
            };

            let auth_token = if needs_auth {
                Some(
                    resolve_auth_token(context, "Slack", item_index, "slackApi", &["accessToken"])
                        .await?,
                )
            } else {
                None
            };

            let response = execute_prepared_request(
                &self.client,
                "Slack",
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
    async fn slack_post_message_executes_real_request() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/api/chat.postMessage")
            .match_header("authorization", "Bearer slack-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"ok":true,"ts":"123"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Slack", "barqflow-nodes.slack");
        context.add_param("operation", json!("postMessage"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("authToken", json!("slack-token"));
        context.add_param("channel", json!("C001"));
        context.add_param("text", json!("hello"));

        let result = SlackNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;

        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
        assert_eq!(
            result[0][0]
                .json
                .0
                .get("operation")
                .and_then(|v| v.as_str()),
            Some("postMessage")
        );
    }

    #[tokio::test]
    async fn slack_uses_bound_credential_when_auth_token_is_empty() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/api/chat.postMessage")
            .match_header("authorization", "Bearer slack-token")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"ok":true}"#)
            .create_async()
            .await;

        let mut context = MockContext::new("Slack", "barqflow-nodes.slack");
        context.add_param("operation", json!("postMessage"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("channel", json!("C001"));
        context.add_param("text", json!("hello"));
        context.add_credential("slackApi", "accessToken", json!("slack-token"));

        let result = SlackNode::new().execute(&context).await.unwrap();
        mock.assert_async().await;
        assert_eq!(
            result[0][0].json.0.get("status").and_then(|v| v.as_u64()),
            Some(200)
        );
    }

    #[tokio::test]
    async fn slack_requires_auth_token() {
        let mut context = MockContext::new("Slack", "barqflow-nodes.slack");
        context.add_param("operation", json!("postMessage"));
        context.add_param("channel", json!("C001"));
        context.add_param("text", json!("hello"));

        let err = SlackNode::new().execute(&context).await.unwrap_err();
        assert!(err.to_string().contains("Auth Token"));
    }
}
