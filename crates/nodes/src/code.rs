use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use rhai::{Dynamic, Engine, Map, Scope};
use serde_json::{json, Value};

pub struct CodeNode;

impl CodeNode {
    pub fn new() -> Self {
        Self
    }

    fn wrap_items(items: &[Value]) -> Result<rhai::Array, BarqError> {
        let mut items_array = rhai::Array::new();

        for item in items {
            match rhai::serde::to_dynamic(item.clone()) {
                Ok(dynamic_json) => {
                    let mut wrapper = Map::new();
                    wrapper.insert("json".into(), dynamic_json);
                    items_array.push(wrapper.into());
                }
                Err(error) => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Code".into(),
                        message: format!("Failed to serialize input for Rhai: {}", error),
                    });
                }
            }
        }

        Ok(items_array)
    }

    fn dynamic_map_to_execution_data(map: Map) -> Result<INodeExecutionData, BarqError> {
        let target_map = match map.get("json") {
            Some(json_field) if json_field.is_map() => json_field.clone().cast::<Map>(),
            _ => map,
        };

        let payload = rhai::serde::from_dynamic::<Value>(&target_map.into()).map_err(|error| {
            BarqError::NodeOperationError {
                node_name: "Code".into(),
                message: format!("Failed to parse return item from Rhai Sandbox: {}", error),
            }
        })?;

        Ok(INodeExecutionData::new(IDataObject::from(payload)))
    }

    fn dynamic_to_execution_items(result: Dynamic) -> Result<Vec<INodeExecutionData>, BarqError> {
        if result.is_unit() {
            return Ok(Vec::new());
        }

        if result.is_map() {
            return Ok(vec![Self::dynamic_map_to_execution_data(result.cast::<Map>())?]);
        }

        if result.is_array() {
            let arr = result.into_array().unwrap_or_default();
            let mut output_items = Vec::new();

            for item in arr {
                if item.is_unit() {
                    continue;
                }

                if item.is_map() {
                    output_items.push(Self::dynamic_map_to_execution_data(item.cast::<Map>())?);
                    continue;
                }

                return Err(BarqError::NodeOperationError {
                    node_name: "Code".into(),
                    message:
                        "Rhai output array must contain structured objects or { json: ... } wrappers."
                            .into(),
                });
            }

            return Ok(output_items);
        }

        Err(BarqError::NodeOperationError {
            node_name: "Code".into(),
            message: "Rhai script must return an object or an array of objects.".into(),
        })
    }
}

