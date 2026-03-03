//! Error Handling and Continue On Fail
//!
//! Implements error trigger functionality and Continue On Fail behavior
//! for workflow execution resilience.

use barqflow_core::errors::BarqError;
use barqflow_core::types::RunId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, warn};

/// Error output configuration for a node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorOutput {
    /// Whether to enable error output capturing
    pub enabled: bool,
    /// Which output index to send errors to (usually output index 1)
    pub error_output_index: usize,
}

/// Continue On Fail configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ContinueOnFail {
    /// Stop execution on any error (default)
    False,
    /// Continue execution regardless of errors
    True,
    /// Continue only for specific error types
    Selective { allowed_errors: Vec<String> },
    /// Continue except for specific error types
    Except { blocked_errors: Vec<String> },
}

impl Default for ContinueOnFail {
    fn default() -> Self {
        Self::False
    }
}

/// Error trigger configuration for executing workflows on failure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorTrigger {
    /// The workflow ID to execute when this error trigger fires
    pub workflow_id: Option<String>,
    /// Error types that should trigger this workflow
    pub error_types: Vec<ErrorType>,
    /// Whether to execute the trigger synchronously (wait for completion)
    pub sync_execution: bool,
}

/// Types of errors that can be caught.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ErrorType {
    /// Node execution failed
    NodeExecution,
    /// Workflow activation failed
    WorkflowActivation,
    /// Authentication/credential error
    Authentication,
    /// Rate limiting error
    RateLimit,
    /// Network/timeout error
    NetworkTimeout,
    /// Validation error
    Validation,
    /// Expression evaluation error
    Expression,
    /// Catch-all for any error
    Any,
}

/// Execution context for error handling.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorContext {
    /// The run ID where the error occurred
    pub run_id: RunId,
    /// The node that caused the error
    pub node_name: String,
    /// The type of error
    pub error_type: ErrorType,
    /// The error message
    pub error_message: String,
    /// Timestamp of the error
    pub timestamp: i64,
    /// Additional error metadata
    pub metadata: Option<serde_json::Value>,
}

impl ErrorContext {
    /// Create a new error context from a BarqError.
    pub fn from_error(
        run_id: RunId,
        node_name: String,
        error: &BarqError,
    ) -> Self {
        let (error_type, error_message) = Self::classify_error(error);

        Self {
            run_id,
            node_name,
            error_type,
            error_message,
            timestamp: chrono::Utc::now().timestamp(),
            metadata: None,
        }
    }

    /// Classify a BarqError into ErrorType and extract message.
    fn classify_error(error: &BarqError) -> (ErrorType, String) {
        match error {
            BarqError::NodeOperationError { .. } => (ErrorType::NodeExecution, error.to_string()),
            BarqError::NodeApiError { .. } => (ErrorType::NodeExecution, error.to_string()),
            BarqError::WorkflowActivationError { .. } => (ErrorType::WorkflowActivation, error.to_string()),
            BarqError::ExpressionError { .. } => (ErrorType::Expression, error.to_string()),
            BarqError::WorkflowConfigurationError { .. } => (ErrorType::Validation, error.to_string()),
            _ => (ErrorType::Any, error.to_string()),
        }
    }

    /// Check if this error matches any of the given error types.
    pub fn matches_types(&self, types: &[ErrorType]) -> bool {
        types.iter().any(|t| matches!(t, ErrorType::Any) || t == &self.error_type)
    }
}

/// Error handler for managing node failures and error workflows.
pub struct ErrorHandler {
    /// Global continue on fail setting
    continue_on_fail: ContinueOnFail,
    /// Error triggers registered by node name
    error_triggers: HashMap<String, ErrorTrigger>,
}

impl ErrorHandler {
    /// Create a new error handler.
    pub fn new(continue_on_fail: ContinueOnFail) -> Self {
        Self {
            continue_on_fail,
            error_triggers: HashMap::new(),
        }
    }

    /// Register an error trigger for a specific node.
    pub fn register_trigger(&mut self, node_name: String, trigger: ErrorTrigger) {
        self.error_triggers.insert(node_name, trigger);
    }

