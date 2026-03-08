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
use futures::FutureExt;
use tokio_util::sync::CancellationToken;
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
    /// Optional execution id persisted in DB for status updates.
    pub execution_id: Option<uuid::Uuid>,
    /// Optional cancellation token for runtime stop requests.
    pub cancellation_token: Option<CancellationToken>,
}

/// The core workflow execution engine.
///
/// Walks the workflow graph and executes nodes in topological order.
pub struct WorkflowRunner {
    /// Node registry for resolving node types
    registry: Arc<NodeRegistry>,
    /// Execution configuration
    config: ExecutionConfig,
    /// Optional runtime credential resolver
    credential_provider: Option<Arc<dyn crate::context::CredentialProvider>>,
}

impl WorkflowRunner {
    fn cancellation_error(context: &WorkflowRunContext) -> BarqError {
        BarqError::ExecutionCancelledError {
            execution_id: context
                .execution_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| context.run_id.to_string()),
        }
    }

    fn is_cancelled(context: &WorkflowRunContext) -> bool {
        context
            .cancellation_token
            .as_ref()
            .map(|token| token.is_cancelled())
            .unwrap_or(false)
    }

    /// Create a new workflow runner.
    ///
    /// # Arguments
    /// * `registry` - Node registry for resolving node types
    /// * `config` - Execution configuration
    pub fn new(registry: Arc<NodeRegistry>, config: ExecutionConfig) -> Self {
        Self {
            registry,
            config,
            credential_provider: None,
        }
    }

    pub fn with_credential_provider(
        mut self,
        credential_provider: Arc<dyn crate::context::CredentialProvider>,
    ) -> Self {
        self.credential_provider = Some(credential_provider);
        self
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

        if Self::is_cancelled(&context) {
            return Err(Self::cancellation_error(&context));
        }

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

        // Group nodes into parallel executable layers
        let mut node_layers: HashMap<NodeIndex, usize> = HashMap::new();
        let mut max_layer = 0;
        
        for &node_index in &execution_order {
            let parents = GraphTraversal::get_parents(&parsed.graph, node_index);
            let layer = parents.iter()
                .filter_map(|p| node_layers.get(p))
                .max()
                .map_or(0, |max_p_layer| max_p_layer + 1);
                
            node_layers.insert(node_index, layer);
            max_layer = max_layer.max(layer);
        }

        let mut layers: Vec<Vec<NodeIndex>> = vec![Vec::new(); max_layer + 1];
        for (&node_index, &layer) in &node_layers {
            layers[layer].push(node_index);
        }

        // Execute layers sequentially, but nodes within a layer concurrently
        let mut results = HashMap::new();
        let mut data_cache: HashMap<NodeId, NodeExecutionResult> = HashMap::new();
        let workflow_cache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        for mut layer in layers {
            if Self::is_cancelled(&context) {
                return Err(Self::cancellation_error(&context));
            }

            // Sort layer to run deterministically
            layer.sort_by_key(|n| execution_order.iter().position(|x| x == n).unwrap());

            // 1. Gather inputs sequentially for the layer where data_cache is safely readable
            let mut layer_inputs = Vec::new();
            
            for &node_index in &layer {
                let node = &parsed.graph[node_index];

                if let Some(inode) = context.workflow.nodes.iter().find(|n| n.id == node.id) {
                    if inode.disabled {
                        debug!("Skipping disabled node: {}", node.name);
                        layer_inputs.push((node_index, None));
                        continue;
                    }
                }

                let input_data = self.gather_input_data(&parsed, node_index, &data_cache).await?;

                let has_incoming_edges = parsed
                    .graph
                    .edges_directed(node_index, petgraph::Direction::Incoming)
                    .next()
                    .is_some();

                if has_incoming_edges {
                    let total_items: usize = input_data.0.values().map(|v| v.len()).sum();
                    if total_items == 0 {
                        debug!("Skipping node '{}' because all upstream branches yielded 0 items", node.name);
                        let result = NodeExecutionResult {
                            node_name: node.name.clone(),
                            outputs: vec![],
                            success: true,
                            error: None,
                        };
                        layer_inputs.push((node_index, Some((input_data, Some(result)))));
                        continue;
                    }
                }

                layer_inputs.push((node_index, Some((input_data, None))));
            }

            // 2. Prepare futures for concurrent execution
            let mut futures = Vec::new();

            for (node_index, input_opt) in layer_inputs {
                if Self::is_cancelled(&context) {
                    return Err(Self::cancellation_error(&context));
                }

                match input_opt {
                    None => {
                        // Skip disabled entirely (no future)
                    },
                    Some((input_data, Some(early_result))) => {
                        // Dead branch, return immediately
                        let fut = async move { Ok((node_index, input_data, early_result)) };
                        futures.push(fut.boxed());
                    },
                    Some((input_data, None)) => {
                        let context_ref = &context;
                        let parsed_ref = &parsed;
                        let node = &parsed.graph[node_index];
                        let workflow_cache_clone = Arc::clone(&workflow_cache);
                        
                        // Execute the node
                        let fut = async move {
                            match self.run_node(context_ref, node, input_data.clone(), parsed_ref, workflow_cache_clone).await {
                                Ok(res) => Ok((node_index, input_data, res)),
                                Err(e) => Err((node_index, input_data, e))
                            }
                        };
                        futures.push(fut.boxed());
                    }
                }
            }

            // 3. Execute all valid nodes in this layer in parallel
            let layer_results = futures::future::join_all(futures).await;

            // 4. Update state caches from results sequentially
            for execute_result in layer_results {
                let (node_index, result) = match execute_result {
                    Ok((n_idx, _in_data, res)) => (n_idx, res),
                    Err((n_idx, in_data, BarqError::SuspendExecution { node_name: _, wait_config })) => {
                        let node = &parsed.graph[n_idx];
                        info!("Execution suspended at node '{}'", node.name);
                        
                        let config: crate::checkpoint::WaitConfig = serde_json::from_value(wait_config)
                            .unwrap_or(crate::checkpoint::WaitConfig {
                                wait_type: crate::checkpoint::WaitType::Time,
                                duration_ms: None,
                                webhook_path: None,
                                external_id: None,
                            });
                            
                        let mut manager = crate::checkpoint::CheckpointManager::with_filesystem(
                            std::env::temp_dir().join("barqflow_checkpoints")
                        );
                        
                        use crate::checkpoint::ExecutionCheckpointBuilder;
                        let checkpoint = ExecutionCheckpointBuilder::new()
                            .with_run_id(context.run_id)
                            .with_workflow_id(context.workflow.id.0.to_string())
                            .with_node_index(n_idx.index())
                            .with_node_data(serde_json::to_value(&in_data).unwrap_or(serde_json::Value::Null))
                            .with_wait_config(config)
                            .build();
                            
                        if let Ok(cp) = checkpoint {
                            let _ = manager.save_checkpoint(cp).await;
                        }

                        // For now, break the loop and return what we have computed so far.
                        return Ok(results);
                    },
                    Err((_n_idx, _in_data, e)) => {
                        if let Some(error_workflow_id) = context.workflow.settings.error_workflow.clone() {
                            return Err(BarqError::TriggerErrorWorkflow { error_workflow_id, original_error: e.to_string() });
                        }
                        return Err(e);
                    }
                };
                
                let node = &parsed.graph[node_index];
                
                if let Some(first_output) = result.outputs.first() {
                    let json_items: Vec<serde_json::Value> = first_output
                        .iter()
                        .map(|item| serde_json::Value::Object(item.json.0.clone()))
                        .collect();
                    workflow_cache.write().await.insert(node.name.clone(), json_items);
                }
                
                data_cache.insert(node.id.clone(), result.clone());
                results.insert(node.name.clone(), result);
            }
        }

        info!("Workflow execution completed");
        Ok(results)
    }

    /// Resume a suspended workflow.
    ///
    /// # Arguments
    /// * `context` - The workflow run context
    /// * `checkpoint` - The loaded checkpoint
    pub async fn resume_workflow(
        &self,
        context: WorkflowRunContext,
        checkpoint: crate::checkpoint::ExecutionCheckpoint,
    ) -> Result<HashMap<String, NodeExecutionResult>, BarqError> {
        info!("Resuming workflow execution for run {}", checkpoint.run_id);

        if Self::is_cancelled(&context) {
            return Err(Self::cancellation_error(&context));
        }

        let flow_workflow = self.convert_workflow(&context.workflow);
        let parsed = WorkflowToGraphParser::parse(&flow_workflow).map_err(|e| {
            BarqError::WorkflowConfigurationError { message: e.to_string() }
        })?;

        let execution_order = GraphTraversal::topological_sort(&parsed.graph).map_err(|e| {
            BarqError::WorkflowConfigurationError { message: e.to_string() }
        })?;

        // Group nodes into parallel executable layers for resuming as well
        let mut node_layers: HashMap<NodeIndex, usize> = HashMap::new();
        let mut max_layer = 0;
        
        for &node_index in &execution_order {
            let parents = GraphTraversal::get_parents(&parsed.graph, node_index);
            let layer = parents.iter()
                .filter_map(|p| node_layers.get(p))
                .max()
                .map_or(0, |max_p_layer| max_p_layer + 1);
                
            node_layers.insert(node_index, layer);
            max_layer = max_layer.max(layer);
        }

        let mut layers: Vec<Vec<NodeIndex>> = vec![Vec::new(); max_layer + 1];
        for (&node_index, &layer) in &node_layers {
            layers[layer].push(node_index);
        }

        let mut results = HashMap::new();
        let mut data_cache: HashMap<NodeId, NodeExecutionResult> = HashMap::new();
        let workflow_cache = Arc::new(tokio::sync::RwLock::new(HashMap::new()));

        let mut resume_started = false;

        for mut layer in layers {
            if Self::is_cancelled(&context) {
                return Err(Self::cancellation_error(&context));
            }

            layer.sort_by_key(|n| execution_order.iter().position(|x| x == n).unwrap());

            let mut layer_inputs = Vec::new();

            for &node_index in &layer {
                let node = &parsed.graph[node_index];

                if !resume_started {
                    if node_index.index() == checkpoint.current_node_index {
                        info!("Resuming at node {}", node.name);
                        resume_started = true;
                        
                        let output_data: barqflow_core::schema::ITaskDataConnections = 
                            serde_json::from_value(checkpoint.node_data.clone())
                            .unwrap_or_default();
                        
                        let mut outputs = Vec::new();
                        if let Some(items) = output_data.0.get(&0) {
                            outputs.push(items.clone());
                        } else if let Some(items) = output_data.0.values().next() {
                            outputs.push(items.clone());
                        } else {
                            outputs.push(vec![]);
                        }

                        let res = NodeExecutionResult {
                            node_name: node.name.clone(),
                            outputs,
                            success: true,
                            error: None,
                        };
                        
                        if let Some(first_output) = res.outputs.first() {
                            let json_items: Vec<serde_json::Value> = first_output
                                .iter()
                                .map(|item| serde_json::Value::Object(item.json.0.clone()))
                                .collect();
                            workflow_cache.write().await.insert(res.node_name.clone(), json_items);
                        }
                        
                        data_cache.insert(node.id.clone(), res.clone());
                        results.insert(node.name.clone(), res);
                    }
                    continue;
                }

                // Normal execution loop from here on
                let input_data = self.gather_input_data(&parsed, node_index, &data_cache).await?;
                
                let has_incoming_edges = parsed.graph.edges_directed(node_index, petgraph::Direction::Incoming).next().is_some();

                if has_incoming_edges {
                    let total_items: usize = input_data.0.values().map(|v| v.len()).sum();
                    if total_items == 0 {
                        let result = NodeExecutionResult {
                            node_name: node.name.clone(),
                            outputs: vec![],
                            success: true,
                            error: None,
                        };
                        layer_inputs.push((node_index, Some((input_data, Some(result)))));
                        continue;
                    }
                }

                layer_inputs.push((node_index, Some((input_data, None))));
            }

            let mut futures = Vec::new();

            for (node_index, input_opt) in layer_inputs {
                if Self::is_cancelled(&context) {
                    return Err(Self::cancellation_error(&context));
                }

                match input_opt {
                    None => {},
                    Some((input_data, Some(early_result))) => {
                        let fut = async move { Ok((node_index, input_data, early_result)) };
                        futures.push(fut.boxed());
                    },
                    Some((input_data, None)) => {
                        let context_ref = &context;
                        let parsed_ref = &parsed;
                        let node = &parsed.graph[node_index];
                        let workflow_cache_clone = Arc::clone(&workflow_cache);
                        
                        let fut = async move {
                            match self.run_node(context_ref, node, input_data.clone(), parsed_ref, workflow_cache_clone).await {
                                Ok(res) => Ok((node_index, input_data, res)),
                                Err(e) => Err((node_index, input_data, e))
                            }
                        };
                        futures.push(fut.boxed());
                    }
                }
            }

            let layer_results = futures::future::join_all(futures).await;

            for execute_result in layer_results {
                let (node_index, result) = match execute_result {
                    Ok((n_idx, _in_data, res)) => (n_idx, res),
                    Err((n_idx, in_data, BarqError::SuspendExecution { node_name: _, wait_config })) => {
                        let config: crate::checkpoint::WaitConfig = serde_json::from_value(wait_config)
                            .unwrap_or(crate::checkpoint::WaitConfig {
                                wait_type: crate::checkpoint::WaitType::Time,
                                duration_ms: None,
                                webhook_path: None,
                                external_id: None,
                            });
                        let mut manager = crate::checkpoint::CheckpointManager::with_filesystem(
                            std::env::temp_dir().join("barqflow_checkpoints")
                        );
                        use crate::checkpoint::ExecutionCheckpointBuilder;
                        let checkpoint = ExecutionCheckpointBuilder::new()
                            .with_run_id(context.run_id)
                            .with_workflow_id(context.workflow.id.0.to_string())
                            .with_node_index(n_idx.index())
                            .with_node_data(serde_json::to_value(&in_data).unwrap_or(serde_json::Value::Null))
                            .with_wait_config(config)
                            .build();
                        if let Ok(cp) = checkpoint {
                            let _ = manager.save_checkpoint(cp).await;
                        }
                        return Ok(results);
                    },
                    Err((n_idx, in_data, BarqError::ExecuteSubWorkflow { workflow_id, input_data: _ })) => {
                        info!("Execution suspended to call sub-workflow '{}'", workflow_id);
                        
                        let config = crate::checkpoint::WaitConfig {
                            wait_type: crate::checkpoint::WaitType::SubWorkflow,
                            duration_ms: None,
                            webhook_path: None,
                            external_id: Some(workflow_id.clone()),
                        };
                            
                        let mut manager = crate::checkpoint::CheckpointManager::with_filesystem(
                            std::env::temp_dir().join("barqflow_checkpoints")
                        );
                        
                        use crate::checkpoint::ExecutionCheckpointBuilder;
                        let checkpoint = ExecutionCheckpointBuilder::new()
                            .with_run_id(context.run_id)
                            .with_workflow_id(context.workflow.id.0.to_string())
                            .with_node_index(n_idx.index())
                            .with_node_data(serde_json::to_value(&in_data).unwrap_or(serde_json::Value::Null))
                            .with_wait_config(config)
                            .build();
                            
                        if let Ok(cp) = checkpoint {
                            let _ = manager.save_checkpoint(cp).await;
                        }

                        return Err(BarqError::ExecuteSubWorkflow { workflow_id, input_data: serde_json::Value::Null });
                    },
                    Err((_n_idx, _in_data, e)) => {
                        if let Some(error_workflow_id) = context.workflow.settings.error_workflow.clone() {
                            error!("Workflow failed, should trigger error workflow: {}", error_workflow_id);
                            return Err(BarqError::TriggerErrorWorkflow {
                                error_workflow_id,
                                original_error: e.to_string(),
                            });
                        }
                        return Err(e);
                    },
                };

                let node = &parsed.graph[node_index];
                
                if let Some(first_output) = result.outputs.first() {
                    let json_items: Vec<serde_json::Value> = first_output
                        .iter()
                        .map(|item| serde_json::Value::Object(item.json.0.clone()))
                        .collect();
                    workflow_cache.write().await.insert(node.name.clone(), json_items);
                }

                data_cache.insert(node.id.clone(), result.clone());
                results.insert(node.name.clone(), result);
            }
        }

        info!("Resumed workflow execution completed");
        Ok(results)
    }

    /// Convert core schema WorkflowDef to flow crate WorkflowDef.
    fn convert_workflow(&self, workflow: &CoreWorkflowDef) -> WorkflowDef {
        let nodes = workflow
            .nodes
            .iter()
            .map(|n| WorkflowNode::from(n.clone()))
            .collect();

        WorkflowDef {
            id: workflow.id.0.to_string(),
            name: workflow.name.clone(),
            nodes,
            connections: workflow.connections.clone(),
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
        workflow_cache: Arc<tokio::sync::RwLock<HashMap<String, Vec<serde_json::Value>>>>,
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
        let exec_context = crate::context::NodeExecutionContext::new_with_credentials(
            inode.clone(),
            input_data,
            context.static_data.clone(),
            context.run_id.0,
            workflow_cache,
            self.credential_provider.clone(),
        );

        let node_continue_on_fail = inode.parameters.0.get("continueOnFail")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
            
        let max_retries = inode.parameters.0.get("retryOnFail")
            .and_then(|v| v.as_bool())
            .and_then(|b| if b { inode.parameters.0.get("maxTries").and_then(|v| v.as_u64()).map(|v| v as u32) } else { None })
            .unwrap_or(0);
            
        let retry_interval_ms = inode.parameters.0.get("waitBetweenTries")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let error_handler = crate::error_handler::ErrorHandler::new(
            self.config.continue_on_fail || node_continue_on_fail
        ).with_retries(max_retries, retry_interval_ms);

        let execute_result = error_handler.execute_isolated_async(&node.name, || {
            node_info.node_impl.execute(&exec_context)
        }).await;

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
                
                if error_handler.continue_on_fail {
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
    use barqflow_core::schema::{IConnection, NodeConnectionType};
    use barqflow_core::traits::IExecuteFunctions;
    use barqflow_core::types::WorkflowId;

    use super::*;
    use barqflow_core::schema::{INode, INodeParameters, IWorkflowSettings};
    use barqflow_core::properties::INodeProperties;
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
                credentials: vec![],
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
            execution_id: None,
            cancellation_token: None,
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
            execution_id: None,
            cancellation_token: None,
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
            execution_id: None,
            cancellation_token: None,
        };

        assert!(!context.manual);
        assert_eq!(context.workflow.name, "Test Workflow");
    }

    #[tokio::test]
    async fn test_run_workflow_honors_cancellation_token() {
        let registry = create_mock_registry();
        let runner = WorkflowRunner::new(registry, ExecutionConfig::default());
        let workflow = create_test_workflow();
        let token = CancellationToken::new();
        token.cancel();

        let context = WorkflowRunContext {
            run_id: RunId::new(),
            workflow,
            static_data: None,
            manual: true,
            execution_id: Some(uuid::Uuid::new_v4()),
            cancellation_token: Some(token),
        };

        let err = runner.run_workflow(context).await.unwrap_err();
        match err {
            BarqError::ExecutionCancelledError { .. } => {}
            other => panic!("expected ExecutionCancelledError, got: {}", other),
        }
    }

    #[tokio::test]
    async fn test_convert_workflow_preserves_connections() {
        let registry = create_mock_registry();
        let runner = WorkflowRunner::new(registry, ExecutionConfig::default());

        let source = INode {
            id: NodeId::new("source"),
            name: "Source".to_string(),
            r#type: "mockPassThrough".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: INodeParameters::default(),
            credentials: vec![],
            disabled: false,
        };
        let sink = INode {
            id: NodeId::new("sink"),
            name: "Sink".to_string(),
            r#type: "mockPassThrough".to_string(),
            type_version: 1.0,
            position: [100.0, 0.0],
            parameters: INodeParameters::default(),
            credentials: vec![],
            disabled: false,
        };

        let mut connections = HashMap::new();
        connections.insert(
            "Source".to_string(),
            barqflow_core::schema::INodeConnections(HashMap::from([(
                NodeConnectionType::Main,
                vec![vec![IConnection {
                    node: "Sink".to_string(),
                    r#type: NodeConnectionType::Main,
                    index: 0,
                }]],
            )])),
        );

        let core = CoreWorkflowDef {
            id: WorkflowId::new(),
            name: "Connected Workflow".to_string(),
            nodes: vec![source, sink],
            connections,
            active: true,
            settings: IWorkflowSettings::default(),
        };

        let flow = runner.convert_workflow(&core);
        assert_eq!(flow.connections.len(), 1);

        let parsed = WorkflowToGraphParser::parse(&flow).expect("workflow should parse");
        assert_eq!(parsed.graph.edge_count(), 1);
    }
}
