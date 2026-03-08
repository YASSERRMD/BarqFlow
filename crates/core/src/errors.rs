use thiserror::Error;

/// The root error type for the BarqFlow engine.
#[derive(Debug, Error)]
pub enum BarqError {
    /// Error thrown when a workflow cannot be activated (e.g., due to invalid cron expression or webhook conflict).
    #[error("Failed to activate workflow '{workflow_id}': {reason}")]
    WorkflowActivationError { workflow_id: String, reason: String },

    /// Error thrown when an execution is manually stopped or cancelled.
    #[error("Execution {execution_id} was cancelled")]
    ExecutionCancelledError { execution_id: String },

    /// Error thrown when a Node's specific API logic fails (equivalent to NodeApiError in n8n).
    #[error("API Execution Error in node '{node_name}': {message}")]
    NodeApiError { node_name: String, message: String },

    /// Error thrown when a generic operational failure occurs during node execution (e.g. unable to read filesystem).
    #[error("Operation Error in node '{node_name}': {message}")]
    NodeOperationError { node_name: String, message: String },

    /// Error thrown when the Rhai expression engine fails to parse or evaluate a `{{ $json... }}` expression.
    #[error("Expression Evaluation Error in node '{node_name}': {message}")]
    ExpressionError { node_name: String, message: String },

    /// Error thrown when something is structurally wrong with the workflow configuration itself (e.g. cycle detected).
    #[error("Workflow Configuration Error: {message}")]
    WorkflowConfigurationError { message: String },

    #[error("Internal System Error: {0}")]
    InternalError(String),

    /// Special control-flow variant to indicate the execution should be suspended and checkpointed.
    #[error("Execution suspended for Wait node")]
    SuspendExecution {
        node_name: String,
        wait_config: serde_json::Value,
    },

    /// Special control-flow variant to indicate the Engine should run an error workflow because this workflow failed.
    #[error("Execution failed and triggered error workflow '{error_workflow_id}': {original_error}")]
    TriggerErrorWorkflow {
        error_workflow_id: String,
        original_error: String,
    },

    /// Special control-flow variant to indicate the Engine should pause the current node, execute a sub-workflow, and map the outputs back.
    #[error("Calling sub-workflow '{workflow_id}'")]
    ExecuteSubWorkflow {
        workflow_id: String,
        input_data: serde_json::Value,
    },

    /// Raised when a child workflow execution fails while running from Execute Workflow node.
    #[error("Subworkflow execution failed (child execution '{child_execution_id}'): {message}")]
    SubworkflowError {
        child_execution_id: String,
        message: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = BarqError::ExpressionError {
            node_name: "Set".into(),
            message: "Variable not found".into(),
        };

        assert_eq!(
            err.to_string(),
            "Expression Evaluation Error in node 'Set': Variable not found"
        );
    }
}
