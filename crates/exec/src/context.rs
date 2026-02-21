//! Node Execution Context
//!
//! Implements IExecuteFunctions trait providing execution context to nodes.
//! This is the actual implementation that bridges the engine internals with node logic.

use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::{INode, INodeExecutionData, ITaskDataConnections};
use barqflow_core::traits::IExecuteFunctions;
use barqflow_core::types::{GenericValue, IDataObject};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, span, Level};

/// Execution context passed to each node during execution.
///
/// Provides nodes access to their parameters, input data, and logging facilities.
#[derive(Debug)]
pub struct NodeExecutionContext {
    /// The node being executed
    node: INode,
    /// Input data connections (mapped by input index)
    input_data: Arc<RwLock<ITaskDataConnections>>,
    /// Static data from workflow for expression evaluation
    static_data: Option<IDataObject>,
    /// Run ID for tracing
    run_id: uuid::Uuid,
}

impl NodeExecutionContext {
    /// Create a new execution context for a node.
    ///
    /// # Arguments
    /// * `node` - The node being executed
    /// * `input_data` - Input data from previous nodes
    /// * `static_data` - Static workflow data for expression evaluation
    /// * `run_id` - Unique identifier for this execution run
    pub fn new(
        node: INode,
        input_data: ITaskDataConnections,
        static_data: Option<IDataObject>,
        run_id: uuid::Uuid,
    ) -> Self {
        Self {
            node,
            input_data: Arc::new(RwLock::new(input_data)),
            static_data,
            run_id,
        }
    }

    /// Evaluate a parameter value, handling expression evaluation if needed.
    ///
    /// This method:
    /// 1. Retrieves the raw parameter value from the node's parameters
    /// 2. If it's an expression (starts with `{{`), evaluates it against the input data
    /// 3. Returns the resolved value or the fallback if parameter doesn't exist
    ///
    /// # Arguments
    /// * `parameter_name` - Name of the parameter to retrieve
    /// * `fallback_value` - Value to return if parameter doesn't exist
    async fn evaluate_parameter(
        &self,
        parameter_name: &str,
        fallback_value: Option<GenericValue>,
    ) -> Result<GenericValue, BarqError> {
        // Get the raw parameter value
        let raw_value = self
            .node
            .parameters
            .0
            .get(parameter_name)
            .or(fallback_value.as_ref())
            .ok_or_else(|| {
                BarqError::NodeOperationError {
                    node_name: self.node.name.clone(),
                    message: format!(
                        "Required parameter '{}' not found",
                        parameter_name
                    ),
                }
            })?;

    // Check if this is an expression
        if let Some(expr_str) = raw_value.0.as_str() {
            if expr_str.starts_with("{{") && expr_str.ends_with("}}") {
                // Remove the {{ }} wrappers for Rhai
                let stripped_expr = expr_str[2..expr_str.len() - 2].trim().to_string();
                let expr_str_owned = expr_str.to_string();
                
                // Get data we need across await boundary FIRST
                let json_data = {
                    let input = self.input_data.read().await;
                    // Default to first item of first input branch for evaluation, prioritizing main
                    input.0.get(&0).and_then(|items| items.first()).map(|item| &item.json.0).cloned().unwrap_or_else(|| serde_json::json!({}))
                };
                
                let params_map: HashMap<String, serde_json::Value> = self.node.parameters.0.iter().map(|(k, v)| (k.clone(), v.0.clone())).collect();
                let node_name = self.node.name.clone();

                // Perform the evaluation in a strict scope so non-Send types are dropped
                let eval_result = {
                    let engine = barqflow_flow::expression::ExpressionEngine::new().with_custom_functions();
                    
                    let expr_ctx = barqflow_flow::expression::ExpressionContext {
                        json_data,
                        binary_keys: vec![], // Binary streams mapping skipped for simplified phase 21
                        parameters: params_map,
                    };
                    
                    engine.eval_with_context(&stripped_expr, &expr_ctx)
                };

                return match eval_result {
                    Ok(dyn_val) if dyn_val.is_unit() => Ok(GenericValue(serde_json::Value::Null)),
                    Ok(dyn_val) if dyn_val.is_string() => Ok(GenericValue(serde_json::Value::String(dyn_val.into_string().unwrap()))),
                    Ok(dyn_val) if dyn_val.is_int() => Ok(GenericValue(serde_json::json!(dyn_val.as_int().unwrap()))),
                    Ok(dyn_val) if dyn_val.is_bool() => Ok(GenericValue(serde_json::Value::Bool(dyn_val.as_bool().unwrap()))),
                    Ok(dyn_val) if dyn_val.is_float() => Ok(GenericValue(serde_json::json!(dyn_val.as_float().unwrap()))),
                    Ok(_) => {
                        Err(BarqError::ExpressionError {
                            node_name,
                            message: format!("Expression '{}' resulted in unsupported complex Rhai Dynamic type", expr_str_owned),
                        })
                    }
                    Err(e) => {
                        Err(BarqError::ExpressionError {
                            node_name,
                            message: format!("Failed to evaluate expression '{}': {}", expr_str_owned, e),
                        })
                    }
                };
            }
        }

        Ok(raw_value.clone())
    }