impl Default for CodeNode {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl INodeType for CodeNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "Code",
            "description": "Execute custom Rhai scripts securely",
            "displayName": "Code",
            "properties": [
                {
                    "name": "mode",
                    "displayName": "Mode",
                    "type": "options",
                    "default": "runOnceForAllItems"
                },
                {
                    "name": "language",
                    "displayName": "Language",
                    "type": "options",
                    "default": "javascript"
                },
                {
                    "name": "jsCode",
                    "displayName": "JavaScript Code",
                    "type": "string",
                    "default": "items"
                },
                {
                    "name": "pythonCode",
                    "displayName": "Python Code",
                    "type": "string",
                    "default": "items"
                },
                {
                    "name": "code",
                    "displayName": "Code",
                    "type": "string",
                    "default": "\n// Modify the item data\nfor item in items {\n    item.data.myCustomField = 1;\n}\nreturn items;\n"
                }
            ]
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let language = context
            .get_node_parameter("language", None)
            .await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_lowercase()))
            .unwrap_or_else(|| "javascript".to_string());

        if language == "python" {
            return Err(BarqError::NodeOperationError {
                node_name: "Code".into(),
                message:
                    "Python mode is not supported yet in BarqFlow Code node. Use JavaScript mode and fill 'jsCode'."
                        .into(),
            });
        }

        let js_code = context
            .get_node_parameter("jsCode", None)
            .await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        let legacy_code = context
            .get_node_parameter("code", None)
            .await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()));
        let code = js_code
            .or(legacy_code)
            .unwrap_or_else(|| "return items;".to_string());

        if code.trim().is_empty() {
            return Err(BarqError::NodeOperationError {
                node_name: "Code".into(),
                message: "Code cannot be empty.".into(),
            });
        }

        let mode = context
            .get_node_parameter("mode", None)
            .await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "runOnceForAllItems".to_string());

        // We receive the previous items
        let previous_items = context.get_input_data(0).await.unwrap_or_default();
        let previous_json_items: Vec<Value> =
            previous_items.iter().map(|item| Value::Object(item.json.0.clone())).collect();

        // Ensure Rhai execution is isolated (no OS access, etc.)
        // By default, `Engine::new()` avoids exposing OS boundaries unless manually registered
        let mut engine = Engine::new();

        // Limit maximum operations to prevent infinite loops
        engine.set_max_operations(100_000);

        let output_items = if mode == "runOnceForEachItem" {
            let source_items = if previous_json_items.is_empty() {
                vec![json!({})]
            } else {
                previous_json_items.clone()
            };

            let mut collected = Vec::new();
            for item in source_items {
                let items_array = Self::wrap_items(std::slice::from_ref(&item))?;
                let first_item = items_array
                    .first()
                    .cloned()
                    .unwrap_or_else(|| Dynamic::from_map(Map::new()));

                let mut scope = Scope::new();
                scope.push("items", Dynamic::from_array(items_array));
                scope.push_dynamic("item", first_item);

                let result = engine.eval_with_scope::<Dynamic>(&mut scope, &code).map_err(|error| {
                    BarqError::NodeOperationError {
                        node_name: "Code".into(),
                        message: format!("Rhai execution error: {}", error),
                    }
                })?;

                collected.extend(Self::dynamic_to_execution_items(result)?);
            }

            collected
        } else {
            let items_array = Self::wrap_items(&previous_json_items)?;
            let mut scope = Scope::new();
            scope.push("items", Dynamic::from_array(items_array));

            let result = engine.eval_with_scope::<Dynamic>(&mut scope, &code).map_err(|error| {
                BarqError::NodeOperationError {
                    node_name: "Code".into(),
                    message: format!("Rhai execution error: {}", error),
                }
            })?;

            Self::dynamic_to_execution_items(result)?
        };

        Ok(vec![output_items])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_core::schema::INode;
    use barqflow_core::types::GenericValue;

    // Mock Context for testing
    struct MockCodeContext {
        js_code: String,
        language: String,
        mode: String,
        inputs: Vec<INodeExecutionData>,
    }

    #[async_trait]
    impl IExecuteFunctions for MockCodeContext {
        async fn get_node_parameter(
            &self,
            parameter_name: &str,
            _fallback: Option<GenericValue>,
        ) -> Result<GenericValue, BarqError> {
            match parameter_name {
                "language" => Ok(serde_json::json!(self.language)),
                "mode" => Ok(serde_json::json!(self.mode)),
                "jsCode" | "code" => Ok(serde_json::json!(self.js_code)),
                "pythonCode" => Ok(serde_json::json!("items")),
                _ => Err(BarqError::NodeOperationError {
                    node_name: "".into(),
                    message: "Param not found".into(),
                }),
            }
        }

        fn get_node(&self) -> &INode {
            unimplemented!()
        }

        async fn get_node_parameter_at_item(
            &self,
            parameter_name: &str,
            _item_index: usize,
            fallback_value: Option<barqflow_core::types::GenericValue>,
        ) -> Result<barqflow_core::types::GenericValue, BarqError> {
            self.get_node_parameter(parameter_name, fallback_value)
                .await
        }

        async fn get_credentials(
            &self,
            _name: &str,
        ) -> Result<std::collections::HashMap<String, barqflow_core::types::GenericValue>, BarqError>
        {
            Ok(std::collections::HashMap::new())
        }

        async fn get_input_data(
            &self,
            _input_index: usize,
        ) -> Result<Vec<INodeExecutionData>, BarqError> {
            Ok(self.inputs.clone())
        }
        fn log(&self, _message: &str) {}
    }

    #[tokio::test]
    async fn test_rhai_sandbox_execution() {
        let node = CodeNode::new();

        let ctx = MockCodeContext {
            js_code: r#"
                items[0].json.hello = "sandbox";
                
                let new_item = #{ json: #{ added: true } };
                items.push(new_item);
                
                items
            "#
            .to_string(),
            language: "javascript".to_string(),
            mode: "runOnceForAllItems".to_string(),
            inputs: vec![INodeExecutionData::new(IDataObject::from(
                json!({ "test": true }),
            ))],
        };

        let result = node.execute(&ctx).await.unwrap();
        let branch = &result[0];
        assert_eq!(branch.len(), 2);

        let first_json = &branch[0].json.0;
        assert_eq!(first_json.get("test").unwrap().as_bool().unwrap(), true);
        assert_eq!(
            first_json.get("hello").unwrap().as_str().unwrap(),
            "sandbox"
        );

        let second_json = &branch[1].json.0;
        assert_eq!(second_json.get("added").unwrap().as_bool().unwrap(), true);
    }

    #[tokio::test]
    async fn test_sandbox_infinite_loop_prevention() {
        let node = CodeNode::new();

        // This simulates a malicious loop trying to freeze the thread
        let ctx = MockCodeContext {
            js_code: r#"
                let i = 0;
                loop {
                    i += 1;
                }
            "#
            .to_string(),
            language: "javascript".to_string(),
            mode: "runOnceForAllItems".to_string(),
            inputs: vec![],
        };

        let err = node.execute(&ctx).await.unwrap_err();
        match err {
            BarqError::NodeOperationError { message, .. } => {
                assert!(message.contains("Rhai execution error"));
                assert!(message.contains("Too many operations"), "{}", message);
            }
            _ => panic!("Expected NodeOperationError"),
        }
    }

    #[tokio::test]
    async fn test_sandbox_os_prevention() {
        let node = CodeNode::new();

        // Attempt to execute an OS command or access File system through typical Rhai/Rust bindings when not registered
        let ctx = MockCodeContext {
            js_code: r#"
                let f = open("test.txt");
            "#
            .to_string(),
            language: "javascript".to_string(),
            mode: "runOnceForAllItems".to_string(),
            inputs: vec![],
        };

        let err = node.execute(&ctx).await.unwrap_err();
        match err {
            BarqError::NodeOperationError { message, .. } => {
                assert!(message.contains("Rhai execution error"));
                // Must complain about 'open' being an unknown function since no OS package is mounted
                assert!(message.contains("Function not found: open"));
            }
            _ => panic!("Expected NodeOperationError"),
        }
    }

    #[tokio::test]
    async fn test_python_mode_returns_clear_error() {
        let node = CodeNode::new();

        let ctx = MockCodeContext {
            js_code: "items".to_string(),
            language: "python".to_string(),
            mode: "runOnceForAllItems".to_string(),
            inputs: vec![],
        };

        let err = node.execute(&ctx).await.unwrap_err();
        match err {
            BarqError::NodeOperationError { message, .. } => {
                assert!(message.contains("Python mode is not supported"));
            }
            _ => panic!("Expected NodeOperationError"),
        }
    }

    #[tokio::test]
    async fn test_run_once_for_each_item_mode_returns_one_item_per_input() {
        let node = CodeNode::new();

        let ctx = MockCodeContext {
            js_code: r#"
                item.json.processed = true;
                item
            "#
            .to_string(),
            language: "javascript".to_string(),
            mode: "runOnceForEachItem".to_string(),
            inputs: vec![
                INodeExecutionData::new(IDataObject::from(json!({ "id": 1 }))),
                INodeExecutionData::new(IDataObject::from(json!({ "id": 2 }))),
            ],
        };

        let result = node.execute(&ctx).await.unwrap();
        assert_eq!(result[0].len(), 2);
        assert_eq!(result[0][0].json.0["processed"], true);
        assert_eq!(result[0][1].json.0["processed"], true);
    }
}
