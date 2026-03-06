//! HTTP Request Nodes
//!
//! Implements HTTP request functionality with reqwest integration.

use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::INodeType;
use barqflow_core::types::IDataObject;
use reqwest::Client;

pub struct HttpRequestNode {
    client: Client,
}

impl HttpRequestNode {
    pub fn new() -> Self {
        let client = Client::builder()
            .danger_accept_invalid_certs(false)
            .build()
            .unwrap_or_default();
        Self { client }
    }
}

impl Default for HttpRequestNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for HttpRequestNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "HTTP Request",
            "description": "Send HTTP request to external service"
        }))
    }

    async fn execute(
        &self,
        context: &dyn barqflow_core::traits::IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;
        let mut output_items = Vec::new();

        for (item_index, _item) in input_data.iter().enumerate() {
            let url = context
                .get_node_parameter_at_item("url", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("").to_string())
                .unwrap_or_default();

            if url.is_empty() {
                return Err(BarqError::NodeOperationError {
                    node_name: "HttpRequest".to_string(),
                    message: format!("URL is required for item {}", item_index),
                });
            }

            let method = context
                .get_node_parameter_at_item("method", item_index, None)
                .await
                .map(|v| v.as_str().unwrap_or("GET").to_string())
                .unwrap_or_else(|_| "GET".to_string());

            let body = context
                .get_node_parameter_at_item("body", item_index, None)
                .await
                .ok()
                .and_then(|v| {
                    if v.is_object() || v.is_array() {
                        Some(v.to_string())
                    } else {
                        v.as_str().map(|s| s.to_string())
                    }
                });

            let mut request = match method.to_uppercase().as_str() {
                "POST" => self.client.post(&url),
                "PUT" => self.client.put(&url),
                "PATCH" => self.client.patch(&url),
                "DELETE" => self.client.delete(&url),
                _ => self.client.get(&url),
            };

            // Query parameters
            if let Ok(queries) = context.get_node_parameter_at_item("queryParameters", item_index, None).await {
                if let Some(q_array) = queries.as_array() {
                    for q in q_array {
                        if let (Some(name), Some(value)) = (q.get("name").and_then(|n| n.as_str()), q.get("value").and_then(|v| v.as_str())) {
                            request = request.query(&[(name, value)]);
                        }
                    }
                }
            }

            // Headers
            if let Ok(headers) = context.get_node_parameter_at_item("headers", item_index, None).await {
                if let Some(h_array) = headers.as_array() {
                    for h in h_array {
                        if let (Some(name), Some(value)) = (h.get("name").and_then(|n| n.as_str()), h.get("value").and_then(|v| v.as_str())) {
                            request = request.header(name, value);
                        }
                    }
                }
            }

            if let Some(b) = body {
                request = request.body(b);
            }

            let result = request
                .send()
                .await
                .map_err(|e| BarqError::NodeOperationError {
                    node_name: "HttpRequest".to_string(),
                    message: format!("Item {}: {}", item_index, e),
                })?;

            let status = result.status().as_u16();
            
            // Handle JSON response automatically if possible
            let content_type = result.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("");
            let output = if content_type.contains("application/json") {
                let json_body: serde_json::Value = result.json().await.unwrap_or(serde_json::json!({}));
                serde_json::json!({
                    "status": status,
                    "body": json_body
                })
            } else {
                let body_text = result.text().await.unwrap_or_default();
                serde_json::json!({
                    "status": status,
                    "body": body_text
                })
            };

            output_items.push(INodeExecutionData::new(IDataObject::from(output)));
        }

        Ok(vec![output_items])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_node_creation() {
        let node = HttpRequestNode::new();
        let desc = node.get_description();
        assert_eq!(desc.0.get("name").unwrap(), "HTTP Request");
    }
}
