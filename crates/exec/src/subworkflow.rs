//! Sub-Workflow Invocation (ExecuteWorkflow)
//!
//! Implements ExecuteWorkflow node functionality for calling other workflows
//! from within a workflow execution context.

use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::types::{GenericValue, IDataObject, RunId, WorkflowId};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, info, instrument};

/// Configuration for executing a sub-workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteWorkflowConfig {
    /// The workflow ID or name to execute
    pub workflow_id: String,
    /// Whether to wait for the sub-workflow to complete
    pub wait_for_completion: bool,
    /// Input data to pass to the sub-workflow
    pub input_data: Option<serde_json::Value>,
    /// Options for passing data back to parent workflow
    pub pass_data_to_parent: PassDataOption,
}

/// How to pass data from sub-workflow back to parent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PassDataOption {
    /// Pass all output data
    All,
    /// Pass only specified fields
    Selective { fields: Vec<String> },
    /// Pass nothing
    None,
}

/// Result of executing a sub-workflow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubWorkflowResult {
    /// The run ID of the sub-workflow execution
    pub run_id: RunId,
    /// The workflow ID that was executed
    pub workflow_id: WorkflowId,
    /// Whether the sub-workflow completed (if waited for)
    pub completed: bool,
    /// Output data from the sub-workflow (if available)
    pub output_data: Option<Vec<INodeExecutionData>>,
    /// Error message if the sub-workflow failed
    pub error: Option<String>,
}

/// Sub-workflow executor for managing workflow-to-workflow calls.
pub struct SubWorkflowExecutor {
    /// Workflow registry for resolving workflow IDs
    workflow_registry: HashMap<String, WorkflowId>,
}

impl SubWorkflowExecutor {
    /// Create a new sub-workflow executor.
    pub fn new() -> Self {
        Self {
            workflow_registry: HashMap::new(),
        }
    }

    /// Register a workflow for execution by ID/name.
    pub fn register_workflow(&mut self, name: String, id: WorkflowId) {
        self.workflow_registry.insert(name, id);
    }

    /// Execute a sub-workflow.
    ///
    /// # Arguments
    /// * `config` - The execution configuration
    /// * `parent_run_id` - The parent workflow's run ID
    ///
    /// # Returns
    /// The sub-workflow execution result
    #[instrument(skip(self, config), fields(workflow_id = %config.workflow_id))]
    pub async fn execute_workflow(
        &self,
        config: &ExecuteWorkflowConfig,
        parent_run_id: RunId,
    ) -> Result<SubWorkflowResult, BarqError> {
        info!("Executing sub-workflow: {}", config.workflow_id);

        // Resolve the workflow ID
        let workflow_id = self
            .resolve_workflow_id(&config.workflow_id)
            .ok_or_else(|| BarqError::WorkflowConfigurationError {
                message: format!("Workflow '{}' not found", config.workflow_id),
            })?;

        // In a real implementation, this would:
        // 1. Create a new execution run
        // 2. Pass the input data to the sub-workflow
        // 3. Execute the sub-workflow (either synchronously or queue it)
        // 4. Return results based on wait_for_completion

        let run_id = RunId::new();

        if config.wait_for_completion {
            // Simulate synchronous execution
            debug!("Waiting for sub-workflow completion");
            Ok(SubWorkflowResult {
                run_id,
                workflow_id,
                completed: true,
                output_data: Some(self.create_mock_output(config)),
                error: None,
            })
        } else {
            // Fire-and-forget execution
            debug!("Fire-and-forget sub-workflow execution");
            Ok(SubWorkflowResult {
                run_id,
                workflow_id,
                completed: false,
                output_data: None,
                error: None,
            })
        }
    }

    /// Resolve a workflow name to a WorkflowId.
    fn resolve_workflow_id(&self, name: &str) -> Option<WorkflowId> {
        // Try as registered name
        if let Some(id) = self.workflow_registry.get(name) {
            return Some(*id);
        }

        // Try parsing as UUID
        if let Ok(uuid) = uuid::Uuid::parse_str(name) {
            return Some(WorkflowId(uuid));
        }

        None
    }

