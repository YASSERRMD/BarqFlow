//! Trigger Nodes
//!
//! Implements manual and webhook trigger nodes.

use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType, IPollFunctions};
use barqflow_core::types::IDataObject;

pub struct ManualTriggerNode;

#[async_trait]
impl INodeType for ManualTriggerNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "Manual Trigger",
            "description": "Triggered manually by user"
        }))
    }

    async fn execute(
        &self,
        _context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let output_item = INodeExecutionData::new(IDataObject::from(serde_json::json!({
            "triggered": true
        })));
        Ok(vec![vec![output_item]])
    }
}

pub struct WebhookNode {
    pub method: Option<String>,
    pub path: Option<String>,
}

impl WebhookNode {
    pub fn new() -> Self {
        Self {
            method: None,
            path: None,
        }
    }

    pub fn with_method(mut self, method: &str) -> Self {
        self.method = Some(method.to_string());
        self
    }

    pub fn with_path(mut self, path: &str) -> Self {
        self.path = Some(path.to_string());
        self
    }
}

impl Default for WebhookNode {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ErrorTriggerNode;#[async_trait]
impl INodeType for ErrorTriggerNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "errorTrigger",
            "displayName": "Error Trigger",
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

#[async_trait]
impl INodeType for WebhookNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "Webhook",
            "description": "Triggered via webhook"
        }))
    }

    async fn execute(
        &self,
        _context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        Ok(vec![])
    }

    async fn poll(
        &self,
        _context: &dyn IPollFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let output_item = INodeExecutionData::new(IDataObject::from(serde_json::json!({
            "webhook": true
        })));
        Ok(vec![vec![output_item]])
    }
}

pub struct CronTriggerNode {
    pub cron_expression: String,
}

impl CronTriggerNode {
    pub fn new(cron: &str) -> Self {
        Self {
            cron_expression: cron.to_string(),
        }
    }
}

#[async_trait]
impl INodeType for CronTriggerNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "Cron Trigger",
            "description": "Triggers on schedule"
        }))
    }

    async fn execute(
        &self,
        _context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        Ok(vec![])
    }

    async fn poll(
        &self,
        _context: &dyn IPollFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let output_item = INodeExecutionData::new(IDataObject::from(serde_json::json!({
            "cron": self.cron_expression
        })));
        Ok(vec![vec![output_item]])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_trigger_description() {
        let node = ManualTriggerNode;
        let desc = node.get_description();
        assert_eq!(desc.0.get("name").unwrap(), "Manual Trigger");
    }

    #[test]
    fn test_webhook_node() {
        let node = WebhookNode::new().with_method("POST").with_path("/webhook");
        assert_eq!(node.method.unwrap(), "POST");
        assert_eq!(node.path.unwrap(), "/webhook");
    }

    #[test]
    fn test_cron_trigger() {
        let node = CronTriggerNode::new("0 * * * *");
        assert_eq!(node.cron_expression, "0 * * * *");
    }
}
