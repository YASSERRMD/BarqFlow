use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use serde::{Deserialize, Serialize};

pub struct SetNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetNodeOptions {
    pub assignments: Vec<FieldAssignment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAssignment {
    pub name: String,
    pub value: serde_json::Value,
}

#[async_trait]
impl INodeType for SetNode {
    fn get_description(&self) -> IDataObject {
        IDataObject(serde_json::json!({
            "name": "Set",
            "description": "Sets values on items and returns them",
            "properties": []
        }))
    }

    async fn execute(&self, context: &dyn IExecuteFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;
        let mut output_items = Vec::new();

        for item in input_data {
            let mut new_item = item.json.0.clone();
            
            if let Ok(options) = context.get_node_parameter("assignments", None) {
                if let Some(assignments) = options.0.as_array() {
                    for assignment in assignments {
                        if let (Some(name), Some(value)) = (
                            assignment.get("name").and_then(|v| v.as_str()),
                            assignment.get("value")
                        ) {
                            new_item.insert(name.to_string(), value.clone());
                        }
                    }
                }
            }

            output_items.push(INodeExecutionData::new(IDataObject(new_item)));
        }

        Ok(vec![output_items])
    }
}

pub struct FilterNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterNodeOptions {
    pub operation: String,
    pub property1: String,
    pub property2: Option<String>,
}

#[async_trait]
impl INodeType for FilterNode {
    fn get_description(&self) -> IDataObject {
        IDataObject(serde_json::json!({
            "name": "Filter",
            "description": "Filters items based on conditions",
            "properties": []
        }))
    }

    async fn execute(&self, context: &dyn IExecuteFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;
        
        let operation = context.get_node_parameter("operation", None)
            .map(|v| v.0.as_str().unwrap_or("equals").to_string())
            .unwrap_or_else(|_| "equals".to_string());
            
        let property1 = context.get_node_parameter("property1", None)
            .map(|v| v.0.as_str().unwrap_or("").to_string())
            .unwrap_or_else(|_| "".to_string());

        let property2 = context.get_node_parameter("property2", None)
            .map(|v| v.0.as_str().unwrap_or("").to_string())
            .ok();

        let mut output_items = Vec::new();

        for item in input_data {
            let value1 = item.json.0.get(&property1).cloned();
            
            let should_keep = match (&value1, &property2, operation.as_str()) {
                (Some(v1), Some(v2), "equals") => v1 == v2,
                (Some(v1), Some(v2), "notEquals") => v1 != v2,
                (Some(v1), Some(v2), "contains") => {
                    if let (Some(s1), Some(s2)) = (v1.as_str(), v2.as_str()) {
                        s1.contains(s2)
                    } else {
                        false
                    }
                },
                (Some(v1), None, "exists") => !v1.is_null(),
                (Some(v1), None, "notExists") => v1.is_null(),
                _ => true,
            };

            if should_keep {
                output_items.push(item.clone());
            }
        }

        Ok(vec![output_items])
    }
}

pub struct ItemListsNode;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemListsNodeOptions {
    pub mode: String,
    pub batch_size: Option<usize>,
    pub include_other_elements: Option<bool>,
}

#[async_trait]
impl INodeType for ItemListsNode {
    fn get_description(&self) -> IDataObject {
        IDataObject(serde_json::json!({
            "name": "Item Lists",
            "description": "Split items into batches or combine items",
            "properties": []
        }))
    }

