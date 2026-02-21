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

        // Check if this is an expression (simplified check - in real implementation would use Rhai)
        if let Some(expr_str) = raw_value.0.as_str() {
            if expr_str.starts_with("{{") && expr_str.ends_with("}}") {
                // For now, return a simple placeholder
                // Full expression evaluation would be implemented with Rhai in a later phase
                return Ok(GenericValue(serde_json::json!({
                    "expression": expr_str,
                    "note": "Expression evaluation not yet implemented - will use Rhai in Phase 12"
                })));
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

    /// Read data from incoming branches.
    fn get_input_data(&self, _input_index: usize) -> Result<&Vec<INodeExecutionData>, BarqError> {
        // We need to return a reference, but we have an Arc<RwLock<>>
        // For now, we'll clone the data - in production this would need a different approach
        // or the trait would need to be redesigned to handle async access
        Err(BarqError::NodeOperationError {
            node_name: self.node.name.clone(),
            message: "Synchronous input data access not implemented - use async variant".to_string(),
        })
    }

    /// Log a debug message scoped to this node execution.
    fn log(&self, message: &str) {
        let span = span!(
            Level::DEBUG,
            "node_execution",
            run_id = %self.run_id,
            node = %self.node.name,
            node_type = %self.node.r#type
        );

        let _enter = span.enter();
        debug!("{}", message);
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
            GenericValue(json!("{{ $json.someValue }}")),
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

        let context = NodeExecutionContext::new(
            node,
            ITaskDataConnections::default(),
            None,
            uuid::Uuid::new_v4(),
        );

        let result = context
            .get_node_parameter("exprParam", None)
            .await
            .unwrap();

        // Should return placeholder indicating expression not yet fully evaluated
        assert!(result.0.is_object());
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
