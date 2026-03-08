use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::INodeType;
use barqflow_core::types::IDataObject;
use reqwest::Client;

pub struct OpenAINode {
    client: Client,
}

impl OpenAINode {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
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
        context: &dyn barqflow_core::traits::IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0).await?;
        let mut output_items = Vec::new();

        // Get credentials
        let creds = context.get_credentials("openAiApi").await?;
        let api_key = creds.get("apiKey").and_then(|v| v.as_str()).unwrap_or("");

        if api_key.is_empty() {
            return Err(BarqError::NodeOperationError {
                node_name: "OpenAI".to_string(),
                message: "Missing 'apiKey' from credentials".to_string(),
            });
        }

        for (item_index, _item) in input_data.iter().enumerate() {
            let operation = context
                .get_node_parameter_at_item("operation", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("chatCompletion").to_string())
                .unwrap_or_else(|_| "chatCompletion".to_string());

            if operation == "chatCompletion" {
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

                if prompt.is_empty() {
                    return Err(BarqError::NodeOperationError {
                        node_name: "OpenAI".to_string(),
                        message: "Prompt cannot be empty".to_string(),
                    });
                }

                let request_body = serde_json::json!({
                    "model": model,
                    "messages": [
                        {
                            "role": "user",
                            "content": prompt
                        }
                    ]
                });

                let result = self.client.post("https://api.openai.com/v1/chat/completions")
                    .header("Authorization", format!("Bearer {}", api_key))
                    .header("Content-Type", "application/json")
                    .json(&request_body)
                    .send()
                    .await
                    .map_err(|e| BarqError::NodeOperationError {
                        node_name: "OpenAI".to_string(),
                        message: format!("API request failed: {}", e),
                    })?;

                let status = result.status();
                if !status.is_success() {
                    let err_text = result.text().await.unwrap_or_default();
                    return Err(BarqError::NodeOperationError {
                        node_name: "OpenAI".to_string(),
                        message: format!("API returned error {}: {}", status, err_text),
                    });
                }

                let json_response: serde_json::Value = result.json().await.map_err(|e| BarqError::NodeOperationError {
                    node_name: "OpenAI".to_string(),
                    message: format!("Failed to parse response: {}", e),
                })?;

                output_items.push(INodeExecutionData::new(IDataObject::from(json_response)));
            } else {
                return Err(BarqError::NodeOperationError {
                    node_name: "OpenAI".to_string(),
                    message: format!("Operation '{}' not supported", operation),
                });
            }
        }

        Ok(vec![output_items])
    }
}
