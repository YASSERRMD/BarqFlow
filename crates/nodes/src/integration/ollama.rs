use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::INodeType;
use barqflow_core::types::IDataObject;
use reqwest::Client;

pub struct OllamaNode {
    client: Client,
}

impl OllamaNode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
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
        context: &dyn barqflow_core::traits::IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;
        let mut output_items = Vec::new();

        for (item_index, _item) in input_data.iter().enumerate() {
            let base_url = context
                .get_node_parameter_at_item("baseUrl", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("http://host.docker.internal:11434").to_string())
                .unwrap_or_else(|_| "http://host.docker.internal:11434".to_string());

            let operation = context
                .get_node_parameter_at_item("operation", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("generate").to_string())
                .unwrap_or_else(|_| "generate".to_string());

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

                if prompt.is_empty() {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Ollama".to_string(),
                        message: "Prompt cannot be empty".to_string(),
                    });
                }

                let request_body = serde_json::json!({
                    "model": model,
                    "prompt": prompt,
                    "stream": false
                });

                let target_url = format!("{}/api/generate", base_url.trim_end_matches('/'));

                let result = self.client.post(&target_url)
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| BarqError::NodeOperationError {
                        node_name: "Ollama".to_string(),
                        message: format!("API request failed: {}", e),
                    })?;

                let status = result.status();
                if !status.is_success() {
                    let err_text = result.text().await.unwrap_or_default();
                    return Err(BarqError::NodeOperationError {
                        node_name: "Ollama".to_string(),
                        message: format!("API returned error {}: {}", status, err_text),
                    });
                }

                let json_response: serde_json::Value = result.json().await.map_err(|e| BarqError::NodeOperationError {
                    node_name: "Ollama".to_string(),
                    message: format!("Failed to parse response: {}", e),
                })?;

                output_items.push(INodeExecutionData::new(IDataObject::from(json_response)));
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
