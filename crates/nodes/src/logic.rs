use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use serde_json::Value;
use std::collections::HashMap;

pub struct IfNode;

impl IfNode {
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
impl INodeType for IfNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "IF",
            "description": "Route items based on conditions"
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

        let mut true_branch = Vec::new();
        let mut false_branch = Vec::new();

        for (item_index, item) in input_data.iter().enumerate() {
            let configured_conditions = context
                .get_node_parameter_at_item("conditions", item_index, None)
                .await
                .ok()
                .map(|v| IfNode::parse_conditions(&v))
                .unwrap_or_default();

            let matches = if configured_conditions.is_empty() {
                // Legacy single-condition parameters for backward compatibility.
                let v1 = context
                    .get_node_parameter_at_item("value1", item_index, None)
                    .await
                    .unwrap_or(Value::Null);
                let v2 = context
                    .get_node_parameter_at_item("value2", item_index, None)
                    .await
                    .ok();

                IfNode::evaluate_condition(&operation, &v1, v2.as_ref())
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

                        IfNode::evaluate_condition(op, &value1, value2)
                    })
                    .collect();

                if combine_operation == "any" {
                    condition_results.into_iter().any(|matched| matched)
                } else {
                    condition_results.into_iter().all(|matched| matched)
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
        IDataObject::from(serde_json::json!({
            "name": "Switch",
            "description": "Route items based on matching values"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let input_data = context.get_input_data(0).await?;
        let data_property = context
            .get_node_parameter("dataProperty", None)
            .await
            .map(|v| v.as_str().unwrap_or("").to_string())
            .unwrap_or_else(|_| "".to_string());

        let fallback_output: usize = context
            .get_node_parameter("fallbackOutput", None)
            .await
            .map(|v| v.as_u64().unwrap_or(0) as usize)
            .unwrap_or(9);

        let mut outputs: Vec<Vec<INodeExecutionData>> = vec![Vec::new(); 10];

        for item in input_data {
            let switch_val = item
                .json
                .0
                .get(&data_property)
                .cloned()
                .unwrap_or(serde_json::Value::Null);
            let switch_value = switch_val.as_str().unwrap_or("");

            let mut matched = false;
            for i in 0..8 {
                let case_prop = format!("case{}", i);
                if let Ok(case_value) = context.get_node_parameter(&case_prop, None).await {
                    let case_str = case_value.as_str().unwrap_or("");
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

impl MergeNode {
    fn normalize_mode(mode: &str) -> &str {
        match mode {
            "merge" | "keepKeyMatches" => "merge",
            "multiplex" => "multiplex",
            _ => "append",
        }
    }

    fn key_for_merge(value: &Value) -> String {
        match value {
            Value::String(v) => format!("s:{v}"),
            Value::Number(v) => format!("n:{v}"),
            Value::Bool(v) => format!("b:{v}"),
            Value::Null => "null".to_string(),
            _ => format!("j:{}", value),
        }
    }

    fn merge_json_objects(
        left: &serde_json::Map<String, Value>,
        right: &serde_json::Map<String, Value>,
    ) -> INodeExecutionData {
        let mut combined = left.clone();
        for (key, value) in right {
            combined.insert(key.clone(), value.clone());
        }
        INodeExecutionData::new(IDataObject(combined))
    }
}

#[async_trait]
impl INodeType for MergeNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "Merge",
            "description": "Merge multiple branches into one"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let raw_mode = context
            .get_node_parameter("mode", None)
            .await
            .map(|v| v.as_str().unwrap_or("append").to_string())
            .unwrap_or_else(|_| "append".to_string());
        let mode = MergeNode::normalize_mode(&raw_mode);

        let input_1 = context.get_input_data(0).await.unwrap_or_default();
        let input_2 = context.get_input_data(1).await.unwrap_or_default();

        let mut merged = Vec::new();
        match mode {
            "append" => {
                merged.extend(input_1.iter().cloned());
                merged.extend(input_2.iter().cloned());
            }
            "merge" => {
                let prop_1 = context
                    .get_node_parameter("property1", None)
                    .await
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| "id".to_string());
                let prop_2 = context
                    .get_node_parameter("property2", None)
                    .await
                    .ok()
                    .and_then(|v| v.as_str().map(|s| s.to_string()))
                    .filter(|s| !s.trim().is_empty())
                    .unwrap_or_else(|| "id".to_string());

                let mut right_by_key: HashMap<String, Vec<serde_json::Map<String, Value>>> =
                    HashMap::new();
                for right_item in &input_2 {
                    if let Some(right_value) = right_item.json.0.get(&prop_2) {
                        right_by_key
                            .entry(MergeNode::key_for_merge(right_value))
                            .or_default()
                            .push(right_item.json.0.clone());
                    }
                }

                for left_item in &input_1 {
                    if let Some(left_value) = left_item.json.0.get(&prop_1) {
                        if let Some(matches) =
                            right_by_key.get(&MergeNode::key_for_merge(left_value))
                        {
                            for right_json in matches {
                                merged.push(MergeNode::merge_json_objects(
                                    &left_item.json.0,
                                    right_json,
                                ));
                            }
                        }
                    }
                }
            }
            "multiplex" => {
                for left_item in &input_1 {
                    for right_item in &input_2 {
                        merged.push(MergeNode::merge_json_objects(
                            &left_item.json.0,
                            &right_item.json.0,
                        ));
                    }
                }
            }
            _ => {}
        }

        Ok(vec![merged])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use barqflow_core::schema::{INode, INodeParameters};
    use barqflow_core::types::{GenericValue, NodeId};

    struct MockContext {
        input_data: Vec<Vec<INodeExecutionData>>,
        params: std::collections::HashMap<String, GenericValue>,
        node: INode,
    }

    impl MockContext {
        fn new(inputs: Vec<Vec<INodeExecutionData>>) -> Self {
            Self {
                input_data: inputs,
                params: std::collections::HashMap::new(),
                node: INode {
                    id: NodeId("test_node".into()),
                    name: "Test Node".into(),
                    r#type: "test".into(),
                    type_version: 1.0,
                    position: [0.0, 0.0],
                    parameters: INodeParameters(std::collections::HashMap::new()),
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
            input_index: usize,
        ) -> Result<Vec<INodeExecutionData>, BarqError> {
            self.input_data
                .get(input_index)
                .cloned()
                .ok_or(BarqError::NodeOperationError {
                    node_name: self.node.name.clone(),
                    message: format!("No input data at index {}", input_index),
                })
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
    async fn test_if_node_operation() {
        let input = vec![
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"val": 10}))),
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"val": 5}))),
        ];

        let mut context = MockContext::new(vec![input]);
        context.add_param("operation", serde_json::json!("larger"));
        context.add_param("value1", serde_json::json!(10));
        context.add_param("value2", serde_json::json!(8));

        let node = IfNode;
        let result = node.execute(&context).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), 2);
    }