    /// Handle an error that occurred during node execution.
    ///
    /// # Arguments
    /// * `context` - The error context
    ///
    /// # Returns
    /// - `Ok(true)` if execution should continue
    /// - `Ok(false)` if execution should stop
    /// - `Err(_)` if the error handler itself failed
    pub fn handle_error(&self, context: &ErrorContext) -> Result<bool, String> {
        warn!("Error in node '{}': {}", context.node_name, context.error_message);

        // Check if we should continue on fail
        let should_continue = self.should_continue(context);

        // Check for error triggers
        if let Some(trigger) = self.error_triggers.get(&context.node_name) {
            if context.matches_types(&trigger.error_types) {
                debug!("Executing error workflow for node '{}'", context.node_name);
                // In a real implementation, this would queue the error workflow
                // For now, we just log it
                if let Some(workflow_id) = &trigger.workflow_id {
                    debug!("Would execute error workflow: {}", workflow_id);
                }
            }
        }

        Ok(should_continue)
    }

    /// Determine if execution should continue based on settings.
    fn should_continue(&self, context: &ErrorContext) -> bool {
        match &self.continue_on_fail {
            ContinueOnFail::True => true,
            ContinueOnFail::False => false,
            ContinueOnFail::Selective { allowed_errors } => {
                let error_str = self.error_type_to_string(&context.error_type);
                allowed_errors.iter().any(|e| e == "any" || e == &error_str)
            }
            ContinueOnFail::Except { blocked_errors } => {
                let error_str = self.error_type_to_string(&context.error_type);
                !blocked_errors.iter().any(|e| e == &error_str)
            }
        }
    }

    fn error_type_to_string(&self, error_type: &ErrorType) -> String {
        match error_type {
            ErrorType::NodeExecution => "nodeExecution".to_string(),
            ErrorType::WorkflowActivation => "workflowActivation".to_string(),
            ErrorType::Authentication => "authentication".to_string(),
            ErrorType::RateLimit => "rateLimit".to_string(),
            ErrorType::NetworkTimeout => "networkTimeout".to_string(),
            ErrorType::Validation => "validation".to_string(),
            ErrorType::Expression => "expression".to_string(),
            ErrorType::Any => "any".to_string(),
        }
    }

    /// Convert a BarqError to an ErrorOutput for routing to error output index.
    pub fn error_to_output(&self, error: &BarqError) -> serde_json::Value {
        serde_json::json!({
            "error": true,
            "message": error.to_string(),
            "timestamp": chrono::Utc::now().to_rfc3339()
        })
    }
}

impl Default for ErrorHandler {
    fn default() -> Self {
        Self::new(ContinueOnFail::default())
    }
}

/// Configuration for node-level error handling.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeErrorConfig {
    /// Continue on fail setting for this node
    pub continue_on_fail: Option<ContinueOnFail>,
    /// Error output configuration
    pub error_output: Option<ErrorOutput>,
    /// Retry configuration
    pub retry_config: Option<RetryConfig>,
}

/// Retry configuration for failed nodes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Delay between retries in milliseconds
    pub retry_delay_ms: u64,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
}

