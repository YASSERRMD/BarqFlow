//! Workflow Execution Engine
//!
//! Implements the core execution engine that walks the workflow graph
//! and executes nodes in topological order.

use barqflow_core::errors::BarqError;
use barqflow_core::schema::{
    INodeExecutionData, ITaskDataConnections, WorkflowDef as CoreWorkflowDef,
};
use barqflow_core::types::{IDataObject, NodeId, RunId};
use barqflow_flow::graph::{GraphTraversal, ParsedGraph, WorkflowDef, WorkflowNode};
use barqflow_registry::registry::NodeRegistry;
use petgraph::graph::NodeIndex;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, error, info, instrument};

/// Configuration for workflow execution.
#[derive(Debug, Clone)]
pub struct ExecutionConfig {
    /// Whether to continue execution even when a node fails
    pub continue_on_fail: bool,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: Option<u64>,
    /// Whether to save execution progress
    pub save_execution_progress: bool,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            continue_on_fail: false,
            max_execution_time_ms: None,
            save_execution_progress: true,
        }
    }
}

/// Result of executing a single node.
#[derive(Debug, Clone)]
pub struct NodeExecutionResult {
    /// The node that was executed
    pub node_name: String,
    /// Output data from the node (one entry per output index)
    pub outputs: Vec<Vec<INodeExecutionData>>,
    /// Whether the node succeeded
    pub success: bool,
    /// Error message if the node failed
    pub error: Option<String>,
}

/// Context for a workflow execution run.
#[derive(Debug, Clone)]
pub struct WorkflowRunContext {
    /// Unique identifier for this run
    pub run_id: RunId,
    /// The workflow being executed (core schema type)
    pub workflow: CoreWorkflowDef,
    /// Static data available for expression evaluation
    pub static_data: Option<IDataObject>,
    /// Whether this is a manual execution
    pub manual: bool,
}

/// The core workflow execution engine.
///
/// Walks the workflow graph and executes nodes in topological order.
pub struct WorkflowRunner {
    /// Node registry for resolving node types
    registry: Arc<NodeRegistry>,
    /// Execution configuration
    config: ExecutionConfig,
}

impl WorkflowRunner {
    /// Create a new workflow runner.
    ///
    /// # Arguments
    /// * `registry` - Node registry for resolving node types
    /// * `config` - Execution configuration
    pub fn new(registry: Arc<NodeRegistry>, config: ExecutionConfig) -> Self {
        Self { registry, config }
    }

    /// Execute a workflow.
    ///
    /// # Arguments
    /// * `context` - The workflow run context
    ///
    /// # Returns
    /// A map of node name to execution result
    #[instrument(skip(self, context), fields(run_id = %context.run_id, workflow = %context.workflow.name))]
    pub async fn run_workflow(
        &self,
        context: WorkflowRunContext,
    ) -> Result<HashMap<String, NodeExecutionResult>, BarqError> {
        info!("Starting workflow execution");

        // Convert core workflow to flow workflow
        let flow_workflow = self.convert_workflow(&context.workflow);

        // Parse the workflow into a graph
        let parsed = WorkflowToGraphParser::parse(&flow_workflow).map_err(|e| {
            BarqError::WorkflowConfigurationError {
                message: format!("Failed to parse workflow: {}", e),
            }
        })?;

        // Check for cycles
        if !GraphTraversal::is_executable_dag(&parsed.graph) {
            return Err(BarqError::WorkflowConfigurationError {
                message: "Workflow contains cycles and cannot be executed".to_string(),
            });
        }

        // Get topological order
        let execution_order = GraphTraversal::topological_sort(&parsed.graph).map_err(|e| {
            BarqError::WorkflowConfigurationError {
                message: format!("Failed to determine execution order: {}", e),
            }
        })?;

        info!(
            "Executing {} nodes in topological order",
            execution_order.len()
        );

        // Execute nodes in order
        let mut results = HashMap::new();
        let mut data_cache: HashMap<NodeId, NodeExecutionResult> = HashMap::new();

        for node_index in execution_order {
            let node = &parsed.graph[node_index];

            // Skip disabled nodes
            if let Some(inode) = context.workflow.nodes.iter().find(|n| n.id == node.id) {
                if inode.disabled {
                    debug!("Skipping disabled node: {}", node.name);
                    continue;
                }
            }

            // Gather input data from predecessors
            let input_data = self
                .gather_input_data(&parsed, node_index, &data_cache)
                .await?;

            // Build workflow_cache for this node execution
            let mut workflow_cache_map = HashMap::new();
            for (_, res) in &data_cache {
                if let Some(first_output) = res.outputs.first() {
                    let json_items: Vec<serde_json::Value> = first_output
                        .iter()
                        .map(|item| serde_json::Value::Object(item.json.0.clone()))
                        .collect();
                    workflow_cache_map.insert(res.node_name.clone(), json_items);
                }
            }
            let workflow_cache = Arc::new(workflow_cache_map);

            // Execute the node
            let result = self.run_node(&context, node, input_data, &parsed, workflow_cache).await?;

            let node_id = node.id.clone();
            let node_name = node.name.clone();
            data_cache.insert(node_id, result.clone());
            results.insert(node_name, result);
        }

        info!("Workflow execution completed");
        Ok(results)
    }

