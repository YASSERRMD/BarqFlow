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
        
        // Fix for operation extraction
        let operation_param = context.get_node_parameter("operation", None).await;
        let operation = match operation_param {
            Ok(v) => v.0.as_str().unwrap_or("equals").to_string(),
            Err(_) => "equals".to_string(),
        };
            
        // Fix for property1 extraction
        let property1_param = context.get_node_parameter("property1", None).await;
        let property1 = match property1_param {
            Ok(v) => v.0.as_str().unwrap_or("").to_string(),
            Err(_) => "".to_string(),
        };

        // Fix for property2 extraction
        let property2_param = context.get_node_parameter("property2", None).await;
        let property2: Option<String> = match property2_param {
            Ok(v) => v.0.as_str().map(|s| s.to_string()),
            Err(_) => None,
        };

        let mut output_items = Vec::new();

        for item in input_data {
            let value1 = item.json.0.get(&property1).cloned();
            
            let should_keep = match (&value1, &property2, operation.as_str()) {
                (Some(v1), Some(ref v2), "equals") => {
                    if let (Some(s1), Some(s2)) = (v1.as_str(), Some(v2.as_str())) {
                        s1 == s2
                    } else {
                        false
                    }
                },
                (Some(v1), Some(ref v2), "notEquals") => {
                    if let (Some(s1), Some(s2)) = (v1.as_str(), Some(v2.as_str())) {
                        s1 != s2
                    } else {
                        false
                    }
                },
                (Some(v1), Some(ref v2), "contains") => {
                    if let (Some(s1), Some(s2)) = (v1.as_str(), Some(v2.as_str())) {
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
        
        // Fix for mode extraction
        let mode_param = context.get_node_parameter("mode", None).await;
        let mode = match mode_param {
            Ok(v) => v.0.as_str().unwrap_or("splitInBatches").to_string(),
            Err(_) => "splitInBatches".to_string(),
        };

        // Fix for batch_size extraction
        let batch_size_param = context.get_node_parameter("batchSize", None).await;
        let batch_size = match batch_size_param {
            Ok(v) => v.0.as_u64().map(|n| n as usize).unwrap_or(1),
            Err(_) => 1,
        };

        match mode.as_str() {
            "splitInBatches" => {
                // Fix for include_others extraction
                let include_others_param = context.get_node_parameter("includeOtherElements", None).await;
                let include_others = match include_others_param {
                    Ok(v) => v.0.as_bool().unwrap_or(false),
                    Err(_) => false,
                };

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
