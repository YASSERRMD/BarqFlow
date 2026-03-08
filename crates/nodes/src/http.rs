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
        let input_data = context.get_input_data(0).await?;
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
            let content_type = result.headers().get("content-type").and_then(|v| v.to_str().ok()).unwrap_or("").to_string();
            
            // Check if response should be treated as binary based on parameters or auto-detect
            let response_format = context
                .get_node_parameter("responseFormat", None)
                .await
                .map(|v| v.as_str().unwrap_or("autodetect").to_string())
                .unwrap_or_else(|_| "autodetect".to_string());

            let mut is_binary = response_format == "file";
            if response_format == "autodetect" {
                if !content_type.starts_with("application/json") && !content_type.starts_with("text/") {
                    is_binary = true;
                }
            }

            if is_binary {
                use base64::{Engine as _, engine::general_purpose::STANDARD};
                let bytes = result.bytes().await.unwrap_or_default();
                let b64 = STANDARD.encode(&bytes);
                
                use barqflow_core::types::{IBinaryData, BinaryDataContent};
                let bin_data = IBinaryData {
                    content: BinaryDataContent::Memory { data: b64 },
                    mime_type: content_type.clone(),
                    file_type: None,
                    file_name: None,
                    directory: None,
                    file_extension: None,
                    file_size: Some(bytes.len().to_string()),
                };
                
                let output = serde_json::json!({
                    "status": status,
                    "body": "[Binary Data]"
                });
                
                let execution_data = INodeExecutionData::new(IDataObject::from(output))
                    .with_binary("data".to_string(), bin_data);
                
                output_items.push(execution_data);
            } else {
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
        }

        Ok(vec![output_items])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use barqflow_core::schema::{INode, INodeParameters};
    use barqflow_core::types::{GenericValue, NodeId};
    use async_trait::async_trait;
    use std::sync::Arc;
    use mockito::Server;

    struct MockContext {
        input_data: Vec<Vec<INodeExecutionData>>,
        params: std::collections::HashMap<String, GenericValue>,
        node: INode,
    }

    impl MockContext {
        fn new() -> Self {
            let input = vec![INodeExecutionData::new(IDataObject::from(serde_json::json!({})))];
            Self {
                input_data: vec![input],
                params: std::collections::HashMap::new(),
                node: INode {
                    id: NodeId("test_http".into()),
                    name: "HTTP Request".into(),
                    r#type: "http".into(),
                    type_version: 1.0,
                    position: [0.0, 0.0],
                    parameters: INodeParameters(std::collections::HashMap::new()),
                    disabled: false,
                },
            }
        }
        
        fn add_param(&mut self, key: &str, value: serde_json::Value) {
            self.params.insert(key.to_string(), value);
        }
    }

    #[async_trait]
    impl barqflow_core::traits::IExecuteFunctions for MockContext {
        async fn get_node_parameter(
            &self,
            parameter_name: &str,
            fallback_value: Option<GenericValue>,
        ) -> Result<GenericValue, BarqError> {
            if let Some(val) = self.params.get(parameter_name) {
                Ok(val.clone())
            } else if let Some(fallback) = fallback_value {
                Ok(fallback)
            } else {
                Err(BarqError::NodeOperationError {
                    node_name: self.node.name.clone(),
                    message: format!("Parameter '{}' not found", parameter_name),
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

        async fn get_input_data(&self, input_index: usize) -> Result<Vec<INodeExecutionData>, BarqError> {
            self.input_data
                .get(input_index)
                .cloned()
                .ok_or(BarqError::NodeOperationError {
                    node_name: self.node.name.clone(),
                    message: format!("No input data at index {}", input_index),
                })
        }

        async fn get_credentials(&self, _name: &str) -> Result<std::collections::HashMap<String, GenericValue>, BarqError> {
            Ok(std::collections::HashMap::new())
        }

        fn log(&self, _message: &str) {}
    }

    #[tokio::test]
    async fn test_http_node_json_request() {
        let mut server = Server::new_async().await;
        let mock = server.mock("POST", "/data")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"success": true}"#)
            .create_async().await;

        let url = format!("{}/data", server.url());
        
        let mut context = MockContext::new();
        context.add_param("url", serde_json::json!(url));
        context.add_param("method", serde_json::json!("POST"));
        context.add_param("body", serde_json::json!({"test": "payload"}));
        
        let node = HttpRequestNode::new();
        let result = node.execute(&context).await.unwrap();
        
        mock.assert_async().await;
        
        assert_eq!(result.len(), 1);
        let output = &result[0][0].json.0;
        assert_eq!(output.get("status").unwrap().as_u64().unwrap(), 200);
        let body = output.get("body").unwrap();
        assert_eq!(body.get("success").unwrap().as_bool().unwrap(), true);
    }
    
    #[tokio::test]
    async fn test_http_node_binary_request() {
        let mut server = Server::new_async().await;
        let mock = server.mock("GET", "/image.png")
            .with_status(200)
            .with_header("content-type", "image/png")
            .with_body(vec![137, 80, 78, 71, 13, 10, 26, 10]) // PNG signature
            .create_async().await;

        let url = format!("{}/image.png", server.url());
        
        let mut context = MockContext::new();
        context.add_param("url", serde_json::json!(url));
        context.add_param("method", serde_json::json!("GET"));
        context.add_param("responseFormat", serde_json::json!("autodetect"));
        
        let node = HttpRequestNode::new();
        let result = node.execute(&context).await.unwrap();
        
        mock.assert_async().await;
        
        assert_eq!(result.len(), 1);
        let exec_data = &result[0][0];
        
        // Assert we got a binary payload map
        let binary_map = exec_data.binary.as_ref().unwrap();
        let bin_payload = binary_map.get("data").unwrap();
        
        assert_eq!(bin_payload.mime_type, "image/png");
        
        use barqflow_core::types::BinaryDataContent;
        match &bin_payload.content {
            BinaryDataContent::Memory { data } => {
                use base64::{Engine as _, engine::general_purpose::STANDARD};
                let bytes = STANDARD.decode(data).unwrap();
                assert_eq!(bytes, vec![137, 80, 78, 71, 13, 10, 26, 10]);
            },
            _ => panic!("Expected Memory binary content"),
        }
    }
}
