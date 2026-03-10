use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use serde_json::Value;

pub struct SetNode;

impl SetNode {
    fn parse_assignments(value: &Value) -> Vec<Value> {
        if value.is_null() {
            return Vec::new();
        }

        if let Some(array) = value.as_array() {
            return array.clone();
        }

        if let Some(object) = value.as_object() {
            if let Some(array) = object.get("assignments").and_then(|v| v.as_array()) {
                return array.clone();
            }

            if let Some(array) = object.get("values").and_then(|v| v.as_array()) {
                return array.clone();
            }

            // Accept n8n-like grouped object forms, e.g. { "string": [{name, value}] }
            let mut flattened = Vec::new();
            for (typed_key, typed_values) in object {
                if let Some(entries) = typed_values.as_array() {
                    for entry in entries {
                        if let Some(map) = entry.as_object() {
                            let mut assignment = Value::Object(map.clone());
                            if assignment.get("type").is_none() {
                                assignment["type"] = Value::String(typed_key.clone());
                            }
                            flattened.push(assignment);
                        }
                    }
                }
            }

            return flattened;
        }

        if let Some(raw) = value.as_str() {
            if let Ok(parsed) = serde_json::from_str::<Value>(raw) {
                return Self::parse_assignments(&parsed);
            }
        }

        Vec::new()
    }

    fn coerce_assignment_value(value: &Value, target_type: Option<&str>) -> Value {
        match target_type {
            Some("string") => {
                if let Some(as_str) = value.as_str() {
                    Value::String(as_str.to_string())
                } else {
                    Value::String(value.to_string())
                }
            }
            Some("number") => {
                if let Some(number) = value.as_f64() {
                    serde_json::json!(number)
                } else if let Some(as_str) = value.as_str() {
                    if let Ok(parsed) = as_str.parse::<f64>() {
                        serde_json::json!(parsed)
                    } else {
                        value.clone()
                    }
                } else {
                    value.clone()
                }
            }
            Some("boolean") => {
                if let Some(boolean) = value.as_bool() {
                    serde_json::json!(boolean)
                } else if let Some(as_str) = value.as_str() {
                    let lowered = as_str.trim().to_lowercase();
                    serde_json::json!(lowered == "true" || lowered == "1")
                } else {
                    value.clone()
                }
            }
            _ => value.clone(),
        }
    }
}

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
        let input_data = context.get_input_data(0).await?;
        let mut output_items = Vec::new();

        let keep_only_set_direct = context
            .get_node_parameter("keepOnlySet", None)
            .await
            .ok()
            .and_then(|v| v.as_bool());
        let keep_only_set_legacy = context
            .get_node_parameter("options", None)
            .await
            .ok()
            .and_then(|value| {
                value
                    .get("keepOnlySet")
                    .and_then(|flag| flag.as_bool())
                    .or_else(|| {
                        value.as_str().and_then(|raw| {
                            serde_json::from_str::<Value>(raw).ok().and_then(|parsed| {
                                parsed.get("keepOnlySet").and_then(|flag| flag.as_bool())
                            })
                        })
                    })
            });
        let keep_only_set = keep_only_set_direct
            .or(keep_only_set_legacy)
            .unwrap_or(false);

        for (item_index, item) in input_data.iter().enumerate() {
            let mut new_item = if keep_only_set {
                serde_json::Map::new()
            } else {
                item.json.0.clone()
            };

            let mut assignments_value = context
                .get_node_parameter_at_item("assignments", item_index, None)
                .await
                .ok();
            if assignments_value.is_none() {
                assignments_value = context
                    .get_node_parameter_at_item("values", item_index, None)
                    .await
                    .ok();
            }

            if let Some(raw_assignments) = assignments_value {
                let assignments = SetNode::parse_assignments(&raw_assignments);

                for assignment in assignments {
                    if let (Some(name), Some(value)) = (
                        assignment.get("name").and_then(|v| v.as_str()),
                        assignment.get("value"),
                    ) {
                        let target_type = assignment.get("type").and_then(|t| t.as_str());
                        let val = SetNode::coerce_assignment_value(value, target_type);
                        new_item.insert(name.to_string(), val);
                    }
                }
            }

            output_items.push(INodeExecutionData::new(IDataObject(new_item)));
        }

        Ok(vec![output_items])
    }
}

pub struct FilterNode;

impl FilterNode {
    fn parse_conditions(value: &Value) -> Vec<Value> {
        if value.is_null() {
            return Vec::new();
        }

        if let Some(array) = value.as_array() {
            return array.clone();
        }

        if let Some(object) = value.as_object() {
            if let Some(array) = object.get("conditions").and_then(|v| v.as_array()) {
                return array.clone();
            }
        }

        if let Some(raw) = value.as_str() {
            if let Ok(parsed) = serde_json::from_str::<Value>(raw) {
                return Self::parse_conditions(&parsed);
            }
        }

        Vec::new()
    }

