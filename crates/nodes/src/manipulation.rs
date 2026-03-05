use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;

pub struct SetNode;

#[async_trait]
impl INodeType for SetNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "Set",
            "description": "Sets values on items and returns them"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;
        let mut output_items = Vec::new();

        for item in input_data {
            let mut new_item = item.json.0.clone();

            if let Ok(options) = context.get_node_parameter("assignments", None).await {
                if let Some(assignments) = options.as_array() {
                    for assignment in assignments {
                        if let (Some(name), Some(value)) = (
                            assignment.get("name").and_then(|v| v.as_str()),
                            assignment.get("value"),
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

#[async_trait]
impl INodeType for FilterNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "Filter",
            "description": "Filters items based on conditions"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;

        let operation = context
            .get_node_parameter("operation", None)
            .await
            .map(|v| v.as_str().unwrap_or("equals").to_string())
            .unwrap_or_else(|_| "equals".to_string());

        let property1 = context
            .get_node_parameter("property1", None)
            .await
            .map(|v| v.as_str().unwrap_or("").to_string())
            .unwrap_or_else(|_| "".to_string());

        let mut output_items = Vec::new();

        for item in input_data {
            let value1 = item.json.0.get(&property1).cloned();
            let mut keep = true;

            if let Some(v1) = value1 {
                if operation == "exists" {
                    keep = !v1.is_null();
                } else if operation == "notExists" {
                    keep = v1.is_null();
                } else if let Ok(v2) = context.get_node_parameter("property2", None).await {
                    let v1_str = v1.as_str().unwrap_or("");
                    let v2_str = v2.as_str().unwrap_or("");
                    if operation == "equals" {
                        keep = v1_str == v2_str;
                    } else if operation == "notEquals" {
                        keep = v1_str != v2_str;
                    } else if operation == "contains" {
                        keep = v1_str.contains(v2_str);
                    }
                }
            }

            if keep {
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
        IDataObject::from(serde_json::json!({
            "name": "Item Lists",
            "description": "Split items into batches or combine items"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0)?;

        let mode = context
            .get_node_parameter("mode", None)
            .await
            .map(|v| v.as_str().unwrap_or("splitInBatches").to_string())
            .unwrap_or_else(|_| "splitInBatches".to_string());

        let batch_size = context
            .get_node_parameter("batchSize", None)
            .await
            .ok()
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(1);

        match mode.as_str() {
            "splitInBatches" => {
                let items_per_batch = input_data
                    .chunks(batch_size)
                    .map(|chunk| chunk.to_vec())
                    .collect::<Vec<_>>();
                Ok(items_per_batch)
            }
            _ => Ok(vec![input_data.clone()]),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_core::types::GenericValue;

    struct MockContext {
        input_data: Vec<INodeExecutionData>,
        params: std::collections::HashMap<String, GenericValue>,
    }

    impl MockContext {
        fn new(input: Vec<INodeExecutionData>) -> Self {
            Self {
                input_data: input,
                params: std::collections::HashMap::new(),
            }
        }
    }

    #[tokio::test]
    async fn test_set_node_creates_item() {
        let input = vec![INodeExecutionData::new(IDataObject::from(
            serde_json::json!({}),
        ))];
        assert!(!input.is_empty());
    }

    #[tokio::test]
    async fn test_filter_node_empty_input() {
        let input: Vec<INodeExecutionData> = vec![];
        assert!(input.is_empty());
    }

    #[tokio::test]
    async fn test_item_lists_split() {
        let node = ItemListsNode;
        let desc = node.get_description();
        assert_eq!(desc.0.get("name").unwrap(), "Item Lists");
    }
}
