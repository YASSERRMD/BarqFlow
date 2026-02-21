use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;

pub struct IfNode;

#[async_trait]
impl INodeType for IfNode {
    fn get_description(&self) -> IDataObject {
        IDataObject(serde_json::json!({
            "name": "IF",
            "description": "Route items based on conditions"
        }))
    }

    async fn execute(&self, context: &dyn IExecuteFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;        
        let operation = context.get_node_parameter("operation", None)
            .await
            .map(|v| v.0.as_str().unwrap_or("equals").to_string())
            .unwrap_or_else(|_| "equals".to_string());
            
        let property1 = context.get_node_parameter("value1", None)
            .await
            .map(|v| v.0.as_str().unwrap_or("").to_string())
            .unwrap_or_else(|_| "".to_string());

        let mut true_branch = Vec::new();
        let mut false_branch = Vec::new();

        for item in input_data {
            let item_value = item.json.0.get(&property1)
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            let v1_str = item_value.as_str().unwrap_or("");
                
            let matches = if let Ok(prop2_result) = context.get_node_parameter("value2", None).await {
                let v2 = prop2_result.0.as_str().unwrap_or("");
                match operation.as_str() {
                    "equals" => v1_str == v2,
                    "notEquals" => v1_str != v2,
                    "contains" => v1_str.contains(v2),
                    _ => v1_str == v2,
                }
            } else {
                match operation.as_str() {
                    "exists" => !v1_str.is_empty(),
                    "notExists" => v1_str.is_empty(),
                    _ => !v1_str.is_empty(),
                }
            };

            if matches {
                true_branch.push(item.clone());
            } else {
                false_branch.push(item.clone());
            }
        }

        Ok(vec![true_branch, false_branch])
    }
}

pub struct SwitchNode;

#[async_trait]
impl INodeType for SwitchNode {
    fn get_description(&self) -> IDataObject {
        IDataObject(serde_json::json!({
            "name": "Switch",
            "description": "Route items based on matching values"
        }))
    }

    async fn execute(&self, context: &dyn IExecuteFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;        
        let data_property = context.get_node_parameter("dataProperty", None)
            .await
            .map(|v| v.0.as_str().unwrap_or("").to_string())
            .unwrap_or_else(|_| "".to_string());

        let fallback_output: usize = context.get_node_parameter("fallbackOutput", None)
            .await
            .map(|v| v.0.as_u64().unwrap_or(0) as usize)
            .unwrap_or(9);

        let mut outputs: Vec<Vec<INodeExecutionData>> = vec![Vec::new(); 10];

        for item in input_data {
            let switch_val = item.json.0.get(&data_property)
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            let switch_value = switch_val.as_str().unwrap_or("");

            let mut matched = false;            
            for i in 0..8 {
                let case_prop = format!("case{}", i);
                if let Ok(case_value) = context.get_node_parameter(&case_prop, None).await {
                    let case_str = case_value.0.as_str().unwrap_or("");
                    if switch_value == case_str {
                        outputs[i].push(item.clone());
                        matched = true;
                        break;
                    }
                }
            }

            if !matched {
                outputs[fallback_output].push(item.clone());
            }
        }

        Ok(outputs)
    }
}

pub struct MergeNode;

#[async_trait]
impl INodeType for MergeNode {
    fn get_description(&self) -> IDataObject {
        IDataObject(serde_json::json!({
            "name": "Merge",
            "description": "Merge multiple branches into one"
        }))
    }

    async fn execute(&self, context: &dyn IExecuteFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let mode = context.get_node_parameter("mode", None)
            .await
            .map(|v| v.0.as_str().unwrap_or("append").to_string())
            .unwrap_or_else(|_| "append".to_string());

        let mut merged = Vec::new();

        for input_index in 0..2 {
            if let Ok(input_data) = context.get_input_data(input_index) {
                merged.extend(input_data.iter().cloned());
            }
        }

        Ok(vec![merged])
    }
}