    fn numeric_value(value: &Value) -> Option<f64> {
        value
            .as_f64()
            .or_else(|| value.as_str().and_then(|s| s.parse::<f64>().ok()))
    }

    fn evaluate_condition(operation: &str, value1: &Value, value2: Option<&Value>) -> bool {
        let value1_str = value1.as_str().unwrap_or("");
        let value1_num = Self::numeric_value(value1);

        match operation {
            "exists" => !value1.is_null() && value1.as_str().map(|s| !s.is_empty()).unwrap_or(true),
            "notExists" => {
                value1.is_null() || value1.as_str().map(|s| s.is_empty()).unwrap_or(false)
            }
            "contains" => value2
                .and_then(|v| v.as_str())
                .map(|needle| value1_str.contains(needle))
                .unwrap_or(false),
            "larger" => value2
                .and_then(Self::numeric_value)
                .and_then(|rhs| value1_num.map(|lhs| lhs > rhs))
                .unwrap_or(false),
            "largerEqual" => value2
                .and_then(Self::numeric_value)
                .and_then(|rhs| value1_num.map(|lhs| lhs >= rhs))
                .unwrap_or(false),
            "smaller" => value2
                .and_then(Self::numeric_value)
                .and_then(|rhs| value1_num.map(|lhs| lhs < rhs))
                .unwrap_or(false),
            "smallerEqual" => value2
                .and_then(Self::numeric_value)
                .and_then(|rhs| value1_num.map(|lhs| lhs <= rhs))
                .unwrap_or(false),
            "notEquals" => value2.map(|v| value1 != v).unwrap_or(!value1.is_null()),
            _ => value2.map(|v| value1 == v).unwrap_or(!value1.is_null()),
        }
    }
}

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
        let input_data = context.get_input_data(0).await?;

        let operation = context
            .get_node_parameter("operation", None)
            .await
            .map(|v| v.as_str().unwrap_or("equals").to_string())
            .unwrap_or_else(|_| "equals".to_string());
        let combine_operation = context
            .get_node_parameter("combineOperation", None)
            .await
            .map(|v| v.as_str().unwrap_or("all").to_string())
            .unwrap_or_else(|_| "all".to_string());

        let mut output_items = Vec::new();

        for (item_index, item) in input_data.iter().enumerate() {
            let configured_conditions = context
                .get_node_parameter_at_item("conditions", item_index, None)
                .await
                .ok()
                .map(|v| FilterNode::parse_conditions(&v))
                .unwrap_or_default();

            let keep = if configured_conditions.is_empty() {
                let value1 = context
                    .get_node_parameter_at_item("value1", item_index, None)
                    .await
                    .unwrap_or(serde_json::Value::Null);
                let value2 = context
                    .get_node_parameter_at_item("value2", item_index, None)
                    .await
                    .ok();

                FilterNode::evaluate_condition(&operation, &value1, value2.as_ref())
            } else {
                let condition_results: Vec<bool> = configured_conditions
                    .iter()
                    .map(|condition| {
                        let op = condition
                            .get("operation")
                            .and_then(|v| v.as_str())
                            .or_else(|| {
                                condition
                                    .get("operator")
                                    .and_then(|v| v.get("operation"))
                                    .and_then(|v| v.as_str())
                            })
                            .unwrap_or("equals");

                        let value1 = condition
                            .get("value1")
                            .or_else(|| condition.get("leftValue"))
                            .cloned()
                            .unwrap_or(Value::Null);
                        let value2 = condition
                            .get("value2")
                            .or_else(|| condition.get("rightValue"));

                        FilterNode::evaluate_condition(op, &value1, value2)
                    })
                    .collect();

                if combine_operation == "any" {
                    condition_results.into_iter().any(|matched| matched)
                } else {
                    condition_results.into_iter().all(|matched| matched)
                }
            };

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
        let input_data = context.get_input_data(0).await?;

        let mode = context
            .get_node_parameter("mode", None)
            .await
            .map(|v| v.as_str().unwrap_or("splitInBatches").to_string())
            .unwrap_or_else(|_| "splitInBatches".to_string());

        let normalized_mode = if context.get_node().r#type == "n8n-nodes-base.splitInBatches" {
            "splitInBatches".to_string()
        } else {
            mode
        };

        let batch_size = context
            .get_node_parameter("batchSize", None)
            .await
            .ok()
            .and_then(|v| v.as_u64())
            .map(|n| n as usize)
            .unwrap_or(1);
        let safe_batch_size = if batch_size == 0 { 1 } else { batch_size };

        match normalized_mode.as_str() {
            "splitInBatches" => {
                let items_per_batch = input_data
                    .chunks(safe_batch_size)
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

    use barqflow_core::schema::INode;

    struct MockContext {
        input_data: Vec<INodeExecutionData>,
        params: std::collections::HashMap<String, GenericValue>,
        node: INode,
    }

    impl MockContext {
        fn new(input: Vec<INodeExecutionData>) -> Self {
            Self {
                input_data: input,
                params: std::collections::HashMap::new(),
                node: INode {
                    id: barqflow_core::types::NodeId("test_node".into()),
                    name: "Test Node".into(),
                    r#type: "test".into(),
                    type_version: 1.0,
                    position: [0.0, 0.0],
                    parameters: barqflow_core::schema::INodeParameters(
                        std::collections::HashMap::new(),
                    ),
                    credentials: vec![],
                    disabled: false,
                },
            }
        }

        fn add_param(&mut self, key: &str, value: serde_json::Value) {
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
        ) -> Result<std::collections::HashMap<String, GenericValue>, BarqError> {
            Ok(std::collections::HashMap::new())
        }

        fn log(&self, _message: &str) {}
    }

    #[tokio::test]
    async fn test_set_node_creates_item() {
        let input = vec![INodeExecutionData::new(IDataObject::from(
            serde_json::json!({"old_key": "old_value"}),
        ))];

        let mut context = MockContext::new(input);
        context.add_param(
            "assignments",
            serde_json::json!([
                { "name": "new_key", "value": "new_val", "type": "string" }
            ]),
        );

        let node = SetNode;
        let result = node.execute(&context).await.unwrap();

        assert_eq!(result.len(), 1);
        let output_items = &result[0];
        assert_eq!(output_items.len(), 1);

        let val = &output_items[0].json.0;
        assert_eq!(val.get("new_key").unwrap().as_str().unwrap(), "new_val");
        assert_eq!(val.get("old_key").unwrap().as_str().unwrap(), "old_value");
    }

    #[tokio::test]
    async fn test_set_node_keep_only_set_from_legacy_options() {
        let input = vec![INodeExecutionData::new(IDataObject::from(
            serde_json::json!({"old_key": "old_value"}),
        ))];

        let mut context = MockContext::new(input);
        context.add_param("options", serde_json::json!({ "keepOnlySet": true }));
        context.add_param(
            "values",
            serde_json::json!([
                { "name": "new_key", "value": "new_val", "type": "string" }
            ]),
        );

        let node = SetNode;
        let result = node.execute(&context).await.unwrap();

        let val = &result[0][0].json.0;
        assert_eq!(val.get("new_key").unwrap().as_str().unwrap(), "new_val");
        assert!(val.get("old_key").is_none());
    }

    #[tokio::test]
    async fn test_filter_node_operation() {
        let input = vec![
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"val": "keep"}))),
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"val": "drop"}))),
        ];

        let mut context = MockContext::new(input);
        context.add_param("operation", serde_json::json!("notEquals"));
        // Simulating the item expression mapping where value1 evaluates per item.
        // Since MockContext doesn't evaluate per item yet, we'll test simple case.
        // So for MockContext, we will just say `value1` is "keep" and `value2` is "drop"
        // Wait, FilterNode evaluates `value1 == value2`. Let's test `exists`.
        context.add_param("operation", serde_json::json!("exists"));
        context.add_param("value1", serde_json::json!("I exist"));

        let node = FilterNode;
        let result = node.execute(&context).await.unwrap();

        assert_eq!(result[0].len(), 2);
    }

    #[tokio::test]
    async fn test_filter_node_conditions_any_mode() {
        let input = vec![INodeExecutionData::new(IDataObject::from(
            serde_json::json!({"status": "ready"}),
        ))];

        let mut context = MockContext::new(input);
        context.add_param("combineOperation", serde_json::json!("any"));
        context.add_param(
            "conditions",
            serde_json::json!([
                { "value1": "queued", "operation": "equals", "value2": "ready" },
                { "value1": 10, "operation": "larger", "value2": 5 }
            ]),
        );

        let node = FilterNode;
        let result = node.execute(&context).await.unwrap();

        assert_eq!(result[0].len(), 1);
    }

    #[tokio::test]
    async fn test_item_lists_split() {
        let input = vec![
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"id": 1}))),
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"id": 2}))),
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"id": 3}))),
        ];

        let mut context = MockContext::new(input);
        context.add_param("mode", serde_json::json!("splitInBatches"));
        context.add_param("batchSize", serde_json::json!(2));

        let node = ItemListsNode;
        let result = node.execute(&context).await.unwrap();

        assert_eq!(result.len(), 2); // 2 batches
        assert_eq!(result[0].len(), 2); // First batch has 2 items
        assert_eq!(result[1].len(), 1); // Second batch has 1 item
    }

    #[tokio::test]
    async fn test_split_in_batches_alias_uses_batch_size() {
        let input = vec![
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"id": 1}))),
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"id": 2}))),
        ];

        let mut context = MockContext::new(input);
        context.node.r#type = "n8n-nodes-base.splitInBatches".to_string();
        context.add_param("batchSize", serde_json::json!(0));

        let node = ItemListsNode;
        let result = node.execute(&context).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), 1);
        assert_eq!(result[1].len(), 1);
    }
}