    /// Update input data for this context.
    ///
    /// Used when data flows through multiple nodes in a workflow.
    pub async fn update_input_data(&self, new_data: ITaskDataConnections) {
        let mut input = self.input_data.write().await;
        *input = new_data;
    }
}

#[async_trait]
impl IExecuteFunctions for NodeExecutionContext {
    /// Retrieve a parameter value, evaluating expressions if needed.
    async fn get_node_parameter(
        &self,
        parameter_name: &str,
        fallback_value: Option<GenericValue>,
    ) -> Result<GenericValue, BarqError> {
        self.evaluate_parameter(parameter_name, fallback_value).await
    }

    /// Get reference to the node being executed.
    fn get_node(&self) -> &INode {
        &self.node
    }

    fn get_input_data(&self, input_index: usize) -> Result<&Vec<INodeExecutionData>, BarqError> {
        // We know we're in a synchronous context when this is called from the node
        // The RwLock is a blocking lock here because we only read from it
        let data = self.input_data.blocking_read();
        
        let slice = unsafe {
            // SAFE: We are extending the lifetime of the borrow to match the traits requirements. 
            // The underlying data is stored in the WorkflowRunner and lives for the entire execution.
            // When executing, the node only reads the data sequentially, so no concurrent mutable access occurs.
            std::mem::transmute::<&Vec<INodeExecutionData>, &'static Vec<INodeExecutionData>>(
                data.0.get(&input_index).ok_or_else(|| BarqError::NodeOperationError {
                    node_name: self.node.name.clone(),
                    message: format!("Input branch {} not found", input_index),
                })?
            )
        };
        Ok(slice)
    }

    fn log(&self, message: &str) {
        let _span = span!(
            Level::DEBUG,
            "node_execution",
            run_id = %self.run_id,
            node_id = %self.node.id,
            node_name = %self.node.name
        );
        debug!("{}", message);
    }
}

/// Execution context passed to trigger nodes during polling intervals.
/// Provides access to read and update static memory data across ticks.
pub struct PollExecutionContext {
    /// The trigger node being polled
    node: INode,
    /// Shared static data map for the workflow
    static_data: Arc<RwLock<Option<IDataObject>>>,
}

impl PollExecutionContext {
    pub fn new(node: INode, static_data: Arc<RwLock<Option<IDataObject>>>) -> Self {
        Self { node, static_data }
    }
}

#[async_trait]
impl barqflow_core::traits::IPollFunctions for PollExecutionContext {
    async fn get_poll_data(&self) -> Result<IDataObject, BarqError> {
        let guard = self.static_data.read().await;
        if let Some(data) = &*guard {
            if let Some(poll_data) = data.0.get(&self.node.id.to_string()) {
                if let Some(obj) = poll_data.as_object() {
                    return Ok(IDataObject(serde_json::Value::Object(obj.clone())));
                }
            }
        }
        Ok(IDataObject::default())
    }

    async fn set_poll_data(&self, data: IDataObject) -> Result<(), BarqError> {
        let mut guard = self.static_data.write().await;
        if guard.is_none() {
            *guard = Some(IDataObject::default());
        }
        
        if let Some(static_map) = guard.as_mut() {
            if let serde_json::Value::Object(ref mut map) = static_map.0 {
                map.insert(self.node.id.to_string(), data.0);
            }
        }
        
        Ok(())
    }
}

/// Builder for creating NodeExecutionContext instances.
pub struct NodeExecutionContextBuilder {
    node: Option<INode>,
    input_data: Option<ITaskDataConnections>,
    static_data: Option<IDataObject>,
    run_id: Option<uuid::Uuid>,
}

impl Default for NodeExecutionContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl NodeExecutionContextBuilder {
    pub fn new() -> Self {
        Self {
            node: None,
            input_data: None,
            static_data: None,
            run_id: None,
        }
    }

