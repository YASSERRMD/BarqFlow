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

    async fn execute(&self, context: &dyn barqflow_core::traits::IExecuteFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let url = context.get_node_parameter("url", None)
            .await
            .map(|v| v.as_str().unwrap_or("").to_string())
            .unwrap_or_default();

        if url.is_empty() {
            return Err(BarqError::NodeOperationError {
                node_name: "HttpRequest".to_string(),
                message: "URL is required".to_string(),
            });
        }

        let method = context.get_node_parameter("method", None)
            .await
            .map(|v| v.as_str().unwrap_or("GET").to_string())
            .unwrap_or_else(|_| "GET".to_string());

        let body = context.get_node_parameter("body", None)
            .await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()));

        let request = match method.to_uppercase().as_str() {
            "POST" => self.client.post(&url),
            "PUT" => self.client.put(&url),
            "PATCH" => self.client.patch(&url),
            "DELETE" => self.client.delete(&url),
            _ => self.client.get(&url),
        };

        let response = if let Some(b) = body {
            request.body(b)
        } else {
            request
        };

        let result = response.send().await.map_err(|e| BarqError::NodeOperationError {
            node_name: "HttpRequest".to_string(),
            message: e.to_string(),
        })?;

        let status = result.status().as_u16();
        let body_text = result.text().await.unwrap_or_default();

        let output = serde_json::json!({
            "status": status,
            "body": body_text
        });

        let output_item = INodeExecutionData::new(IDataObject::from(output));
        Ok(vec![vec![output_item]])
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
