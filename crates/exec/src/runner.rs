//! Workflow Execution Engine
//!
//! Implements the core execution engine that walks the workflow graph
//! and executes nodes in topological order.

use async_trait::async_trait;
use barqflow_core::contracts::{ExecutionEvent, ExecutionEventType, ExecutionStatus};
use barqflow_core::errors::BarqError;
use barqflow_core::schema::{
    INodeExecutionData, ITaskDataConnections, WorkflowDef as CoreWorkflowDef,
};
use barqflow_core::types::{IDataObject, NodeId, RunId};
use barqflow_flow::graph::{GraphTraversal, ParsedGraph, WorkflowDef, WorkflowNode};
use barqflow_registry::registry::NodeRegistry;
use chrono::Utc;
use futures::FutureExt;
use petgraph::graph::NodeIndex;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::time::timeout;
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

#[async_trait]
pub trait ExecutionEventReporter: Send + Sync {
    async fn report(&self, event: ExecutionEvent);
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
    /// Optional parent execution id for nested/sub-workflow runs.
    pub parent_execution_id: Option<uuid::Uuid>,
    /// Optional cancellation token for runtime stop requests.
    pub cancellation_token: Option<CancellationToken>,
    /// Optional target node id/name for partial execution.
    /// When set, runner executes only target ancestors + target and stops after target runs.
    pub stop_after_node_id: Option<String>,
    /// Monotonic event sequence offset. The next emitted event uses `event_sequence_start + 1`.
    pub event_sequence_start: u64,
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
    /// Optional sub-workflow executor used by Execute Workflow nodes.
    subworkflow_executor: Option<Arc<dyn crate::subworkflow::SubWorkflowExecutor>>,
    /// Optional reporter used to surface lifecycle events while the workflow runs.
    event_reporter: Option<Arc<dyn ExecutionEventReporter>>,
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
            subworkflow_executor: None,
            event_reporter: None,
        }
    }

    pub fn with_credential_provider(
        mut self,
        credential_provider: Arc<dyn crate::context::CredentialProvider>,
    ) -> Self {
        self.credential_provider = Some(credential_provider);
        self
    }

    pub fn with_subworkflow_executor(
        mut self,
        subworkflow_executor: Arc<dyn crate::subworkflow::SubWorkflowExecutor>,
    ) -> Self {
        self.subworkflow_executor = Some(subworkflow_executor);
        self
    }

    pub fn with_event_reporter(mut self, event_reporter: Arc<dyn ExecutionEventReporter>) -> Self {
        self.event_reporter = Some(event_reporter);
        self
    }

    #[allow(clippy::too_many_arguments)]
    async fn emit_execution_event(
        &self,
        context: &WorkflowRunContext,
        sequence: &mut u64,
        event_type: ExecutionEventType,
        status: ExecutionStatus,
        node: Option<&WorkflowNode>,
        message: impl Into<String>,
        data: serde_json::Value,
    ) {
        let Some(event_reporter) = self.event_reporter.as_ref() else {
            return;
        };

        *sequence += 1;
        event_reporter
            .report(ExecutionEvent {
                execution_id: context.execution_id.unwrap_or(context.run_id.0),
                workflow_id: context.workflow.id,
                run_id: context.run_id,
                event_type,
                status,
                node_id: node.map(|current| current.id.clone()),
                node_name: node.map(|current| current.name.clone()),
                message: message.into(),
                timestamp: Utc::now(),
                sequence: *sequence,
                data: IDataObject::from(data),
            })
            .await;
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
        if let Some(max_ms) = self.config.max_execution_time_ms {
            let duration = std::time::Duration::from_millis(max_ms);
            return match timeout(duration, self.run_workflow_inner(context)).await {
                Ok(result) => result,
                Err(_elapsed) => Err(BarqError::WorkflowConfigurationError {
                    message: format!(
                        "Workflow execution exceeded maximum allowed time of {}ms",
                        max_ms
                    ),
                }),
            };
        }
        self.run_workflow_inner(context).await
    }

    async fn run_workflow_inner(
        &self,
        context: WorkflowRunContext,
    ) -> Result<HashMap<String, NodeExecutionResult>, BarqError> {
        info!("Starting workflow execution");
        let mut event_sequence = context.event_sequence_start;

        if Self::is_cancelled(&context) {
            return Err(Self::cancellation_error(&context));
        }

        self.emit_execution_event(
            &context,
            &mut event_sequence,
            ExecutionEventType::Started,
            ExecutionStatus::Running,
            None,
            "Execution started",
            json!({
                "workflowName": context.workflow.name,
                "manual": context.manual,
                "stopAfterNodeId": context.stop_after_node_id,
            }),
        )
        .await;

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

        let scoped_execution: Option<(String, HashSet<NodeIndex>)> =
            if let Some(target) = context.stop_after_node_id.clone() {
                let target_index = execution_order
                    .iter()
                    .copied()
                    .find(|idx| {
                        let node = &parsed.graph[*idx];
                        node.id.to_string() == target || node.name == target
                    })
                    .ok_or_else(|| BarqError::WorkflowConfigurationError {
                        message: format!("Target test node '{}' was not found in workflow", target),
                    })?;

                let mut required_nodes = HashSet::new();
                let mut stack = vec![target_index];
                while let Some(current) = stack.pop() {
                    if !required_nodes.insert(current) {
                        continue;
                    }
                    for parent in GraphTraversal::get_parents(&parsed.graph, current) {
                        stack.push(parent);
                    }
                }

                info!(
                    "Executing partial workflow for target '{}' with {} scoped nodes",
                    target,
                    required_nodes.len()
                );
                Some((target, required_nodes))
            } else {
                None
            };

        // Group nodes into parallel executable layers
        let mut node_layers: HashMap<NodeIndex, usize> = HashMap::new();
        let mut max_layer = 0;

        for &node_index in &execution_order {
            let parents = GraphTraversal::get_parents(&parsed.graph, node_index);
            let layer = parents
                .iter()
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
        let mut stop_reached = false;

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

                if let Some((_, scope)) = &scoped_execution {
                    if !scope.contains(&node_index) {
                        continue;
                    }
                }

                if let Some(inode) = context.workflow.nodes.iter().find(|n| n.id == node.id) {
                    if inode.disabled {
                        debug!("Skipping disabled node: {}", node.name);
                        layer_inputs.push((node_index, None));
                        continue;
                    }
                }

                let input_data = self
                    .gather_input_data(&parsed, node_index, &data_cache)
                    .await?;

                let has_incoming_edges = parsed
                    .graph
                    .edges_directed(node_index, petgraph::Direction::Incoming)
                    .next()
                    .is_some();

                if has_incoming_edges {
                    let total_items: usize = input_data.0.values().map(|v| v.len()).sum();
                    if total_items == 0 {
                        debug!(
                            "Skipping node '{}' because all upstream branches yielded 0 items",
                            node.name
                        );
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
                    }
                    Some((input_data, Some(early_result))) => {
                        // Dead branch, return immediately
                        let fut = async move { Ok((node_index, input_data, early_result)) };
                        futures.push(fut.boxed());
                    }
                    Some((input_data, None)) => {
                        let input_items: usize =
                            input_data.0.values().map(|items| items.len()).sum();
                        let context_ref = &context;
                        let parsed_ref = &parsed;
                        let node = &parsed.graph[node_index];
                        let workflow_cache_clone = Arc::clone(&workflow_cache);

                        self.emit_execution_event(
                            &context,
                            &mut event_sequence,
                            ExecutionEventType::NodeStarted,
                            ExecutionStatus::Running,
                            Some(node),
                            format!("Node '{}' started", node.name),
                            json!({
                                "inputItems": input_items,
                                "layer": node_layers.get(&node_index).copied().unwrap_or_default(),
                            }),
                        )
                        .await;

                        // Execute the node
                        let fut = async move {
                            match self
                                .run_node(
                                    context_ref,
                                    node,
                                    input_data.clone(),
                                    parsed_ref,
                                    workflow_cache_clone,
                                )
                                .await
                            {
                                Ok(res) => Ok((node_index, input_data, res)),
                                Err(e) => Err((node_index, input_data, e)),
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
                let (node_index, input_data, result) = match execute_result {
                    Ok((n_idx, in_data, res)) => (n_idx, in_data, res),
                    Err((
                        n_idx,
                        in_data,
                        BarqError::SuspendExecution {
                            node_name: _,
                            wait_config,
                        },
                    )) => {
                        let node = &parsed.graph[n_idx];
                        info!("Execution suspended at node '{}'", node.name);

                        let config: crate::checkpoint::WaitConfig = serde_json::from_value(
                            wait_config,
                        )
                        .unwrap_or(crate::checkpoint::WaitConfig {
                            wait_type: crate::checkpoint::WaitType::Time,
                            duration_ms: None,
                            webhook_path: None,
                            external_id: None,
                        });

                        let mut manager = crate::checkpoint::CheckpointManager::with_filesystem(
                            std::env::temp_dir().join("barqflow_checkpoints"),
                        );

                        use crate::checkpoint::ExecutionCheckpointBuilder;
                        let checkpoint = ExecutionCheckpointBuilder::new()
                            .with_run_id(context.run_id)
                            .with_workflow_id(context.workflow.id.0.to_string())
                            .with_node_index(n_idx.index())
                            .with_node_data(
                                serde_json::to_value(&in_data).unwrap_or(serde_json::Value::Null),
                            )
                            .with_wait_config(config.clone())
                            .build();

                        if let Ok(cp) = checkpoint {
                            let _ = manager.save_checkpoint(cp).await;
                        }

                        return Err(BarqError::SuspendExecution {
                            node_name: node.name.clone(),
                            wait_config: serde_json::to_value(config)
                                .unwrap_or(serde_json::Value::Null),
                        });
                    }
                    Err((
                        n_idx,
                        in_data,
                        BarqError::ExecuteSubWorkflow {
                            workflow_id,
                            input_data,
                        },
                    )) => {
                        let (resolved_index, result) = self
                            .handle_subworkflow_execution(
                                &context,
                                &parsed,
                                n_idx,
                                &in_data,
                                workflow_id,
                                input_data,
                            )
                            .await?;
                        (resolved_index, in_data, result)
                    }
                    Err((_n_idx, _in_data, e)) => {
                        if let Some(error_workflow_id) =
                            context.workflow.settings.error_workflow.clone()
                        {
                            return Err(BarqError::TriggerErrorWorkflow {
                                error_workflow_id,
                                original_error: e.to_string(),
                            });
                        }
                        return Err(e);
                    }
                };

                let node = &parsed.graph[node_index];
                let input_items: usize = input_data.0.values().map(|items| items.len()).sum();
                let output_items: usize = result.outputs.iter().map(|branch| branch.len()).sum();
                let skipped = input_items == 0
                    && output_items == 0
                    && result.success
                    && result.error.is_none();

                self.emit_execution_event(
                    &context,
                    &mut event_sequence,
                    ExecutionEventType::NodeFinished,
                    ExecutionStatus::Running,
                    Some(node),
                    if skipped {
                        format!(
                            "Node '{}' skipped because upstream branches yielded no items",
                            node.name
                        )
                    } else if result.success {
                        format!("Node '{}' completed", node.name)
                    } else {
                        format!("Node '{}' failed", node.name)
                    },
                    json!({
                        "inputItems": input_items,
                        "outputItems": output_items,
                        "success": result.success,
                        "error": result.error,
                        "skipped": skipped,
                    }),
                )
                .await;

                if let Some(first_output) = result.outputs.first() {
                    let json_items: Vec<serde_json::Value> = first_output
                        .iter()
                        .map(|item| serde_json::Value::Object(item.json.0.clone()))
                        .collect();
                    workflow_cache
                        .write()
                        .await
                        .insert(node.name.clone(), json_items);
                }

                data_cache.insert(node.id.clone(), result.clone());
                results.insert(node.name.clone(), result);

                if let Some((target, _)) = &scoped_execution {
                    if node.id.to_string() == *target || node.name == *target {
                        stop_reached = true;
                    }
                }
            }

            if stop_reached {
                info!("Reached target node; stopping partial execution");
                break;
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
        let mut event_sequence = context.event_sequence_start;

        if Self::is_cancelled(&context) {
            return Err(Self::cancellation_error(&context));
        }

        let flow_workflow = self.convert_workflow(&context.workflow);
        let parsed = WorkflowToGraphParser::parse(&flow_workflow).map_err(|e| {
            BarqError::WorkflowConfigurationError {
                message: e.to_string(),
            }
        })?;

        let execution_order = GraphTraversal::topological_sort(&parsed.graph).map_err(|e| {
            BarqError::WorkflowConfigurationError {
                message: e.to_string(),
            }
        })?;

        // Group nodes into parallel executable layers for resuming as well
        let mut node_layers: HashMap<NodeIndex, usize> = HashMap::new();
        let mut max_layer = 0;

        for &node_index in &execution_order {
            let parents = GraphTraversal::get_parents(&parsed.graph, node_index);
            let layer = parents
                .iter()
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
                            workflow_cache
                                .write()
                                .await
                                .insert(res.node_name.clone(), json_items);
                        }

                        data_cache.insert(node.id.clone(), res.clone());
                        results.insert(node.name.clone(), res);
                    }
                    continue;
                }

                // Normal execution loop from here on
                let input_data = self
                    .gather_input_data(&parsed, node_index, &data_cache)
                    .await?;

                let has_incoming_edges = parsed
                    .graph
                    .edges_directed(node_index, petgraph::Direction::Incoming)
                    .next()
                    .is_some();

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
                    None => {}
                    Some((input_data, Some(early_result))) => {
                        let fut = async move { Ok((node_index, input_data, early_result)) };
                        futures.push(fut.boxed());
                    }
                    Some((input_data, None)) => {
                        let input_items: usize =
                            input_data.0.values().map(|items| items.len()).sum();
                        let context_ref = &context;
                        let parsed_ref = &parsed;
                        let node = &parsed.graph[node_index];
                        let workflow_cache_clone = Arc::clone(&workflow_cache);

                        self.emit_execution_event(
                            &context,
                            &mut event_sequence,
                            ExecutionEventType::NodeStarted,
                            ExecutionStatus::Running,
                            Some(node),
                            format!("Node '{}' started", node.name),
                            json!({
                                "inputItems": input_items,
                                "layer": node_layers.get(&node_index).copied().unwrap_or_default(),
                            }),
                        )
                        .await;

                        let fut = async move {
                            match self
                                .run_node(
                                    context_ref,
                                    node,
                                    input_data.clone(),
                                    parsed_ref,
                                    workflow_cache_clone,
                                )
                                .await
                            {
                                Ok(res) => Ok((node_index, input_data, res)),
                                Err(e) => Err((node_index, input_data, e)),
                            }
                        };
                        futures.push(fut.boxed());
                    }
                }
            }

            let layer_results = futures::future::join_all(futures).await;

            for execute_result in layer_results {
                let (node_index, input_data, result) = match execute_result {
                    Ok((n_idx, in_data, res)) => (n_idx, in_data, res),
                    Err((
                        n_idx,
                        in_data,
                        BarqError::SuspendExecution {
                            node_name: _,
                            wait_config,
                        },
                    )) => {
                        let node = &parsed.graph[n_idx];
                        let config: crate::checkpoint::WaitConfig = serde_json::from_value(
                            wait_config,
                        )
                        .unwrap_or(crate::checkpoint::WaitConfig {
                            wait_type: crate::checkpoint::WaitType::Time,
                            duration_ms: None,
                            webhook_path: None,
                            external_id: None,
                        });
                        let mut manager = crate::checkpoint::CheckpointManager::with_filesystem(
                            std::env::temp_dir().join("barqflow_checkpoints"),
                        );
                        use crate::checkpoint::ExecutionCheckpointBuilder;
                        let checkpoint = ExecutionCheckpointBuilder::new()
                            .with_run_id(context.run_id)
                            .with_workflow_id(context.workflow.id.0.to_string())
                            .with_node_index(n_idx.index())
                            .with_node_data(
                                serde_json::to_value(&in_data).unwrap_or(serde_json::Value::Null),
                            )
                            .with_wait_config(config.clone())
                            .build();
                        if let Ok(cp) = checkpoint {
                            let _ = manager.save_checkpoint(cp).await;
                        }
                        return Err(BarqError::SuspendExecution {
                            node_name: node.name.clone(),
                            wait_config: serde_json::to_value(config)
                                .unwrap_or(serde_json::Value::Null),
                        });
                    }
                    Err((
                        n_idx,
                        in_data,
                        BarqError::ExecuteSubWorkflow {
                            workflow_id,
                            input_data,
                        },
                    )) => {
                        let (resolved_index, result) = self
                            .handle_subworkflow_execution(
                                &context,
                                &parsed,
                                n_idx,
                                &in_data,
                                workflow_id,
                                input_data,
                            )
                            .await?;
                        (resolved_index, in_data, result)
                    }
                    Err((_n_idx, _in_data, e)) => {
                        if let Some(error_workflow_id) =
                            context.workflow.settings.error_workflow.clone()
                        {
                            error!(
                                "Workflow failed, should trigger error workflow: {}",
                                error_workflow_id
                            );
                            return Err(BarqError::TriggerErrorWorkflow {
                                error_workflow_id,
                                original_error: e.to_string(),
                            });
                        }
                        return Err(e);
                    }
                };

                let node = &parsed.graph[node_index];
                let input_items: usize = input_data.0.values().map(|items| items.len()).sum();
                let output_items: usize = result.outputs.iter().map(|branch| branch.len()).sum();
                let skipped = input_items == 0
                    && output_items == 0
                    && result.success
                    && result.error.is_none();

                self.emit_execution_event(
                    &context,
                    &mut event_sequence,
                    ExecutionEventType::NodeFinished,
                    ExecutionStatus::Running,
                    Some(node),
                    if skipped {
                        format!(
                            "Node '{}' skipped because upstream branches yielded no items",
                            node.name
                        )
                    } else if result.success {
                        format!("Node '{}' completed", node.name)
                    } else {
                        format!("Node '{}' failed", node.name)
                    },
                    json!({
                        "inputItems": input_items,
                        "outputItems": output_items,
                        "success": result.success,
                        "error": result.error,
                        "skipped": skipped,
                    }),
                )
                .await;

                if let Some(first_output) = result.outputs.first() {
                    let json_items: Vec<serde_json::Value> = first_output
                        .iter()
                        .map(|item| serde_json::Value::Object(item.json.0.clone()))
                        .collect();
                    workflow_cache
                        .write()
                        .await
                        .insert(node.name.clone(), json_items);
                }

                data_cache.insert(node.id.clone(), result.clone());
                results.insert(node.name.clone(), result);
            }
        }

        info!("Resumed workflow execution completed");
        Ok(results)
    }

    fn fallback_subworkflow_input(input: &ITaskDataConnections) -> Vec<INodeExecutionData> {
        if let Some(items) = input.0.get(&0) {
            return items.clone();
        }

        let mut merged = Vec::new();
        for items in input.0.values() {
            merged.extend(items.clone());
        }
        merged
    }

    fn materialize_subworkflow_input(
        &self,
        node_name: &str,
        raw_input_data: serde_json::Value,
        fallback_input: &ITaskDataConnections,
    ) -> Result<Vec<INodeExecutionData>, BarqError> {
        if raw_input_data.is_null() {
            return Ok(Self::fallback_subworkflow_input(fallback_input));
        }

        if let Ok(items) = serde_json::from_value::<Vec<INodeExecutionData>>(raw_input_data.clone())
        {
            return Ok(items);
        }

        if let Ok(streams) = serde_json::from_value::<Vec<Vec<INodeExecutionData>>>(raw_input_data)
        {
            let merged = streams.into_iter().flatten().collect::<Vec<_>>();
            return Ok(merged);
        }

        Err(BarqError::NodeOperationError {
            node_name: node_name.to_string(),
            message: "Invalid subworkflow inputData payload. Expected array of execution items."
                .to_string(),
        })
    }

    async fn handle_subworkflow_execution(
        &self,
        context: &WorkflowRunContext,
        parsed: &ParsedGraph,
        node_index: NodeIndex,
        node_input: &ITaskDataConnections,
        workflow_id: String,
        raw_input_data: serde_json::Value,
    ) -> Result<(NodeIndex, NodeExecutionResult), BarqError> {
        let node = &parsed.graph[node_index];

        let child_workflow_id = uuid::Uuid::parse_str(workflow_id.as_str()).map_err(|_| {
            BarqError::NodeOperationError {
                node_name: node.name.clone(),
                message: format!(
                    "Invalid workflowId '{}'. Expected UUID from existing workflow.",
                    workflow_id
                ),
            }
        })?;

        let child_input =
            self.materialize_subworkflow_input(node.name.as_str(), raw_input_data, node_input)?;
        let child_outputs = self
            .execute_subworkflow(context, child_workflow_id, child_input)
            .await?;

        Ok((
            node_index,
            NodeExecutionResult {
                node_name: node.name.clone(),
                outputs: child_outputs,
                success: true,
                error: None,
            },
        ))
    }

    pub async fn execute_subworkflow(
        &self,
        parent_ctx: &WorkflowRunContext,
        child_workflow_id: uuid::Uuid,
        input: Vec<INodeExecutionData>,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let Some(executor) = self.subworkflow_executor.as_ref() else {
            return Err(BarqError::NodeOperationError {
                node_name: "executeWorkflow".to_string(),
                message:
                    "Subworkflow executor is not configured in runtime; cannot execute child workflow."
                        .to_string(),
            });
        };

        let parent = crate::subworkflow::SubWorkflowParentContext {
            run_id: parent_ctx.run_id,
            execution_id: parent_ctx.execution_id,
            parent_execution_id: parent_ctx.parent_execution_id,
            manual: parent_ctx.manual,
        };

        match executor
            .execute_subworkflow(parent, child_workflow_id, input)
            .await
        {
            Ok(result) => Ok(result.outputs),
            Err(err @ BarqError::SubworkflowError { .. }) => Err(err),
            Err(other) => Err(BarqError::SubworkflowError {
                child_execution_id: "unknown".to_string(),
                message: other.to_string(),
            }),
        }
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

        let node_continue_on_fail = inode
            .parameters
            .0
            .get("continueOnFail")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let max_retries = inode
            .parameters
            .0
            .get("retryOnFail")
            .and_then(|v| v.as_bool())
            .and_then(|b| {
                if b {
                    inode
                        .parameters
                        .0
                        .get("maxTries")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u32)
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let retry_interval_ms = inode
            .parameters
            .0
            .get("waitBetweenTries")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let error_handler = crate::error_handler::ErrorHandler::new(
            self.config.continue_on_fail || node_continue_on_fail,
        )
        .with_retries(max_retries, retry_interval_ms);

        let execute_result = error_handler
            .execute_isolated_async(&node.name, || node_info.node_impl.execute(&exec_context))
            .await;

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
    use uuid::Uuid;

    use super::*;
    use barqflow_core::properties::INodeProperties;
    use barqflow_core::schema::{INode, INodeParameters, IWorkflowSettings};
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

    struct MockSubworkflowCallNode;

    #[async_trait]
    impl barqflow_core::traits::INodeType for MockSubworkflowCallNode {
        fn get_description(&self) -> IDataObject {
            IDataObject::from(json!({
                "name": "mockSubworkflowCall",
                "displayName": "Mock Subworkflow Call",
                "description": "Triggers sub-workflow execution from tests"
            }))
        }

        async fn execute(
            &self,
            context: &dyn IExecuteFunctions,
        ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
            let workflow_id = context
                .get_node_parameter("workflowId", None)
                .await
                .map(|v| v.as_str().unwrap_or("").to_string())
                .unwrap_or_default();
            let input_data =
                serde_json::to_value(context.get_input_data(0).await.unwrap_or_default())
                    .unwrap_or(serde_json::Value::Null);

            Err(BarqError::ExecuteSubWorkflow {
                workflow_id,
                input_data,
            })
        }
    }

    struct MockSubworkflowExecutor {
        expected_workflow_id: Uuid,
    }

    #[async_trait]
    impl crate::subworkflow::SubWorkflowExecutor for MockSubworkflowExecutor {
        async fn execute_subworkflow(
            &self,
            _parent: crate::subworkflow::SubWorkflowParentContext,
            child_workflow_id: Uuid,
            input: Vec<INodeExecutionData>,
        ) -> Result<crate::subworkflow::SubWorkflowExecutionResult, BarqError> {
            assert_eq!(child_workflow_id, self.expected_workflow_id);
            assert_eq!(input.len(), 1);

            Ok(crate::subworkflow::SubWorkflowExecutionResult {
                child_execution_id: "child-exec-test".to_string(),
                outputs: vec![vec![INodeExecutionData::new(IDataObject::from(json!({
                    "childResult": true,
                })))]],
            })
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

        let subworkflow_node = barqflow_registry::registry::NodeInfo {
            name: "mockSubworkflowCall".to_string(),
            display_name: "Mock Subworkflow Call".to_string(),
            version: 1.0,
            description: "A mock node that requests sub-workflow execution".to_string(),
            properties: INodeProperties {
                display_name: Some("Mock Subworkflow".to_string()),
                properties: vec![],
                required_values: None,
            },
            is_trigger: false,
            max_inputs: 1,
            node_impl: Arc::new(MockSubworkflowCallNode),
        };

        registry.register_node(subworkflow_node).unwrap();

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

    fn create_targeted_scope_workflow() -> CoreWorkflowDef {
        let node_a = INode {
            id: NodeId::new("node_a"),
            name: "NodeA".to_string(),
            r#type: "mockPassThrough".to_string(),
            type_version: 1.0,
            position: [0.0, 0.0],
            parameters: INodeParameters::default(),
            credentials: vec![],
            disabled: false,
        };
        let node_b = INode {
            id: NodeId::new("node_b"),
            name: "NodeB".to_string(),
            r#type: "mockPassThrough".to_string(),
            type_version: 1.0,
            position: [100.0, 0.0],
            parameters: INodeParameters::default(),
            credentials: vec![],
            disabled: false,
        };
        let node_c = INode {
            id: NodeId::new("node_c"),
            name: "NodeC".to_string(),
            r#type: "mockPassThrough".to_string(),
            type_version: 1.0,
            position: [200.0, 0.0],
            parameters: INodeParameters::default(),
            credentials: vec![],
            disabled: false,
        };

        let mut connections = HashMap::new();
        connections.insert(
            "NodeA".to_string(),
            barqflow_core::schema::INodeConnections(HashMap::from([(
                NodeConnectionType::Main,
                vec![vec![IConnection {
                    node: "NodeB".to_string(),
                    r#type: NodeConnectionType::Main,
                    index: 0,
                }]],
            )])),
        );

        CoreWorkflowDef {
            id: WorkflowId::new(),
            name: "Targeted Scope Workflow".to_string(),
            nodes: vec![node_a, node_b, node_c],
            connections,
            active: true,
            settings: IWorkflowSettings::default(),
        }
    }

    #[tokio::test]
    async fn test_runner_creation() {
        let registry = create_mock_registry();
        let config = ExecutionConfig::default();
        let runner = WorkflowRunner::new(registry, config);

        assert!(!runner.config.continue_on_fail);
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
            parent_execution_id: None,
            cancellation_token: None,
            stop_after_node_id: None,
            event_sequence_start: 0,
        };

        let results = runner.run_workflow(context).await.unwrap();

        assert_eq!(results.len(), 1);
        assert!(results.contains_key("TestNode"));

        let result = &results["TestNode"];
        assert!(result.success);
        assert_eq!(result.outputs.len(), 1);
    }

    #[tokio::test]
    async fn test_run_workflow_can_stop_at_target_node() {
        let registry = create_mock_registry();
        let runner = WorkflowRunner::new(registry, ExecutionConfig::default());
        let workflow = create_targeted_scope_workflow();

        let context = WorkflowRunContext {
            run_id: RunId::new(),
            workflow,
            static_data: None,
            manual: true,
            execution_id: None,
            parent_execution_id: None,
            cancellation_token: None,
            stop_after_node_id: Some("node_b".to_string()),
            event_sequence_start: 0,
        };

        let results = runner.run_workflow(context).await.unwrap();
        assert!(results.contains_key("NodeA"));
        assert!(results.contains_key("NodeB"));
        assert!(!results.contains_key("NodeC"));
    }

    #[tokio::test]
    async fn test_run_workflow_stop_at_unknown_node_returns_error() {
        let registry = create_mock_registry();
        let runner = WorkflowRunner::new(registry, ExecutionConfig::default());
        let workflow = create_test_workflow();

        let context = WorkflowRunContext {
            run_id: RunId::new(),
            workflow,
            static_data: None,
            manual: true,
            execution_id: None,
            parent_execution_id: None,
            cancellation_token: None,
            stop_after_node_id: Some("missing-node".to_string()),
            event_sequence_start: 0,
        };

        let err = runner.run_workflow(context).await.unwrap_err();
        match err {
            BarqError::WorkflowConfigurationError { message } => {
                assert!(message.contains("not found"));
            }
            other => panic!("expected WorkflowConfigurationError, got {}", other),
        }
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
            parent_execution_id: None,
            cancellation_token: None,
            stop_after_node_id: None,
            event_sequence_start: 0,
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
            parent_execution_id: None,
            cancellation_token: None,
            stop_after_node_id: None,
            event_sequence_start: 0,
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
            parent_execution_id: None,
            cancellation_token: Some(token),
            stop_after_node_id: None,
            event_sequence_start: 0,
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

    #[tokio::test]
    async fn test_run_workflow_executes_subworkflow_via_runtime_executor() {
        let registry = create_mock_registry();
        let expected_child_id = Uuid::new_v4();

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
        let call = INode {
            id: NodeId::new("call"),
            name: "CallChild".to_string(),
            r#type: "mockSubworkflowCall".to_string(),
            type_version: 1.0,
            position: [120.0, 0.0],
            parameters: INodeParameters(HashMap::from([(
                "workflowId".to_string(),
                json!(expected_child_id.to_string()),
            )])),
            credentials: vec![],
            disabled: false,
        };

        let mut connections = HashMap::new();
        connections.insert(
            "Source".to_string(),
            barqflow_core::schema::INodeConnections(HashMap::from([(
                NodeConnectionType::Main,
                vec![vec![IConnection {
                    node: "CallChild".to_string(),
                    r#type: NodeConnectionType::Main,
                    index: 0,
                }]],
            )])),
        );

        let workflow = CoreWorkflowDef {
            id: WorkflowId::new(),
            name: "Parent".to_string(),
            nodes: vec![source, call],
            connections,
            active: true,
            settings: IWorkflowSettings::default(),
        };

        let runner = WorkflowRunner::new(registry, ExecutionConfig::default())
            .with_subworkflow_executor(Arc::new(MockSubworkflowExecutor {
                expected_workflow_id: expected_child_id,
            }));

        let results = runner
            .run_workflow(WorkflowRunContext {
                run_id: RunId::new(),
                workflow,
                static_data: None,
                manual: true,
                execution_id: None,
                parent_execution_id: None,
                cancellation_token: None,
                stop_after_node_id: None,
                event_sequence_start: 0,
            })
            .await
            .unwrap();

        let child_result = results.get("CallChild").expect("call child result");
        let output_item = child_result
            .outputs
            .first()
            .and_then(|stream| stream.first())
            .expect("child output item");
        assert_eq!(
            output_item
                .json
                .0
                .get("childResult")
                .and_then(|value| value.as_bool()),
            Some(true)
        );
    }

    #[tokio::test]
    async fn test_run_workflow_subworkflow_without_executor_returns_error() {
        let registry = create_mock_registry();
        let expected_child_id = Uuid::new_v4();

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
        let call = INode {
            id: NodeId::new("call"),
            name: "CallChild".to_string(),
            r#type: "mockSubworkflowCall".to_string(),
            type_version: 1.0,
            position: [120.0, 0.0],
            parameters: INodeParameters(HashMap::from([(
                "workflowId".to_string(),
                json!(expected_child_id.to_string()),
            )])),
            credentials: vec![],
            disabled: false,
        };

        let mut connections = HashMap::new();
        connections.insert(
            "Source".to_string(),
            barqflow_core::schema::INodeConnections(HashMap::from([(
                NodeConnectionType::Main,
                vec![vec![IConnection {
                    node: "CallChild".to_string(),
                    r#type: NodeConnectionType::Main,
                    index: 0,
                }]],
            )])),
        );

        let workflow = CoreWorkflowDef {
            id: WorkflowId::new(),
            name: "Parent".to_string(),
            nodes: vec![source, call],
            connections,
            active: true,
            settings: IWorkflowSettings::default(),
        };

        let runner = WorkflowRunner::new(registry, ExecutionConfig::default());
        let err = runner
            .run_workflow(WorkflowRunContext {
                run_id: RunId::new(),
                workflow,
                static_data: None,
                manual: true,
                execution_id: None,
                parent_execution_id: None,
                cancellation_token: None,
                stop_after_node_id: None,
                event_sequence_start: 0,
            })
            .await
            .unwrap_err();

        match err {
            BarqError::NodeOperationError { message, .. } => {
                assert!(message.contains("Subworkflow executor is not configured"));
            }
            other => panic!("expected NodeOperationError, got {}", other),
        }
    }

    #[tokio::test]
    async fn test_run_workflow_respects_max_execution_time() {
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::Arc as StdArc;

        struct SlowNode {
            started: StdArc<AtomicBool>,
        }

        #[async_trait::async_trait]
        impl barqflow_core::traits::INodeType for SlowNode {
            fn get_description(&self) -> IDataObject {
                IDataObject::from(serde_json::json!({"name": "slowNode"}))
            }
            async fn execute(
                &self,
                _ctx: &dyn barqflow_core::traits::IExecuteFunctions,
            ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
                self.started.store(true, Ordering::SeqCst);
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                Ok(vec![])
            }
        }

        let started = StdArc::new(AtomicBool::new(false));
        let registry = Arc::new(NodeRegistry::new());
        registry
            .register_node(barqflow_registry::registry::NodeInfo {
                name: "slowNode".to_string(),
                display_name: "Slow Node".to_string(),
                version: 1.0,
                description: "sleeps forever".to_string(),
                properties: barqflow_core::properties::INodeProperties {
                    display_name: None,
                    properties: vec![],
                    required_values: None,
                },
                is_trigger: false,
                max_inputs: 1,
                node_impl: Arc::new(SlowNode {
                    started: StdArc::clone(&started),
                }),
            })
            .unwrap();

        let config = ExecutionConfig {
            max_execution_time_ms: Some(50), // 50 ms limit
            ..Default::default()
        };
        let runner = WorkflowRunner::new(registry, config);

        let workflow = {
            use barqflow_core::schema::{INode, INodeParameters, IWorkflowSettings};
            use barqflow_core::types::{NodeId, WorkflowId};
            CoreWorkflowDef {
                id: WorkflowId::new(),
                name: "Timeout Test".to_string(),
                nodes: vec![INode {
                    id: NodeId::new("slow"),
                    name: "Slow".to_string(),
                    r#type: "slowNode".to_string(),
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
        };

        let err = runner
            .run_workflow(WorkflowRunContext {
                run_id: RunId::new(),
                workflow,
                static_data: None,
                manual: true,
                execution_id: None,
                parent_execution_id: None,
                cancellation_token: None,
                stop_after_node_id: None,
                event_sequence_start: 0,
            })
            .await
            .unwrap_err();

        match err {
            BarqError::WorkflowConfigurationError { message } => {
                assert!(message.contains("exceeded maximum allowed time"));
            }
            other => panic!(
                "expected WorkflowConfigurationError (timeout), got: {}",
                other
            ),
        }
    }
}
