//! Sub-workflow execution contracts.
//!
//! This module defines the pluggable runtime interface used by the execution
//! engine to execute child workflows for the Execute Workflow node.

use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::types::RunId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Lightweight metadata about the parent run passed to sub-workflow executors.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubWorkflowParentContext {
    pub run_id: RunId,
    pub execution_id: Option<Uuid>,
    pub parent_execution_id: Option<Uuid>,
    pub manual: bool,
}

/// Child execution outcome consumed by the workflow runner.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubWorkflowExecutionResult {
    pub child_execution_id: String,
    pub outputs: Vec<Vec<INodeExecutionData>>,
}

/// Runtime abstraction for executing child workflows.
#[async_trait]
pub trait SubWorkflowExecutor: Send + Sync {
    async fn execute_subworkflow(
        &self,
        parent: SubWorkflowParentContext,
        child_workflow_id: Uuid,
        input: Vec<INodeExecutionData>,
    ) -> Result<SubWorkflowExecutionResult, BarqError>;
}