    async fn execute(&self, context: &dyn IExecuteFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;
        
        let mode = context.get_node_parameter("mode", None)
            .map(|v| v.0.as_str().unwrap_or("splitInBatches").to_string())
            .unwrap_or_else(|_| "splitInBatches".to_string());

        let batch_size = context.get_node_parameter("batchSize", None)
            .ok()
            .and_then(|v| v.0.as_u64())
            .map(|n| n as usize)
            .unwrap_or(1);

        match mode.as_str() {
            "splitInBatches" => {
                let include_others = context.get_node_parameter("includeOtherElements", None)
                    .map(|v| v.0.as_bool().unwrap_or(false))
                    .unwrap_or(false);

                let items_per_batch = input_data.chunks(batch_size).map(|chunk| {
                    chunk.iter().map(|item| item.clone()).collect::<Vec<_>>()
                }).collect::<Vec<_>>();

                let mut outputs = Vec::new();
                for batch in items_per_batch {
                    if !batch.is_empty() || include_others {
                        outputs.push(batch);
                    }
                }

                Ok(outputs)
            },
            "merge" => {
                Ok(vec![input_data.clone()])
            },
            "flatten" => {
                let mut flattened = Vec::new();
                for item in input_data {
                    flattened.push(item.clone());
                }
                Ok(vec![flattened])
            },
            _ => Ok(vec![input_data.clone()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_core::schema::INodeParameters;
    use barqflow_core::types::{GenericValue, NodeId};
    use std::collections::HashMap;

    struct MockExecuteContext {
        input_data: Vec<INodeExecutionData>,
        parameters: HashMap<String, GenericValue>,
    }

    impl MockExecuteContext {
        fn new(input_data: Vec<INodeExecutionData>) -> Self {
            Self {
                input_data,
                parameters: HashMap::new(),
            }
        }

        fn with_parameter(mut self, name: &str, value: serde_json::Value) -> Self {
            self.parameters.insert(name.to_string(), GenericValue(value));
            self
        }
    }

    #[async_trait]
    impl IExecuteFunctions for MockExecuteContext {
        async fn get_node_parameter(&self, parameter_name: &str, _fallback_value: Option<GenericValue>) -> Result<GenericValue, BarqError> {
            Ok(self.parameters.get(parameter_name)
                .cloned()
                .unwrap_or(GenericValue(serde_json::Value::Null)))
        }

        fn get_node(&self) -> &barqflow_core::schema::INode {
            unimplemented!()
        }

        fn get_input_data(&self, _input_index: usize) -> Result<&Vec<INodeExecutionData>, BarqError> {
            Ok(&self.input_data)
        }

        fn log(&self, _message: &str) {}
    }

    #[tokio::test]
    async fn test_set_node() {
        let input = vec![
            INodeExecutionData::new(IDataObject(serde_json::json!({"id": 1, "name": "test"})))
        ];
        
        let context = MockExecuteContext::new(input)
            .with_parameter("assignments", serde_json::json!([
                {"name": "newField", "value": "newValue"}
            ]));

        let node = SetNode;
        let result = node.execute(&context).await.unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 1);
        assert_eq!(result[0][0].json.0.get("newField"), Some(&serde_json::Value::String("newValue".to_string())));
    }

    #[tokio::test]
    async fn test_filter_node_equals() {
        let input = vec![
            INodeExecutionData::new(IDataObject(serde_json::json!({"status": "active"}))),
            INodeExecutionData::new(IDataObject(serde_json::json!({"status": "inactive"}))),
            INodeExecutionData::new(IDataObject(serde_json::json!({"status": "active"}))),
        ];
        
        let context = MockExecuteContext::new(input)
            .with_parameter("operation", "equals")
            .with_parameter("property1", "status")
            .with_parameter("property2", "active");

        let node = FilterNode;
        let result = node.execute(&context).await.unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
    }

    #[tokio::test]
    async fn test_item_lists_split() {
        let input = vec![
            INodeExecutionData::new(IDataObject(serde_json::json!({"id": 1}))),
            INodeExecutionData::new(IDataObject(serde_json::json!({"id": 2}))),
            INodeExecutionData::new(IDataObject(serde_json::json!({"id": 3}))),
            INodeExecutionData::new(IDataObject(serde_json::json!({"id": 4}))),
        ];
        
        let context = MockExecuteContext::new(input)
            .with_parameter("mode", "splitInBatches")
            .with_parameter("batchSize", serde_json::json!(2));

        let node = ItemListsNode;
        let result = node.execute(&context).await.unwrap();
        
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), 2);
        assert_eq!(result[1].len(), 2);
    }
}