    /// Create mock output data for testing.
    fn create_mock_output(&self, config: &ExecuteWorkflowConfig) -> Vec<INodeExecutionData> {
        let data = config
            .input_data
            .clone()
            .unwrap_or_else(|| json!({"result": "mock_sub_workflow_output"}));

        vec![INodeExecutionData::new(IDataObject::from(data))]
    }
}

impl Default for SubWorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Context passed to sub-workflow about parent workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ParentWorkflowContext {
    /// Parent workflow run ID
    pub parent_run_id: RunId,
    /// Parent workflow ID
    pub parent_workflow_id: WorkflowId,
    /// Node that triggered this sub-workflow
    pub triggering_node: String,
    /// Timestamp when sub-workflow was triggered
    pub triggered_at: i64,
}

/// Builder for creating ExecuteWorkflowConfig.
pub struct ExecuteWorkflowConfigBuilder {
    workflow_id: Option<String>,
    wait_for_completion: Option<bool>,
    input_data: Option<serde_json::Value>,
    pass_data_to_parent: Option<PassDataOption>,
}

impl Default for ExecuteWorkflowConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecuteWorkflowConfigBuilder {
    pub fn new() -> Self {
        Self {
            workflow_id: None,
            wait_for_completion: Some(true),
            input_data: None,
            pass_data_to_parent: Some(PassDataOption::All),
        }
    }

    pub fn with_workflow_id(mut self, id: String) -> Self {
        self.workflow_id = Some(id);
        self
    }

    pub fn with_wait_for_completion(mut self, wait: bool) -> Self {
        self.wait_for_completion = Some(wait);
        self
    }

    pub fn with_input_data(mut self, data: serde_json::Value) -> Self {
        self.input_data = Some(data);
        self
    }

    pub fn with_pass_data_option(mut self, option: PassDataOption) -> Self {
        self.pass_data_to_parent = Some(option);
        self
    }

    pub fn build(self) -> Result<ExecuteWorkflowConfig, String> {
        Ok(ExecuteWorkflowConfig {
            workflow_id: self.workflow_id.ok_or("workflow_id is required")?,
            wait_for_completion: self.wait_for_completion.unwrap_or(true),
            input_data: self.input_data,
            pass_data_to_parent: self.pass_data_to_parent.unwrap_or(PassDataOption::All),
        })
    }
}

