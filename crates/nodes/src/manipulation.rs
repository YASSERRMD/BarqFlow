use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;

pub struct SetNode;

#[async_trait]
impl INodeType for SetNode {
    fn get_description(&self) -> IDataObject {
        IDataObject(serde_json::json!({
            "name": "Set",
            "description": "Sets values on items and returns them"
        }))
    }

    async fn execute(&self, context: &dyn IExecuteFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;
        let mut output_items = Vec::new();

        for item in input_data {
            let mut new_item = item.json.0.clone();
            
            if let Ok(options) = context.get_node_parameter("assignments", None).await {
                if let Some(assignments) = options.0.as_array() {
                    for assignment in assignments {
                        if let (Some(name), Some(value)) = (
                            assignment.get("name").and_then(|v| v.as_str()),
                            assignment.get("value")
                        ) {
                            if let serde_json::Value::Object(ref mut map) = new_item {
                                map.insert(name.to_string(), value.clone());
                            }
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

#[async_trait]
impl INodeType for FilterNode {
    fn get_description(&self) -> IDataObject {
        IDataObject(serde_json::json!({
            "name": "Filter",
            "description": "Filters items based on conditions"
        }))
    }

    async fn execute(&self, context: &dyn IExecuteFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;
        
        let operation = context.get_node_parameter("operation", None)
            .await
            .map(|v| v.0.as_str().unwrap_or("equals").to_string())
            .unwrap_or_else(|_| "equals".to_string());
            
        let property1 = context.get_node_parameter("property1", None)
            .await
            .map(|v| v.0.as_str().unwrap_or("").to_string())
            .unwrap_or_else(|_| "".to_string());

        let property2 = context.get_node_parameter("property2", None)
            .await
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

#[async_trait]
impl INodeType for ItemListsNode {
    fn get_description(&self) -> IDataObject {
        IDataObject(serde_json::json!({
            "name": "Item Lists",
            "description": "Split items into batches or combine items"
        }))
    }

    async fn execute(&self, context: &dyn IExecuteFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;
        
        let mode = context.get_node_parameter("mode", None)
            .await
            .map(|v| v.0.as_str().unwrap_or("splitInBatches").to_string())
            .unwrap_or_else(|_| "splitInBatches".to_string());

        let batch_size = context.get_node_parameter("batchSize", None)
            .await
            .ok()
            .and_then(|v| v.0.as_u64())
            .map(|n| n as usize)
            .unwrap_or(1);

        match mode.as_str() {
            "splitInBatches" => {
                let include_others = context.get_node_parameter("includeOtherElements", None)
                    .await
                    .map(|v| v.0.as_bool().unwrap_or(false))
                    .unwrap_or(false);

                let items_per_batch = input_data.chunks(batch_size).map(|chunk| {
                    chunk.iter().cloned().collect::<Vec<_>>()
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