    /// Convert core schema WorkflowDef to flow crate WorkflowDef.
    fn convert_workflow(&self, workflow: &CoreWorkflowDef) -> WorkflowDef {
        let nodes = workflow
            .nodes
            .iter()
            .map(|n| WorkflowNode::from(n.clone()))
            .collect();

        let connections: std::collections::HashMap<
            String,
            barqflow_core::schema::INodeConnections,
        > = std::collections::HashMap::new();

        WorkflowDef {
            id: workflow.id.0.to_string(),
            name: workflow.name.clone(),
            nodes,
            connections,
            settings: Some(workflow.settings.clone()),
            static_data: None,
            pin_data: None,
            version_id: None,
        }
    }

    /// Gather input data for a node from its predecessors.
    async fn gather_input_data(
        &self,
        parsed: &ParsedGraph,
        node_index: NodeIndex,
        data_cache: &HashMap<NodeId, NodeExecutionResult>,
    ) -> Result<ITaskDataConnections, BarqError> {
        use petgraph::visit::EdgeRef;
        let mut input_data = ITaskDataConnections::new();

        // Get all incoming edges to this node
        let edges = parsed
            .graph
            .edges_directed(node_index, petgraph::Direction::Incoming);

        for edge in edges {
            let parent_node = &parsed.graph[edge.source()];
            let out_idx = edge.weight().source_output_index;
            let in_idx = edge.weight().target_input_index;

            if let Some(parent_result) = data_cache.get(&parent_node.id) {
                if let Some(output_data) = parent_result.outputs.get(out_idx) {
                    // Push will append if multiple connections map to the same input index
                    input_data.push(in_idx, output_data.clone());
                }
            }
        }

        Ok(input_data)
    }

    /// Execute a single node.
    ///
    /// # Arguments
    /// * `context` - The workflow run context
    /// * `node` - The node to execute
    /// * `input_data` - Input data from previous nodes
    /// * `parsed` - The parsed workflow graph
    #[instrument(skip(self, context, node, input_data, _parsed), fields(node = %node.name))]
    async fn run_node(
        &self,
        context: &WorkflowRunContext,
        node: &WorkflowNode,
        input_data: ITaskDataConnections,
        _parsed: &ParsedGraph,
        workflow_cache: Arc<HashMap<String, Vec<serde_json::Value>>>,
    ) -> Result<NodeExecutionResult, BarqError> {
        debug!("Executing node: {} (type: {})", node.name, node.type_);

        // Find the INode for this WorkflowNode
        let inode = context
            .workflow
            .nodes
            .iter()
            .find(|n| n.id == node.id)
            .ok_or_else(|| BarqError::NodeOperationError {
                node_name: node.name.clone(),
                message: "Node not found in workflow definition".to_string(),
            })?;

        // Resolve the node implementation from the registry
        let node_info = self
            .registry
            .get_node_by_name_with_fallback(&inode.r#type, inode.type_version)
            .ok_or_else(|| BarqError::NodeOperationError {
                node_name: node.name.clone(),
                message: format!(
                    "Node type '{}' version {} not found in registry",
                    inode.r#type, inode.type_version
                ),
            })?;

        // Create execution context
        let exec_context = crate::context::NodeExecutionContext::new(
            inode.clone(),
            input_data,
            context.static_data.clone(),
            context.run_id.0,
            workflow_cache,
        );

        // Execute the node
        let execute_result = node_info.node_impl.execute(&exec_context).await;

        match execute_result {
            Ok(outputs) => {
                debug!(
                    "Node {} executed successfully, produced {} output streams",
                    node.name,
                    outputs.len()
                );
                Ok(NodeExecutionResult {
                    node_name: node.name.clone(),
                    outputs,
                    success: true,
                    error: None,
                })
            }
            Err(e) => {
                error!("Node {} failed: {}", node.name, e);
                if self.config.continue_on_fail {
                    Ok(NodeExecutionResult {
                        node_name: node.name.clone(),
                        outputs: vec![],
                        success: false,
                        error: Some(e.to_string()),
                    })
                } else {
                    Err(e)
                }
            }
        }
    }
}

/// Re-export parser from flow crate
pub use barqflow_flow::graph::WorkflowToGraphParser;

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use barqflow_core::traits::IExecuteFunctions;
    use barqflow_core::types::WorkflowId;