/// Transform output data from sub-workflow based on PassDataOption.
pub fn transform_output_data(
    data: Vec<INodeExecutionData>,
    option: &PassDataOption,
) -> Vec<INodeExecutionData> {
    match option {
        PassDataOption::All => data,
        PassDataOption::Selective { fields } => {
            data.into_iter()
                .map(|mut item| {
                    if let Some(obj) = item.json.0.as_object_mut() {
                        let keys_to_keep: std::collections::HashSet<String> =
                            fields.iter().cloned().collect();
                        obj.retain(|k, _| keys_to_keep.contains(k));
                    }
                    item
                })
                .collect()
        }
        PassDataOption::None => vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_executor_creation() {
        let executor = SubWorkflowExecutor::new();
        assert!(executor.workflow_registry.is_empty());
    }

    #[test]
    fn test_workflow_registration() {
        let mut executor = SubWorkflowExecutor::new();
        let id = WorkflowId::new();

        executor.register_workflow("test-workflow".to_string(), id);
        assert!(executor.workflow_registry.contains_key("test-workflow"));
    }

    #[tokio::test]
    async fn test_execute_workflow_wait_completion() {
        let mut executor = SubWorkflowExecutor::new();
        let id = WorkflowId::new();
        executor.register_workflow("sub-flow".to_string(), id);

        let config = ExecuteWorkflowConfigBuilder::new()
            .with_workflow_id("sub-flow".to_string())
            .with_wait_for_completion(true)
            .build()
            .unwrap();

        let result = executor
            .execute_workflow(&config, RunId::new())
            .await
            .unwrap();

        assert!(result.completed);
        assert!(result.output_data.is_some());
    }

    #[tokio::test]
    async fn test_execute_workflow_no_wait() {
        let mut executor = SubWorkflowExecutor::new();
        let id = WorkflowId::new();
        executor.register_workflow("async-flow".to_string(), id);

        let config = ExecuteWorkflowConfigBuilder::new()
            .with_workflow_id("async-flow".to_string())
            .with_wait_for_completion(false)
            .build()
            .unwrap();

        let result = executor
            .execute_workflow(&config, RunId::new())
            .await
            .unwrap();

        assert!(!result.completed);
        assert!(result.output_data.is_none());
    }

    #[tokio::test]
    async fn test_execute_workflow_not_found() {
        let executor = SubWorkflowExecutor::new();

        let config = ExecuteWorkflowConfigBuilder::new()
            .with_workflow_id("non-existent".to_string())
            .build()
            .unwrap();

        let result = executor.execute_workflow(&config, RunId::new()).await;

        assert!(result.is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = ExecuteWorkflowConfigBuilder::new()
            .with_workflow_id("test".to_string())
            .with_wait_for_completion(false)
            .with_input_data(json!({"key": "value"}))
            .with_pass_data_option(PassDataOption::None)
            .build()
            .unwrap();

        assert_eq!(config.workflow_id, "test");
        assert!(!config.wait_for_completion);
        assert!(config.input_data.is_some());
        assert!(matches!(config.pass_data_to_parent, PassDataOption::None));
    }

    #[test]
    fn test_config_builder_missing_required() {
        let result = ExecuteWorkflowConfigBuilder::new().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_output_all() {
        let data = vec![INodeExecutionData::new(IDataObject::from(
            json!({"field1": "value1", "field2": "value2"}),
        ))];

        let result = transform_output_data(data, &PassDataOption::All);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].json.0["field1"], "value1");
        assert_eq!(result[0].json.0["field2"], "value2");
    }

    #[test]
    fn test_transform_output_selective() {
        let data = vec![INodeExecutionData::new(IDataObject::from(
            json!({"field1": "value1", "field2": "value2", "field3": "value3"}),
        ))];

        let result = transform_output_data(
            data,
            &PassDataOption::Selective {
                fields: vec!["field1".to_string(), "field3".to_string()],
            },
        );

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].json.0["field1"], "value1");
        assert!(result[0].json.0.get("field2").is_none());
        assert_eq!(result[0].json.0["field3"], "value3");
    }

    #[test]
    fn test_transform_output_none() {
        let data = vec![INodeExecutionData::new(IDataObject::from(
            json!({"field1": "value1"}),
        ))];

        let result = transform_output_data(data, &PassDataOption::None);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_resolve_workflow_id_by_name() {
        let mut executor = SubWorkflowExecutor::new();
        let id = WorkflowId::new();
        executor.register_workflow("my-workflow".to_string(), id);

        let resolved = executor.resolve_workflow_id("my-workflow");
        assert_eq!(resolved, Some(id));
    }

    #[test]
    fn test_resolve_workflow_id_by_uuid() {
        let executor = SubWorkflowExecutor::new();
        let id = WorkflowId::new();

        let resolved = executor.resolve_workflow_id(&id.0.to_string());
        assert_eq!(resolved, Some(id));
    }

    #[test]
    fn test_resolve_workflow_id_not_found() {
        let executor = SubWorkflowExecutor::new();

        let resolved = executor.resolve_workflow_id("non-existent");
        assert!(resolved.is_none());
    }

    #[test]
    fn test_sub_workflow_result_serialization() {
        let result = SubWorkflowResult {
            run_id: RunId::new(),
            workflow_id: WorkflowId::new(),
            completed: true,
            output_data: Some(vec![INodeExecutionData::new(IDataObject::new())]),
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("\"completed\":true"));
    }

    #[test]
    fn test_pass_data_option_serialization() {
        let option = PassDataOption::Selective {
            fields: vec!["field1".to_string(), "field2".to_string()],
        };

        let json = serde_json::to_string(&option).unwrap();
        assert!(json.contains("selective"));
        assert!(json.contains("field1"));
    }
}
