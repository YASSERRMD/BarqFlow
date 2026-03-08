use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use futures_util::FutureExt;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::panic::AssertUnwindSafe;
use std::time::Duration;

pub struct OpenAINode {
    client: Client,
}

impl OpenAINode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn extract_response_text(payload: &Value) -> String {
        payload
            .get("choices")
            .and_then(|choices| choices.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .map(|s| s.to_string())
            .or_else(|| {
                payload
                    .get("output_text")
                    .and_then(|content| content.as_str())
                    .map(|s| s.to_string())
            })
            .unwrap_or_default()
    }

    fn status_hint(status: StatusCode) -> &'static str {
        match status {
            StatusCode::UNAUTHORIZED => {
                "Unauthorized (401). Check the OpenAI API key in /credentials."
            }
            StatusCode::FORBIDDEN => "Forbidden (403). The key does not have access to this model.",
            StatusCode::TOO_MANY_REQUESTS => "Rate limited (429). Retry later.",
            StatusCode::BAD_REQUEST => "Bad request (400). Verify model and payload fields.",
            _ => "",
        }
    }
}

impl Default for OpenAINode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for OpenAINode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "OpenAI",
            "description": "Interact with OpenAI APIs"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let run_count = match AssertUnwindSafe(context.get_input_data(0))
            .catch_unwind()
            .await
        {
            Ok(Ok(input_data)) => {
                if input_data.is_empty() {
                    1
                } else {
                    input_data.len()
                }
            }
            Ok(Err(_)) => 1,
            Err(_) => {
                context.log(
                    "Input data probe panicked; defaulting OpenAI node to single-item execution.",
                );
                1
            }
        };
        let mut output_items = Vec::new();

        let creds = context.get_credentials("openAiApi").await?;
        let api_key = creds
            .get("apiKey")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if api_key.is_empty() {
            return Err(BarqError::NodeOperationError {
                node_name: "OpenAI".to_string(),
                message:
                    "Missing OpenAI API key in credential 'openAiApi'. Go to /credentials and add or update it."
                        .to_string(),
            });
        }

        let credential_base_url = creds
            .get("baseUrl")
            .and_then(|v| v.as_str())
            .unwrap_or("https://api.openai.com/v1")
            .trim_end_matches('/')
            .to_string();

        for item_index in 0..run_count {
            let operation = context
                .get_node_parameter_at_item("operation", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("chatCompletion").to_string())
                .unwrap_or_else(|_| "chatCompletion".to_string());

            if operation != "chatCompletion" {
                return Err(BarqError::NodeOperationError {
                    node_name: "OpenAI".to_string(),
                    message: format!("Operation '{}' is not supported", operation),
                });
            }

            let model = context
                .get_node_parameter_at_item("model", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("gpt-4o-mini").to_string())
                .unwrap_or_else(|_| "gpt-4o-mini".to_string());
            let prompt = context
                .get_node_parameter_at_item("prompt", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("").to_string())
                .unwrap_or_default();
            if prompt.trim().is_empty() {
                return Err(BarqError::NodeOperationError {
                    node_name: "OpenAI".to_string(),
                    message: "Prompt cannot be empty. Fill the Prompt parameter in the node."
                        .to_string(),
                });
            }

            let system_prompt = context
                .get_node_parameter_at_item("systemPrompt", item_index, None)
                .await
                .ok()
                .and_then(|v| v.as_str().map(|s| s.to_string()))
                .unwrap_or_default();
            let temperature = context
                .get_node_parameter_at_item("temperature", item_index, None)
                .await
                .ok()
                .and_then(|v| {
                    v.as_f64()
                        .or_else(|| v.as_str().and_then(|s| s.parse::<f64>().ok()))
                })
                .unwrap_or(0.7);
            let max_tokens = context
                .get_node_parameter_at_item("maxTokens", item_index, None)
                .await
                .ok()
                .and_then(|v| {
                    v.as_u64()
                        .or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
                });
            let timeout_ms = context
                .get_node_parameter_at_item("timeout", item_index, None)
                .await
                .ok()
                .and_then(|v| v.as_u64())
                .unwrap_or(60_000);

            let endpoint_base = context
                .get_node_parameter_at_item("baseUrl", item_index, None)
                .await
                .ok()
                .and_then(|v| v.as_str().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| credential_base_url.clone());
            let endpoint = format!("{}/chat/completions", endpoint_base.trim_end_matches('/'));

            let mut messages = Vec::new();
            if !system_prompt.trim().is_empty() {
                messages.push(json!({
                    "role": "system",
                    "content": system_prompt,
                }));
            }
            messages.push(json!({
                "role": "user",
                "content": prompt,
            }));

            let mut request_body = serde_json::Map::new();
            request_body.insert("model".to_string(), json!(model));
            request_body.insert("messages".to_string(), Value::Array(messages));
            request_body.insert("temperature".to_string(), json!(temperature));
            if let Some(max) = max_tokens {
                request_body.insert("max_tokens".to_string(), json!(max));
            }

            let response = self
                .client
                .post(&endpoint)
                .timeout(Duration::from_millis(timeout_ms))
                .header("Authorization", format!("Bearer {}", api_key))
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| BarqError::NodeOperationError {
                    node_name: "OpenAI".to_string(),
                    message: format!(
                        "OpenAI request failed: {}. Check network, base URL, and API key.",
                        e
                    ),
                })?;

            let status = response.status();
            if !status.is_success() {
                let hint = Self::status_hint(status);
                let body = response.text().await.unwrap_or_default();
                let body_msg = if body.trim().is_empty() {
                    String::new()
                } else {
                    format!(" Response: {}", body)
                };
                return Err(BarqError::NodeOperationError {
                    node_name: "OpenAI".to_string(),
                    message: format!(
                        "OpenAI API returned {}. {}{}",
                        status.as_u16(),
                        hint,
                        body_msg
                    )
                    .trim()
                    .to_string(),
                });
            }

            let json_response: Value =
                response
                    .json()
                    .await
                    .map_err(|e| BarqError::NodeOperationError {
                        node_name: "OpenAI".to_string(),
                        message: format!("Failed to parse OpenAI response: {}", e),
                    })?;

            let output = json!({
                "model": model,
                "responseText": Self::extract_response_text(&json_response),
                "usage": json_response.get("usage").cloned().unwrap_or(json!({})),
                "raw": json_response,
            });
            output_items.push(INodeExecutionData::new(IDataObject::from(output)));
        }

        Ok(vec![output_items])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_core::schema::{INode, INodeParameters};
    use barqflow_core::types::{GenericValue, NodeId};
    use mockito::Server;
    use std::collections::HashMap;

    struct MockContext {
        input_data: Vec<INodeExecutionData>,
        params: HashMap<String, GenericValue>,
        creds: HashMap<String, GenericValue>,
        node: INode,
        panic_on_input_probe: bool,
    }

    impl MockContext {
        fn new(input_data: Vec<INodeExecutionData>) -> Self {
            Self {
                input_data,
                params: HashMap::new(),
                creds: HashMap::new(),
                node: INode {
                    id: NodeId("openai-node".into()),
                    name: "OpenAI".into(),
                    r#type: "barqflow-nodes.openai".into(),
                    type_version: 1.0,
                    position: [0.0, 0.0],
                    parameters: INodeParameters(HashMap::new()),
                    credentials: vec![],
                    disabled: false,
                },
                panic_on_input_probe: false,
            }
        }

        fn with_panicking_input_probe(mut self) -> Self {
            self.panic_on_input_probe = true;
            self
        }

        fn add_param(&mut self, key: &str, value: Value) {
            self.params.insert(key.to_string(), value);
        }

        fn add_credential(&mut self, key: &str, value: Value) {
            self.creds.insert(key.to_string(), value);
        }
    }

    #[async_trait]
    impl IExecuteFunctions for MockContext {
        async fn get_node_parameter(
            &self,
            parameter_name: &str,
            fallback_value: Option<GenericValue>,
        ) -> Result<GenericValue, BarqError> {
            if let Some(value) = self.params.get(parameter_name) {
                Ok(value.clone())
            } else if let Some(fallback) = fallback_value {
                Ok(fallback)
            } else {
                Err(BarqError::NodeOperationError {
                    node_name: self.node.name.clone(),
                    message: format!("missing parameter '{}'", parameter_name),
                })
            }
        }

        async fn get_node_parameter_at_item(
            &self,
            parameter_name: &str,
            _item_index: usize,
            fallback_value: Option<GenericValue>,
        ) -> Result<GenericValue, BarqError> {
            self.get_node_parameter(parameter_name, fallback_value)
                .await
        }

        fn get_node(&self) -> &INode {
            &self.node
        }

        async fn get_input_data(
            &self,
            _input_index: usize,
        ) -> Result<Vec<INodeExecutionData>, BarqError> {
            if self.panic_on_input_probe {
                panic!("Cannot block the current thread from within a runtime.");
            }
            Ok(self.input_data.clone())
        }

        async fn get_credentials(
            &self,
            _name: &str,
        ) -> Result<HashMap<String, GenericValue>, BarqError> {
            Ok(self.creds.clone())
        }

        fn log(&self, _message: &str) {}
    }

    #[tokio::test]
    async fn openai_requires_api_key() {
        let mut context =
            MockContext::new(vec![INodeExecutionData::new(IDataObject::from(json!({})))]);
        context.add_param("prompt", json!("hello"));

        let node = OpenAINode::new();
        let err = node.execute(&context).await.unwrap_err();
        match err {
            BarqError::NodeOperationError { message, .. } => {
                assert!(message.contains("Missing OpenAI API key"));
            }
            _ => panic!("expected node operation error"),
        }
    }

    #[tokio::test]
    async fn openai_executes_with_empty_input_once() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":3}}"#)
            .create_async()
            .await;

        let mut context = MockContext::new(vec![]);
        context.add_param("prompt", json!("Say OK"));
        context.add_param("model", json!("gpt-4o-mini"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_credential("apiKey", json!("test-key"));

        let node = OpenAINode::new();
        let result = node.execute(&context).await.unwrap();

        mock.assert_async().await;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(
            result[0][0]
                .json
                .0
                .get("responseText")
                .and_then(|v| v.as_str()),
            Some("ok")
        );
    }

    #[tokio::test]
    async fn openai_requires_prompt() {
        let mut context =
            MockContext::new(vec![INodeExecutionData::new(IDataObject::from(json!({})))]);
        context.add_credential("apiKey", json!("test-key"));

        let node = OpenAINode::new();
        let err = node.execute(&context).await.unwrap_err();
        match err {
            BarqError::NodeOperationError { message, .. } => {
                assert!(message.contains("Prompt cannot be empty"));
            }
            _ => panic!("expected node operation error"),
        }
    }

    #[tokio::test]
    async fn openai_executes_when_input_probe_panics() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"choices":[{"message":{"content":"ok"}}],"usage":{"total_tokens":3}}"#)
            .create_async()
            .await;

        let mut context = MockContext::new(vec![]).with_panicking_input_probe();
        context.add_param("prompt", json!("Say OK"));
        context.add_param("model", json!("gpt-4o-mini"));
        context.add_param("baseUrl", json!(server.url()));
        context.add_credential("apiKey", json!("test-key"));

        let node = OpenAINode::new();
        let result = node.execute(&context).await.unwrap();

        mock.assert_async().await;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(
            result[0][0]
                .json
                .0
                .get("responseText")
                .and_then(|v| v.as_str()),
            Some("ok")
        );
    }
}
