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
        let code_val = context.get_node_parameter("code", None).await?;
        let code = code_val.as_str().unwrap_or("return items;").to_string();

        // Ensure Rhai execution is isolated (no OS access, etc.)
        // By default, `Engine::new()` avoids exposing OS boundaries unless manually registered
        let mut engine = Engine::new();

        // Limit maximum operations to prevent infinite loops
        engine.set_max_operations(100_000);

        // We receive the previous items
        let previous_items = match context.get_input_data(0) {
            Ok(data) => data.clone(),
            Err(_) => vec![],
        };

        // Convert the input list to Rhai format
        let mut items_array = rhai::Array::new();
        for item in previous_items {
            match rhai::serde::to_dynamic(item.json.0.clone()) {
                Ok(dyn_map) => {
                    let mut wrapper = Map::new();
                    wrapper.insert("json".into(), dyn_map); // Typical N8N style `items[0].json` wrapper
                    items_array.push(wrapper.into());
                }
                Err(e) => {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Code".into(),
                        message: format!("Failed to serialize input for Rhai: {}", e),
                    });
                }
            }
        }

        let mut scope = Scope::new();
        scope.push("items", Dynamic::from_array(items_array));

        let result: Dynamic = match engine.eval_with_scope::<Dynamic>(&mut scope, &code) {
            Ok(v) => v,
            Err(e) => {
                return Err(BarqError::NodeOperationError {
                    node_name: "Code".into(),
                    message: format!("Rhai execution error: {}", e),
                });
            }
        };

        let mut output_items = Vec::new();

        // Ensure structured return mapping: force result back to Vec<INodeExecutionData>
        if result.is_array() {
            let arr = result.into_array().unwrap();
            for item in arr {
                if item.is_map() {
                    let map = item.cast::<Map>();

                    // Allow unwrapping the `.json` structure N8N style, or handle raw map if provided directly.
                    let target_map = match map.get("json") {
                        Some(json_field) => {
                            if json_field.is_map() {
                                json_field.clone().cast::<Map>()
                            } else {
                                map.clone()
                            }
                        }
                        None => map.clone(),
                    };

                    match rhai::serde::from_dynamic::<Value>(&target_map.into()) {
                        Ok(val) => {
                            output_items.push(INodeExecutionData::new(IDataObject::from(val)));
                        }
                        Err(e) => {
                            return Err(BarqError::NodeOperationError {
                                node_name: "Code".into(),
                                message: format!(
                                    "Failed to parse return item from Rhai Sandbox: {}",
                                    e
                                ),
                            });
                        }
                    }
                } else if item.is_unit() {
                    continue;
                } else {
                    return Err(BarqError::NodeOperationError {
                        node_name: "Code".into(),
                        message: "Rhai output array must contain valid structured objects (maps)"
                            .into(),
                    });
                }
            }
        } else {
            return Err(BarqError::NodeOperationError {
                node_name: "Code".into(),
                message: "Rhai script must return an Array of objects".into(),
            });
        }

        Ok(vec![output_items])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_core::types::GenericValue;
    use barqflow_core::schema::INode;
    use std::collections::HashMap;

    // Mock Context for testing
    struct MockCodeContext {
        code: String,
        inputs: Vec<INodeExecutionData>,
    }

    #[async_trait]
    impl IExecuteFunctions for MockCodeContext {
        async fn get_node_parameter(
            &self,
            parameter_name: &str,
            _fallback: Option<GenericValue>,
        ) -> Result<GenericValue, BarqError> {
            if parameter_name == "code" {
                Ok(serde_json::json!(self.code))
            } else {
                Err(BarqError::NodeOperationError {
                    node_name: "".into(),
                    message: "Param not found".into(),
                })
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
            self.get_node_parameter(parameter_name, fallback_value).await
        }

        async fn get_credentials(
            &self,
            _name: &str,
        ) -> Result<std::collections::HashMap<String, barqflow_core::types::GenericValue>, BarqError> {
            Ok(std::collections::HashMap::new())
        }

        fn get_input_data(
            &self,
            _input_index: usize,
        ) -> Result<&Vec<INodeExecutionData>, BarqError> {
            Ok(&self.inputs)
        }
        fn log(&self, _message: &str) {}
    }

    #[tokio::test]
    async fn test_rhai_sandbox_execution() {
        let node = CodeNode::new();

        let ctx = MockCodeContext {
            code: r#"
                items[0].json.hello = "sandbox";
                
                let new_item = #{ json: #{ added: true } };
                items.push(new_item);
                
                items
            "#
            .to_string(),
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
            code: r#"
                let i = 0;
                loop {
                    i += 1;
                }
            "#
            .to_string(),
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
}
