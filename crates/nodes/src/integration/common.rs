use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::IExecuteFunctions;
use barqflow_core::types::IDataObject;
use reqwest::{Client, Method};
use serde_json::{json, Value};
use std::time::Duration;

pub(crate) struct PreparedRequest {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub query: Vec<(String, String)>,
    pub body: Option<Value>,
    pub auth_token: Option<String>,
    pub timeout_ms: u64,
}

pub(crate) struct IntegrationResponse {
    pub status: u16,
    pub headers: Value,
    pub body: Value,
    pub raw_text: String,
}

pub(crate) async fn run_count(context: &dyn IExecuteFunctions) -> usize {
    match context.get_input_data(0).await {
        Ok(input_data) if !input_data.is_empty() => input_data.len(),
        _ => 1,
    }
}

pub(crate) async fn get_optional_param(
    context: &dyn IExecuteFunctions,
    parameter_name: &str,
    item_index: usize,
) -> Option<Value> {
    context
        .get_node_parameter_at_item(parameter_name, item_index, None)
        .await
        .ok()
}

pub(crate) async fn get_string_param(
    context: &dyn IExecuteFunctions,
    parameter_name: &str,
    item_index: usize,
    default: &str,
) -> String {
    get_optional_param(context, parameter_name, item_index)
        .await
        .and_then(value_to_string)
        .unwrap_or_else(|| default.to_string())
}

pub(crate) async fn get_optional_string_param(
    context: &dyn IExecuteFunctions,
    parameter_name: &str,
    item_index: usize,
) -> Option<String> {
    get_optional_param(context, parameter_name, item_index)
        .await
        .and_then(value_to_string)
}

pub(crate) async fn get_u64_param(
    context: &dyn IExecuteFunctions,
    parameter_name: &str,
    item_index: usize,
    default: u64,
) -> u64 {
    get_optional_param(context, parameter_name, item_index)
        .await
        .and_then(|v| {
            v.as_u64()
                .or_else(|| v.as_str().and_then(|s| s.trim().parse::<u64>().ok()))
        })
        .unwrap_or(default)
}

pub(crate) fn parse_kv_pairs(value: &Value) -> Vec<(String, String)> {
    if value.is_null() {
        return Vec::new();
    }

    if let Some(obj) = value.as_object() {
        return obj
            .iter()
            .map(|(k, v)| {
                let val = v
                    .as_str()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| v.to_string());
                (k.clone(), val)
            })
            .collect();
    }

    if let Some(arr) = value.as_array() {
        return arr
            .iter()
            .filter_map(|entry| {
                let name = entry.get("name").and_then(|n| n.as_str())?;
                let value = entry
                    .get("value")
                    .map(|v| {
                        v.as_str()
                            .map(|s| s.to_string())
                            .unwrap_or_else(|| v.to_string())
                    })
                    .unwrap_or_default();
                Some((name.to_string(), value))
            })
            .collect();
    }

    if let Some(raw) = value.as_str() {
        if let Ok(parsed) = serde_json::from_str::<Value>(raw) {
            return parse_kv_pairs(&parsed);
        }
    }

    Vec::new()
}

pub(crate) fn parse_body(value: Option<Value>) -> Option<Value> {
    let body_value = value?;

    if body_value.is_null() {
        return None;
    }

    if body_value.is_object() || body_value.is_array() {
        return Some(body_value);
    }

    if let Some(raw) = body_value.as_str() {
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            return None;
        }
        if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
            return Some(parsed);
        }
        return Some(Value::String(trimmed.to_string()));
    }

    Some(body_value)
}

pub(crate) fn ensure_required_string(
    node_name: &str,
    field_name: &str,
    value: Option<String>,
    hint: &str,
) -> Result<String, BarqError> {
    let normalized = value.unwrap_or_default().trim().to_string();
    if normalized.is_empty() {
        return Err(BarqError::NodeOperationError {
            node_name: node_name.to_string(),
            message: format!("Missing {}. {}", field_name, hint),
        });
    }

    Ok(normalized)
}

pub(crate) fn require_auth_token(
    node_name: &str,
    token: Option<String>,
) -> Result<String, BarqError> {
    ensure_required_string(
        node_name,
        "Auth Token",
        token,
        "Add a valid API token in the node configuration.",
    )
}

pub(crate) fn build_url(base_url: &str, resource_path: &str) -> String {
    let path = resource_path.trim();
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_string();
    }

    let base = base_url.trim().trim_end_matches('/');
    let relative = path.trim_start_matches('/');

    if base.is_empty() {
        format!("/{}", relative)
    } else if relative.is_empty() {
        base.to_string()
    } else {
        format!("{}/{}", base, relative)
    }
}