impl RetryConfig {
    /// Calculate delay for a given retry attempt.
    pub fn calculate_delay(&self, attempt: u32) -> tokio::time::Duration {
        let delay_ms = if self.exponential_backoff {
            self.retry_delay_ms * 2_u64.pow(attempt.min(10))
        } else {
            self.retry_delay_ms
        };
        tokio::time::Duration::from_millis(delay_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_continue_on_fail_default() {
        let config = ContinueOnFail::default();
        assert_eq!(config, ContinueOnFail::False);
    }

    #[test]
    fn test_error_context_from_barq_error() {
        let run_id = RunId::new();
        let error = BarqError::NodeOperationError {
            node_name: "TestNode".to_string(),
            message: "Test error".to_string(),
        };

        let context = ErrorContext::from_error(run_id, "TestNode".to_string(), &error);

        assert_eq!(context.node_name, "TestNode");
        assert_eq!(context.error_type, ErrorType::NodeExecution);
    }

    #[test]
    fn test_error_context_matches_types() {
        let run_id = RunId::new();
        let error = BarqError::ExpressionError {
            node_name: "ExprNode".to_string(),
            message: "Bad expression".to_string(),
        };

        let context = ErrorContext::from_error(run_id, "ExprNode".to_string(), &error);

        assert!(context.matches_types(&[ErrorType::Expression]));
        assert!(context.matches_types(&[ErrorType::Any]));
        assert!(!context.matches_types(&[ErrorType::NodeExecution]));
    }

    #[test]
    fn test_error_handler_continue_on_fail_true() {
        let handler = ErrorHandler::new(ContinueOnFail::True);
        let run_id = RunId::new();
        let error = BarqError::NodeOperationError {
            node_name: "TestNode".to_string(),
            message: "Test error".to_string(),
        };

        let context = ErrorContext::from_error(run_id, "TestNode".to_string(), &error);
        let result = handler.handle_error(&context).unwrap();

        assert!(result);
    }

    #[test]
    fn test_error_handler_continue_on_fail_false() {
        let handler = ErrorHandler::new(ContinueOnFail::False);
        let run_id = RunId::new();
        let error = BarqError::NodeOperationError {
            node_name: "TestNode".to_string(),
            message: "Test error".to_string(),
        };

        let context = ErrorContext::from_error(run_id, "TestNode".to_string(), &error);
        let result = handler.handle_error(&context).unwrap();

        assert!(!result);
    }

    #[test]
    fn test_error_handler_selective_continue() {
        let handler = ErrorHandler::new(ContinueOnFail::Selective {
            allowed_errors: vec!["networkTimeout".to_string(), "any".to_string()],
        });
        let run_id = RunId::new();

        // Test with "any" in allowed list - should continue for any error
        let network_error = BarqError::NodeApiError {
            node_name: "TestNode".to_string(),
            message: "Network timeout".to_string(),
        };
        let context = ErrorContext::from_error(run_id, "TestNode".to_string(), &network_error);

        // Since "any" is in the allowed list, this should continue (true)
        let result = handler.handle_error(&context).unwrap();
        assert!(result);

        // Test without "any" - should not continue for non-matching errors
        let handler2 = ErrorHandler::new(ContinueOnFail::Selective {
            allowed_errors: vec!["networkTimeout".to_string()],
        });
        let result2 = handler2.handle_error(&context).unwrap();
        // NodeExecution doesn't match networkTimeout, so don't continue
        assert!(!result2);
    }

    #[test]
    fn test_error_handler_except_blocked() {
        let handler = ErrorHandler::new(ContinueOnFail::Except {
            blocked_errors: vec!["authentication".to_string()],
        });
        let run_id = RunId::new();
        let error = BarqError::NodeOperationError {
            node_name: "TestNode".to_string(),
            message: "Test error".to_string(),
        };

        let context = ErrorContext::from_error(run_id, "TestNode".to_string(), &error);
        let result = handler.handle_error(&context).unwrap();

        // NodeExecution is not in blocked list, so continue
        assert!(result);
    }

    #[test]
    fn test_error_trigger_registration() {
        let mut handler = ErrorHandler::new(ContinueOnFail::False);
        let trigger = ErrorTrigger {
            workflow_id: Some("error-workflow".to_string()),
            error_types: vec![ErrorType::NodeExecution],
            sync_execution: true,
        };

        handler.register_trigger("RiskyNode".to_string(), trigger);
        assert!(handler.error_triggers.contains_key("RiskyNode"));
    }

    #[test]
    fn test_retry_config_calculate_delay() {
        let config = RetryConfig {
            max_retries: 3,
            retry_delay_ms: 1000,
            exponential_backoff: false,
        };

        assert_eq!(config.calculate_delay(0).as_millis(), 1000);
        assert_eq!(config.calculate_delay(1).as_millis(), 1000);
    }

    #[test]
    fn test_retry_config_exponential_backoff() {
        let config = RetryConfig {
            max_retries: 3,
            retry_delay_ms: 1000,
            exponential_backoff: true,
        };

        assert_eq!(config.calculate_delay(0).as_millis(), 1000);
        assert_eq!(config.calculate_delay(1).as_millis(), 2000);
        assert_eq!(config.calculate_delay(2).as_millis(), 4000);
        assert_eq!(config.calculate_delay(3).as_millis(), 8000);
    }

    #[test]
    fn test_error_to_output() {
        let handler = ErrorHandler::new(ContinueOnFail::False);
        let error = BarqError::WorkflowConfigurationError {
            message: "Invalid workflow".to_string(),
        };

        let output = handler.error_to_output(&error);
        assert!(output.get("error").unwrap().as_bool().unwrap());
        assert!(output.get("message").is_some());
        assert!(output.get("timestamp").is_some());
    }

    #[test]
    fn test_continue_on_fail_serialization() {
        let config = ContinueOnFail::Selective {
            allowed_errors: vec!["networkTimeout".to_string()],
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("selective"));
        assert!(json.contains("networkTimeout"));
    }
}
