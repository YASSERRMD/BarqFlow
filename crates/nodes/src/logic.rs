use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;

pub struct IfNode;

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

        let mut true_branch = Vec::new();
        let mut false_branch = Vec::new();

        for (item_index, item) in input_data.iter().enumerate() {
            let v1 = context
                .get_node_parameter_at_item("value1", item_index, None)
                .await
                .unwrap_or(serde_json::Value::Null);
            
            let v1_str = v1.as_str().unwrap_or("");
            let v1_num = v1.as_f64().or_else(|| v1_str.parse::<f64>().ok());

            let matches = if let Ok(v2) = context.get_node_parameter_at_item("value2", item_index, None).await
            {
                let v2_str = v2.as_str().unwrap_or("");
                let v2_num = v2.as_f64().or_else(|| v2_str.parse::<f64>().ok());
                
                match operation.as_str() {
                    "equals" => v1 == v2 || (v1_num.is_some() && v1_num == v2_num),
                    "notEquals" => v1 != v2 && (v1_num.is_none() || v1_num != v2_num),
                    "contains" => v1_str.contains(v2_str),
                    "larger" => v1_num.is_some() && v2_num.is_some() && v1_num.unwrap() > v2_num.unwrap(),
                    "largerEqual" => v1_num.is_some() && v2_num.is_some() && v1_num.unwrap() >= v2_num.unwrap(),
                    "smaller" => v1_num.is_some() && v2_num.is_some() && v1_num.unwrap() < v2_num.unwrap(),
                    "smallerEqual" => v1_num.is_some() && v2_num.is_some() && v1_num.unwrap() <= v2_num.unwrap(),
                    _ => v1 == v2,
                }
            } else {
                match operation.as_str() {
                    "exists" => !v1.is_null() && (v1.as_str().map(|s| !s.is_empty()).unwrap_or(true)),
                    "notExists" => v1.is_null() || (v1.as_str().map(|s| s.is_empty()).unwrap_or(false)),
                    _ => !v1.is_null(),
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
        let mode = context
            .get_node_parameter("mode", None)
            .await
            .map(|v| v.as_str().unwrap_or("append").to_string())
            .unwrap_or_else(|_| "append".to_string());

        let mut merged = Vec::new();

        if mode == "append" {
            for input_index in 0..2 {
                if let Ok(input_data) = context.get_input_data(input_index).await {
                    merged.extend(input_data.iter().cloned());
                }
            }
        } else if mode == "keepKeyMatches" {
            let prop_1 = context.get_node_parameter("property1", None).await.map(|v| v.as_str().unwrap_or("").to_string()).unwrap_or_default();
            let prop_2 = context.get_node_parameter("property2", None).await.map(|v| v.as_str().unwrap_or("").to_string()).unwrap_or_default();
            
            if let (Ok(input1), Ok(input2)) = (
                context.get_input_data(0).await,
                context.get_input_data(1).await,
            ) {
                for item1 in input1 {
                    let v1 = item1.json.0.get(&prop_1);
                    for item2 in &input2 {
                        let v2 = item2.json.0.get(&prop_2);
                        if v1.is_some() && v1 == v2 {
                            let mut combined = item1.json.0.clone();
                            for (k, v) in &item2.json.0 {
                                combined.insert(k.clone(), v.clone());
                            }
                            merged.push(INodeExecutionData::new(IDataObject(combined)));
                        }
                    }
                }
            }
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
            self.get_node_parameter(parameter_name, fallback_value).await
        }

        fn get_node(&self) -> &INode {
            &self.node
        }

        async fn get_input_data(&self, input_index: usize) -> Result<Vec<INodeExecutionData>, BarqError> {
            self.input_data
                .get(input_index)
                .cloned()
                .ok_or(BarqError::NodeOperationError {
                    node_name: self.node.name.clone(),
                    message: format!("No input data at index {}", input_index),
                })
        }

        async fn get_credentials(&self, _name: &str) -> Result<std::collections::HashMap<String, GenericValue>, BarqError> {
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
        assert_eq!(result[0].len(), 2); // Both items pass because value1(10) > value2(8)
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
        assert_eq!(result[0].len(), 1); // "A" routed to case0
        assert_eq!(result[1].len(), 1); // "B" routed to case1
        assert_eq!(result[2].len(), 1); // "C" routed to fallback (index 2)
    }

    #[tokio::test]
    async fn test_merge_node_append() {
        let input1 = vec![INodeExecutionData::new(IDataObject::from(serde_json::json!({"id": 1})))];
        let input2 = vec![INodeExecutionData::new(IDataObject::from(serde_json::json!({"id": 2})))];
        
        let mut context = MockContext::new(vec![input1, input2]);
        context.add_param("mode", serde_json::json!("append"));
        
        let node = MergeNode;
        let result = node.execute(&context).await.unwrap();
        
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].len(), 2);
    }
}
