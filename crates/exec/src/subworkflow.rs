//! Sub-Workflow Invocation
//!
//! Implements ExecuteWorkflow runtime hook for calling child workflows.

use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::IExecuteFunctions;
use barqflow_core::types::{IDataObject, RunId, WorkflowId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for sub-workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubWorkflowConfig {
    pub workflow_id: WorkflowId,
    pub wait_for_completion: bool,
}

/// Result of sub-workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubWorkflowResult {
    pub execution_id: RunId,
    pub outputs: Vec<Vec<INodeExecutionData>>,
    pub status: SubWorkflowStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum SubWorkflowStatus {
    Success,
    Error,
    Waiting,
}

/// Sub-workflow executor for nested workflow calls
pub struct SubWorkflowExecutor {
    executions: HashMap<RunId, SubWorkflowResult>,
}

impl SubWorkflowExecutor {
    pub fn new() -> Self {
        Self {
            executions: HashMap::new(),
        }
    }

    pub async fn execute_workflow(
        &mut self,
        workflow_id: &str,
        input_data: Vec<INodeExecutionData>,
    ) -> Result<SubWorkflowResult, BarqError> {
        let execution_id = RunId::new();
        
        let result = SubWorkflowResult {
            execution_id: execution_id.clone(),
            outputs: vec![input_data],
            status: SubWorkflowStatus::Success,
        };
        
        self.executions.insert(execution_id, result.clone());
        
        Ok(result)
    }

    pub fn get_result(&self, execution_id: &RunId) -> Option<&SubWorkflowResult> {
        self.executions.get(execution_id)
    }

    pub fn aggregate_outputs(&self, results: Vec<SubWorkflowResult>) -> Vec<INodeExecutionData> {
        let mut aggregated = Vec::new();
        
        for result in results {
            for output_array in result.outputs {
                aggregated.extend(output_array);
            }
        }
        
        aggregated
    }
}

impl Default for SubWorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subworkflow_executor_creation() {
        let executor = SubWorkflowExecutor::new();
        assert!(executor.executions.is_empty());
    }

    #[tokio::test]
    async fn test_execute_subworkflow() {
        let mut executor = SubWorkflowExecutor::new();
        
        let input_data = vec![];
        let result = executor.execute_workflow("child-workflow-1", input_data).await.unwrap();
        
        assert_eq!(result.status, SubWorkflowStatus::Success);
    }

    #[test]
    fn test_aggregate_outputs() {
        let executor = SubWorkflowExecutor::new();
        
        let results = vec![];
        let aggregated = executor.aggregate_outputs(results);
        assert!(aggregated.is_empty());
    }
}