    #[tokio::test]
    async fn test_if_node_conditions_any_mode() {
        let input = vec![INodeExecutionData::new(IDataObject::from(
            serde_json::json!({"id": 1}),
        ))];

        let mut context = MockContext::new(vec![input]);
        context.add_param("combineOperation", serde_json::json!("any"));
        context.add_param(
            "conditions",
            serde_json::json!([
                { "value1": 5, "operation": "larger", "value2": 10 },
                { "value1": "hello", "operation": "contains", "value2": "ell" }
            ]),
        );

        let node = IfNode;
        let result = node.execute(&context).await.unwrap();

        assert_eq!(result[0].len(), 1);
        assert_eq!(result[1].len(), 0);
    }

    #[tokio::test]
    async fn test_switch_node_routing() {
        let input = vec![
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"route": "A"}))),
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"route": "B"}))),
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"route": "C"}))),
        ];

        let mut context = MockContext::new(vec![input]);
        context.add_param("dataProperty", serde_json::json!("route"));
        context.add_param("case0", serde_json::json!("A"));
        context.add_param("case1", serde_json::json!("B"));
        context.add_param("fallbackOutput", serde_json::json!(2));

        let node = SwitchNode;
        let result = node.execute(&context).await.unwrap();

        assert_eq!(result.len(), 10);
        assert_eq!(result[0].len(), 1);
        assert_eq!(result[1].len(), 1);
        assert_eq!(result[2].len(), 1);
    }

    #[tokio::test]
    async fn test_merge_node_append() {
        let input1 = vec![INodeExecutionData::new(IDataObject::from(
            serde_json::json!({"id": 1}),
        ))];
        let input2 = vec![INodeExecutionData::new(IDataObject::from(
            serde_json::json!({"id": 2}),
        ))];

        let mut context = MockContext::new(vec![input1, input2]);
        context.add_param("mode", serde_json::json!("append"));

        let node = MergeNode;
        let result = node.execute(&context).await.unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
    }

    #[tokio::test]
    async fn test_merge_node_merge_alias() {
        let input1 = vec![INodeExecutionData::new(IDataObject::from(
            serde_json::json!({"id": 1, "left": true}),
        ))];
        let input2 = vec![
            INodeExecutionData::new(IDataObject::from(
                serde_json::json!({"id": 1, "right": true}),
            )),
            INodeExecutionData::new(IDataObject::from(
                serde_json::json!({"id": 2, "right": false}),
            )),
        ];

        let mut context = MockContext::new(vec![input1, input2]);
        context.add_param("mode", serde_json::json!("merge"));
        context.add_param("property1", serde_json::json!("id"));
        context.add_param("property2", serde_json::json!("id"));

        let node = MergeNode;
        let result = node.execute(&context).await.unwrap();

        assert_eq!(result[0].len(), 1);
        assert_eq!(
            result[0][0].json.0.get("left").and_then(|v| v.as_bool()),
            Some(true)
        );
        assert_eq!(
            result[0][0].json.0.get("right").and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    #[tokio::test]
    async fn test_merge_node_multiplex_mode() {
        let input1 = vec![
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"left": 1}))),
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"left": 2}))),
        ];
        let input2 = vec![
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"right": "A"}))),
            INodeExecutionData::new(IDataObject::from(serde_json::json!({"right": "B"}))),
        ];

        let mut context = MockContext::new(vec![input1, input2]);
        context.add_param("mode", serde_json::json!("multiplex"));

        let node = MergeNode;
        let result = node.execute(&context).await.unwrap();

        assert_eq!(result[0].len(), 4);
    }
}