pub(crate) async fn execute_prepared_request(
    client: &Client,
    node_name: &str,
    request: PreparedRequest,
) -> Result<IntegrationResponse, BarqError> {
    let method =
        Method::from_bytes(request.method.trim().to_uppercase().as_bytes()).unwrap_or(Method::GET);

    let mut req = client
        .request(method, &request.url)
        .timeout(Duration::from_millis(request.timeout_ms));

    if let Some(token) = request.auth_token {
        req = req.bearer_auth(token);
    }

    if !request.query.is_empty() {
        req = req.query(&request.query);
    }

    for (name, value) in request.headers {
        if !name.trim().is_empty() {
            req = req.header(name, value);
        }
    }

    if let Some(body_value) = request.body {
        if body_value.is_object() || body_value.is_array() {
            req = req.json(&body_value);
        } else if let Some(raw) = body_value.as_str() {
            req = req.body(raw.to_string());
        } else {
            req = req.body(body_value.to_string());
        }
    }

    let response = req
        .send()
        .await
        .map_err(|e| BarqError::NodeOperationError {
            node_name: node_name.to_string(),
            message: format!(
                "Request failed for {} {}: {}",
                request.method.to_uppercase(),
                request.url,
                e
            ),
        })?;

    let status = response.status();
    let headers = response.headers().clone();
    let content_type = headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();
    let raw_text = response.text().await.unwrap_or_default();

    if !status.is_success() {
        let response_preview = if raw_text.trim().is_empty() {
            "(empty response)".to_string()
        } else {
            truncate_text(&raw_text, 600)
        };
        return Err(BarqError::NodeOperationError {
            node_name: node_name.to_string(),
            message: format!(
                "API returned status {} for request. Response: {}",
                status.as_u16(),
                response_preview
            ),
        });
    }

    let mut header_obj = serde_json::Map::new();
    for (name, value) in headers.iter() {
        header_obj.insert(
            name.as_str().to_string(),
            Value::String(value.to_str().unwrap_or_default().to_string()),
        );
    }

    let body = parse_response_body(&content_type, &raw_text);

    Ok(IntegrationResponse {
        status: status.as_u16(),
        headers: Value::Object(header_obj),
        body,
        raw_text,
    })
}

pub(crate) fn build_standard_output(
    operation: &str,
    response: IntegrationResponse,
) -> INodeExecutionData {
    INodeExecutionData::new(IDataObject::from(json!({
        "operation": operation,
        "status": response.status,
        "headers": response.headers,
        "body": response.body,
        "rawText": response.raw_text,
    })))
}

fn parse_response_body(content_type: &str, raw_text: &str) -> Value {
    if raw_text.trim().is_empty() {
        return Value::Null;
    }

    if content_type.contains("application/json") || content_type.contains("+json") {
        return serde_json::from_str(raw_text)
            .unwrap_or_else(|_| Value::String(raw_text.to_string()));
    }

    Value::String(raw_text.to_string())
}

fn value_to_string(value: Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        let trimmed = s.trim();
        if trimmed.is_empty() {
            return None;
        }
        return Some(trimmed.to_string());
    }

    if value.is_number() || value.is_boolean() {
        return Some(value.to_string());
    }

    None
}

fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    let truncated: String = text.chars().take(max_chars).collect();
    format!("{}...", truncated)
}

#[cfg(test)]
pub(crate) mod test_utils {
    use super::*;
    use async_trait::async_trait;
    use barqflow_core::schema::{INode, INodeParameters};
    use barqflow_core::types::{GenericValue, NodeId};
    use std::collections::HashMap;

    pub(crate) struct MockContext {
        pub(crate) input_data: Vec<INodeExecutionData>,
        pub(crate) params: HashMap<String, GenericValue>,
        pub(crate) node: INode,
    }

    impl MockContext {
        pub(crate) fn new(node_name: &str, node_type: &str) -> Self {
            Self {
                input_data: vec![INodeExecutionData::new(IDataObject::from(json!({})))],
                params: HashMap::new(),
                node: INode {
                    id: NodeId(format!("{}-node", node_name.to_lowercase())),
                    name: node_name.to_string(),
                    r#type: node_type.to_string(),
                    type_version: 1.0,
                    position: [0.0, 0.0],
                    parameters: INodeParameters(HashMap::new()),
                    credentials: vec![],
                    disabled: false,
                },
            }
        }

        pub(crate) fn add_param(&mut self, key: &str, value: Value) {
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
            Ok(self.input_data.clone())
        }

        async fn get_credentials(
            &self,
            _name: &str,
        ) -> Result<HashMap<String, GenericValue>, BarqError> {
            Ok(HashMap::new())
        }

        fn log(&self, _message: &str) {}
    }
}
