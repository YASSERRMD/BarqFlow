//! Error Handling for Workflow Execution
//!
//! Implements error triggers, handlers, and Continue On Fail functionality.

use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::IExecuteFunctions;
use barqflow_core::types::IDataObject;
use serde::{Deserialize, Serialize};

/// Configuration for error trigger node
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorTriggerConfig {
    pub error_message_contains: Option<String>,
    pub error_message_equals: Option<String>,
    pub workflow_id: Option<String>,
}

/// Configuration for error handler
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorHandlerConfig {
    pub continue_on_fail: bool,
    pub max_retries: Option<u32>,
    pub retry_interval_ms: Option<u64>,
}

/// Error trigger node - spawns error workflow on failure
pub struct ErrorTriggerNode;

#[async_trait]
impl barqflow_core::traits::INodeType for ErrorTriggerNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "Error Trigger",
            "description": "Triggers error workflow on failure"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let error_msg = context
            .get_node_parameter("errorMessage", None)
            .await
            .map(|v| v.as_str().unwrap_or("").to_string())
            .unwrap_or_default();

        let continue_on_fail = context
            .get_node_parameter("continueOnFail", None)
            .await
            .map(|v| v.as_bool().unwrap_or(false))
            .unwrap_or(false);

        if !continue_on_fail && !error_msg.is_empty() {
            return Err(BarqError::NodeOperationError {
                node_name: "ErrorTrigger".to_string(),
                message: error_msg,
            });
        }

        Ok(vec![vec![]])
    }
}

/// Error handler wrapper for nodes
#[derive(Debug, Clone)]
pub struct ErrorHandler {
    pub continue_on_fail: bool,
    pub max_retries: u32,
    pub retry_interval_ms: u64,
}

impl ErrorHandler {
    pub fn new(continue_on_fail: bool) -> Self {
        Self {
            continue_on_fail,
            max_retries: 0,
            retry_interval_ms: 0,
        }
    }

    pub fn with_retries(mut self, max_retries: u32, interval_ms: u64) -> Self {
        self.max_retries = max_retries;
        self.retry_interval_ms = interval_ms;
        self
    }

    pub async fn execute_with_retry<F, T, E>(&self, mut operation: F) -> Result<T, E>
    where
        F: FnMut() -> Result<T, E>,
        E: std::fmt::Debug,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            match operation() {
                Ok(result) => return Ok(result),
                Err(e) if attempt < self.max_retries => {
                    last_error = Some(e);
                    if self.retry_interval_ms > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            self.retry_interval_ms,
                        ))
                        .await;
                    }
                }
                Err(e) => return Err(e),
            }
        }

        Err(last_error.unwrap())
    }

    /// Executes an async operation with retry logic and isolates panics per task
    pub async fn execute_isolated_async<F, Fut, T>(
        &self,
        node_name: &str,
        mut operation: F,
    ) -> Result<T, BarqError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, BarqError>>,
    {
        use futures::FutureExt;
        use std::panic::AssertUnwindSafe;

        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            let fut = operation();

            // Wrap in AssertUnwindSafe to catch async task panics without killing the root thread
            let execute_future = AssertUnwindSafe(fut).catch_unwind();

            let execute_result: Result<T, BarqError> = match execute_future.await {
                Ok(Ok(result)) => return Ok(result),
                Ok(Err(e)) => Err(e),
                Err(panic_err) => {
                    let panic_msg = if let Some(s) = panic_err.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = panic_err.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic".to_string()
                    };

                    Err(BarqError::NodeOperationError {
                        node_name: node_name.to_string(),
                        message: format!("Node execution panicked: {}", panic_msg),
                    })
                }
            };

            // Trap error and handle retries
            match execute_result {
                Err(e) if attempt < self.max_retries => {
                    last_error = Some(e);
                    if self.retry_interval_ms > 0 {
                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            self.retry_interval_ms,
                        ))
                        .await;
                    }
                }
                Err(e) => return Err(e),
                _ => unreachable!(),
            }
        }

        Err(last_error.unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_handler_creation() {
        let handler = ErrorHandler::new(true);
        assert!(handler.continue_on_fail);
        assert_eq!(handler.max_retries, 0);
    }

    #[test]
    fn test_error_handler_with_retries() {
        let handler = ErrorHandler::new(false).with_retries(3, 100);
        assert!(!handler.continue_on_fail);
        assert_eq!(handler.max_retries, 3);
        assert_eq!(handler.retry_interval_ms, 100);
    }
}
