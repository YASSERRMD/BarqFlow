use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use reqwest::{Client, StatusCode};
use serde_json::{json, Value};
use std::time::Duration;

pub struct OllamaNode {
    client: Client,
}

impl OllamaNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    fn status_hint(status: StatusCode) -> &'static str {
        match status {
            StatusCode::NOT_FOUND => {
                "Endpoint not found (404). Verify the Ollama base URL and API path."
            }
            StatusCode::BAD_REQUEST => "Bad request (400). Verify model and payload.",
            StatusCode::INTERNAL_SERVER_ERROR => {
                "Ollama returned internal error (500). Check server logs."
            }
            _ => "",
        }
    }
}

impl Default for OllamaNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for OllamaNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "Ollama",
            "description": "Interact with a local Ollama instance"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0).await?;
        let run_count = if input_data.is_empty() { 1 } else { input_data.len() };
        let mut output_items = Vec::new();

        for item_index in 0..run_count {
            let base_url = context
                .get_node_parameter_at_item("baseUrl", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("http://localhost:11434").to_string())
                .unwrap_or_else(|_| "http://localhost:11434".to_string());
            let base_url = base_url.trim().trim_end_matches('/').to_string();
            if base_url.is_empty() {
                return Err(BarqError::NodeOperationError {
                    node_name: "Ollama".to_string(),
                    message: "Base URL cannot be empty. Set it to your Ollama host, for example http://localhost:11434."
                        .to_string(),
                });
            }

            let operation = context
                .get_node_parameter_at_item("operation", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("generate").to_string())
                .unwrap_or_else(|_| "generate".to_string());
            let timeout_ms = context
                .get_node_parameter_at_item("timeout", item_index, None)
                .await
                .ok()
                .and_then(|v| v.as_u64())
                .unwrap_or(60_000);

            if operation == "generate" {
                let model = context
                    .get_node_parameter_at_item("model", item_index, None)
                    .await
                    .map(|v| v.as_str().unwrap_or("llama3").to_string())
                    .unwrap_or_else(|_| "llama3".to_string());
                let prompt = context
                    .get_node_parameter_at_item("prompt", item_index, None)
                    .await
                    .map(|v| v.as_str().unwrap_or("").to_string())
                    .unwrap_or_default();
                if prompt.trim().is_empty() {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Ollama".to_string(),
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
                    });

                let mut request_body = serde_json::Map::new();
                request_body.insert("model".to_string(), json!(model));
                request_body.insert("prompt".to_string(), json!(prompt));
                request_body.insert("stream".to_string(), json!(false));
                if !system_prompt.trim().is_empty() {
                    request_body.insert("system".to_string(), json!(system_prompt));
                }
                if let Some(temp) = temperature {
                    request_body.insert(
                        "options".to_string(),
                        json!({
                            "temperature": temp
                        }),
                    );
                }

                let endpoint = format!("{}/api/generate", base_url);
                let response = self
                    .client
                    .post(&endpoint)
                    .timeout(Duration::from_millis(timeout_ms))
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| BarqError::NodeOperationError {
                        node_name: "Ollama".to_string(),
                        message: format!(
                            "Ollama request failed: {}. Ensure Ollama is running and reachable at {}.",
                            e, base_url
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
                        node_name: "Ollama".to_string(),
                        message: format!(
                            "Ollama API returned {}. {}{}",
                            status.as_u16(),
                            hint,
                            body_msg
                        )
                        .trim()
                        .to_string(),
                    });
                }

                let payload: Value = response.json().await.map_err(|e| BarqError::NodeOperationError {
                    node_name: "Ollama".to_string(),
                    message: format!("Failed to parse Ollama response: {}", e),
                })?;
                output_items.push(INodeExecutionData::new(IDataObject::from(json!({
                    "operation": operation,
                    "responseText": payload
                        .get("response")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    "raw": payload,
                }))));
            } else if operation == "listModels" {
                let endpoint = format!("{}/api/tags", base_url);
                let response = self
                    .client
                    .get(&endpoint)
                    .timeout(Duration::from_millis(timeout_ms))
                    .send()
                    .await
                    .map_err(|e| BarqError::NodeOperationError {
                        node_name: "Ollama".to_string(),
                        message: format!(
                            "Ollama request failed: {}. Ensure Ollama is running and reachable at {}.",
                            e, base_url
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
                        node_name: "Ollama".to_string(),
                        message: format!(
                            "Ollama API returned {}. {}{}",
                            status.as_u16(),
                            hint,
                            body_msg
                        )
                        .trim()
                        .to_string(),
                    });
                }

                let payload: Value = response.json().await.map_err(|e| BarqError::NodeOperationError {
                    node_name: "Ollama".to_string(),
                    message: format!("Failed to parse Ollama response: {}", e),
                })?;
                let models: Vec<Value> = payload
                    .get("models")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .filter_map(|model| {
                        model
                            .get("name")
                            .and_then(|v| v.as_str())
                            .map(|name| Value::String(name.to_string()))
                    })
                    .collect();

                output_items.push(INodeExecutionData::new(IDataObject::from(json!({
                    "operation": operation,
                    "models": models,
                    "raw": payload,
                }))));
            } else {
                return Err(BarqError::NodeOperationError {
                    node_name: "Ollama".to_string(),
                    message: format!("Operation '{}' not supported", operation),
                });
            }
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
        node: INode,
    }

    impl MockContext {
        fn new(input_data: Vec<INodeExecutionData>) -> Self {
            Self {
                input_data,
                params: HashMap::new(),
                node: INode {
                    id: NodeId("ollama-node".into()),
                    name: "Ollama".into(),
                    r#type: "barqflow-nodes.ollama".into(),
                    type_version: 1.0,
                    position: [0.0, 0.0],
                    parameters: INodeParameters(HashMap::new()),
                    credentials: vec![],
                    disabled: false,
                },
            }
        }

        fn add_param(&mut self, key: &str, value: Value) {
            self.params.insert(key.to_string(), value);
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
            self.get_node_parameter(parameter_name, fallback_value).await
        }

        fn get_node(&self) -> &INode {
            &self.node
        }

        async fn get_input_data(&self, _input_index: usize) -> Result<Vec<INodeExecutionData>, BarqError> {
            Ok(self.input_data.clone())
        }

        async fn get_credentials(&self, _name: &str) -> Result<HashMap<String, GenericValue>, BarqError> {
            Ok(HashMap::new())
        }

        fn log(&self, _message: &str) {}
    }

    #[tokio::test]
    async fn ollama_generate_requires_prompt() {
        let mut context = MockContext::new(vec![INodeExecutionData::new(IDataObject::from(json!({})))]);
        context.add_param("operation", json!("generate"));

        let node = OllamaNode::new();
        let err = node.execute(&context).await.unwrap_err();
        match err {
            BarqError::NodeOperationError { message, .. } => {
                assert!(message.contains("Prompt cannot be empty"));
            }
            _ => panic!("expected node operation error"),
        }
    }

    #[tokio::test]
    async fn ollama_list_models_works_without_input_items() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("GET", "/api/tags")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"models":[{"name":"llama3.2"},{"name":"qwen2.5"}]}"#)
            .create_async()
            .await;

        let mut context = MockContext::new(vec![]);
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("operation", json!("listModels"));

        let node = OllamaNode::new();
        let result = node.execute(&context).await.unwrap();

        mock.assert_async().await;
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        let models = result[0][0].json.0.get("models").unwrap().as_array().unwrap();
        assert_eq!(models.len(), 2);
    }

    #[tokio::test]
    async fn ollama_generate_executes_with_empty_input_once() {
        let mut server = Server::new_async().await;
        let mock = server
            .mock("POST", "/api/generate")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"response":"hello"}"#)
            .create_async()
            .await;

        let mut context = MockContext::new(vec![]);
        context.add_param("baseUrl", json!(server.url()));
        context.add_param("operation", json!("generate"));
        context.add_param("model", json!("llama3.2"));
        context.add_param("prompt", json!("hello"));

        let node = OllamaNode::new();
        let result = node.execute(&context).await.unwrap();

        mock.assert_async().await;
        assert_eq!(result[0].len(), 1);
        assert_eq!(
            result[0][0]
                .json
                .0
                .get("responseText")
                .and_then(|v| v.as_str()),
            Some("hello")
        );
    }
}