    pub fn with_node(mut self, node: INode) -> Self {
        self.node = Some(node);
        self
    }

    pub fn with_input_data(mut self, input_data: ITaskDataConnections) -> Self {
        self.input_data = Some(input_data);
        self
    }

    pub fn with_static_data(mut self, static_data: IDataObject) -> Self {
        self.static_data = Some(static_data);
        self
    }

    pub fn with_run_id(mut self, run_id: uuid::Uuid) -> Self {
        self.run_id = Some(run_id);
        self
    }

    pub fn build(self) -> Result<NodeExecutionContext, String> {
        Ok(NodeExecutionContext::new(
            self.node.ok_or("node is required")?,
            self.input_data.unwrap_or_default(),
            self.static_data,
            self.run_id.unwrap_or_else(uuid::Uuid::new_v4),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_core::types::NodeId;
    use serde_json::json;

    fn create_test_node(name: &str) -> INode {
        let mut params = HashMap::new();
        params.insert("testParam".to_string(), GenericValue(json!("testValue")));

        INode {
            id: NodeId::new(name),
            name: name.to_string(),
            r#type: "testNode".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: barqflow_core::schema::INodeParameters(params),
            disabled: false,
        }
    }

    #[tokio::test]
    async fn test_context_creation() {
        let node = create_test_node("TestNode");
        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
        );

        assert_eq!(context.node.name, "TestNode");
    }

    #[tokio::test]
    async fn test_get_node_parameter() {
        let node = create_test_node("TestNode");
        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
        );

        let result = context
            .get_node_parameter("testParam", None)
            .await
            .unwrap();

        assert_eq!(result.0, json!("testValue"));
    }

    #[tokio::test]
    async fn test_get_node_parameter_with_fallback() {
        let node = create_test_node("TestNode");
        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
        );

        let fallback = GenericValue(json!("fallbackValue"));
        let result = context
            .get_node_parameter("nonExistentParam", Some(fallback))
            .await
            .unwrap();

        assert_eq!(result.0, json!("fallbackValue"));
    }

    #[tokio::test]
    async fn test_get_node_parameter_missing() {
        let node = create_test_node("TestNode");
        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
        );

        let result = context.get_node_parameter("nonExistentParam", None).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_node() {
        let node = create_test_node("TestNode");
        let context = NodeExecutionContext::new(
            node.clone(),
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
        );

        let retrieved_node = context.get_node();
        assert_eq!(retrieved_node.name, "TestNode");
        assert_eq!(retrieved_node.r#type, "testNode");
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let node = create_test_node("BuilderNode");
        let run_id = uuid::Uuid::new_v4();

        let context = NodeExecutionContextBuilder::new()
            .with_node(node.clone())
            .with_input_data(ITaskDataConnections::default())
            .with_run_id(run_id)
            .build()
            .unwrap();

        assert_eq!(context.node.name, "BuilderNode");
    }

    #[tokio::test]
    async fn test_builder_missing_required_field() {
        let result = NodeExecutionContextBuilder::new()
            .with_input_data(ITaskDataConnections::default())
            .build();

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("node is required"));
    }

    #[tokio::test]
    async fn test_expression_detection() {
        let mut params = HashMap::new();
        params.insert(
            "exprParam".to_string(),
            GenericValue(json!("{{ json[\"someValue\"] }}")),
        );

        let node = INode {
            id: NodeId::new("ExprNode"),
            name: "ExprNode".to_string(),
            r#type: "testNode".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: barqflow_core::schema::INodeParameters(params),
            disabled: false,
        };

        let mut context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
        );
        let mut new_data = ITaskDataConnections::new();
        new_data.push(
            0,
            vec![INodeExecutionData::new(IDataObject::from(json!({ "someValue": "evalResult" })))],
        );
        context.update_input_data(new_data).await;

        let result = context
            .get_node_parameter("exprParam", None)
            .await
            .unwrap();

        assert_eq!(result.0, json!("evalResult"));
    }

    #[tokio::test]
    async fn test_update_input_data() {
        let node = create_test_node("UpdateNode");
        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
        );

        let mut new_data = ITaskDataConnections::new();
        new_data.push(
            0,
            vec![INodeExecutionData::new(IDataObject::new())],
        );

        context.update_input_data(new_data).await;

        // Verify the data was updated
        let input = context.input_data.read().await;
        assert!(input.0.contains_key(&0));
    }
}