    use super::*;
    use barqflow_core::schema::{INode, INodeParameters, IWorkflowSettings};
    use barqflow_registry::node_properties::INodeProperties;
    use serde_json::json;
    use std::sync::Arc;

    // Mock node implementation for testing
    struct MockPassThroughNode;

    #[async_trait]
    impl barqflow_core::traits::INodeType for MockPassThroughNode {
        fn get_description(&self) -> IDataObject {
            IDataObject::from(json!({
                "name": "mockPassThrough",
                "displayName": "Mock Pass Through",
                "description": "A mock node that passes through data"
            }))
        }

        async fn execute(
            &self,
            _context: &dyn IExecuteFunctions,
        ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
            // Return mock output data
            let output = INodeExecutionData::new(IDataObject::from(json!({
                "result": "mock_output"
            })));
            Ok(vec![vec![output]])
        }
    }

    fn create_mock_registry() -> Arc<NodeRegistry> {
        let registry = Arc::new(NodeRegistry::new());

        // Register mock node type
        let node_info = barqflow_registry::registry::NodeInfo {
            name: "mockPassThrough".to_string(),
            display_name: "Mock Pass Through".to_string(),
            version: 1.0,
            description: "A mock node for testing".to_string(),
            properties: INodeProperties {
                display_name: Some("Mock Properties".to_string()),
                properties: vec![],
                required_values: None,
            },
            is_trigger: false,
            max_inputs: 1,
            node_impl: Arc::new(MockPassThroughNode),
        };

        registry.register_node(node_info).unwrap();
        registry
    }

    fn create_test_workflow() -> CoreWorkflowDef {
        CoreWorkflowDef {
            id: WorkflowId::new(),
            name: "Test Workflow".to_string(),
            nodes: vec![INode {
                id: NodeId::new("node1"),
                name: "TestNode".to_string(),
                r#type: "mockPassThrough".to_string(),
                type_version: 1.0,
                position: [0.0, 0.0],
                parameters: INodeParameters::default(),
                disabled: false,
            }],
            connections: HashMap::new(),
            active: true,
            settings: IWorkflowSettings::default(),
        }
    }

    #[tokio::test]
    async fn test_runner_creation() {
        let registry = create_mock_registry();
        let config = ExecutionConfig::default();
        let runner = WorkflowRunner::new(registry, config);

        assert_eq!(runner.config.continue_on_fail, false);
    }

    #[tokio::test]
    async fn test_run_simple_workflow() {
        let registry = create_mock_registry();
        let config = ExecutionConfig::default();
        let runner = WorkflowRunner::new(registry, config);

        let workflow = create_test_workflow();
        let context = WorkflowRunContext {
            run_id: RunId::new(),
            workflow,
            static_data: None,
            manual: true,
        };

        let results = runner.run_workflow(context).await.unwrap();

        assert_eq!(results.len(), 1);
        assert!(results.contains_key("TestNode"));

        let result = &results["TestNode"];
        assert!(result.success);
        assert_eq!(result.outputs.len(), 1);
    }

    #[tokio::test]
    async fn test_run_with_continue_on_fail() {
        let registry = create_mock_registry();
        let config = ExecutionConfig {
            continue_on_fail: true,
            ..Default::default()
        };
        let runner = WorkflowRunner::new(registry, config);

        let workflow = create_test_workflow();
        let context = WorkflowRunContext {
            run_id: RunId::new(),
            workflow,
            static_data: None,
            manual: true,
        };

        let results = runner.run_workflow(context).await.unwrap();
        assert_eq!(results.len(), 1);
    }

    #[tokio::test]
    async fn test_execution_config_default() {
        let config = ExecutionConfig::default();
        assert!(!config.continue_on_fail);
        assert!(config.max_execution_time_ms.is_none());
        assert!(config.save_execution_progress);
    }

    #[tokio::test]
    async fn test_workflow_run_context_creation() {
        let run_id = RunId::new();
        let workflow = create_test_workflow();
        let context = WorkflowRunContext {
            run_id,
            workflow,
            static_data: None,
            manual: false,
        };

        assert!(!context.manual);
        assert_eq!(context.workflow.name, "Test Workflow");
    }
}
