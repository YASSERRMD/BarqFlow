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

    async fn configured_value(
        context: &dyn IPollFunctions,
        key: &str,
        fallback: Option<&str>,
    ) -> String {
        context
            .get_node_parameter(key, None)
            .await
            .ok()
            .and_then(|value| value.as_str().map(|raw| raw.trim().to_string()))
            .filter(|value| !value.is_empty())
            .or_else(|| fallback.map(|value| value.to_string()))
            .unwrap_or_default()
    }
}

impl Default for WebhookNode {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ErrorTriggerNode;
#[async_trait]
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
        context: &dyn IPollFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let path = Self::configured_value(context, "path", self.path.as_deref()).await;
        let http_method =
            Self::configured_value(context, "httpMethod", self.method.as_deref()).await;
        let response_mode =
            Self::configured_value(context, "responseMode", Some("onReceived")).await;

        let output_item = INodeExecutionData::new(IDataObject::from(serde_json::json!({
            "webhook": true,
            "path": path,
            "httpMethod": if http_method.is_empty() { "ANY" } else { http_method.as_str() },
            "responseMode": if response_mode.is_empty() { "onReceived" } else { response_mode.as_str() },
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

    async fn resolve_cron_expression(&self, context: &dyn IPollFunctions) -> String {
        for key in ["cron", "cronExpression", "expression"] {
            if let Ok(value) = context.get_node_parameter(key, None).await {
                if let Some(expression) = value
                    .as_str()
                    .map(|raw| raw.trim().to_string())
                    .filter(|value| !value.is_empty())
                {
                    return expression;
                }
            }
        }

        self.cron_expression.clone()
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
        context: &dyn IPollFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let cron_expression = self.resolve_cron_expression(context).await;
        let output_item = INodeExecutionData::new(IDataObject::from(serde_json::json!({
            "cron": cron_expression,
            "triggerType": "cron"
        })));
        Ok(vec![vec![output_item]])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_core::schema::INode;
    use barqflow_core::types::GenericValue;
    use std::collections::HashMap;

    struct MockPollContext {
        node: INode,
        poll_data: IDataObject,
        params: HashMap<String, GenericValue>,
    }

    #[async_trait]
    impl IPollFunctions for MockPollContext {
        async fn get_poll_data(&self) -> Result<IDataObject, BarqError> {
            Ok(self.poll_data.clone())
        }

        async fn set_poll_data(&self, _data: IDataObject) -> Result<(), BarqError> {
            Ok(())
        }

        async fn get_node_parameter(
            &self,
            parameter_name: &str,
            fallback_value: Option<GenericValue>,
        ) -> Result<GenericValue, BarqError> {
            self.params
                .get(parameter_name)
                .cloned()
                .or(fallback_value)
                .ok_or_else(|| BarqError::NodeOperationError {
                    node_name: self.node.name.clone(),
                    message: format!("Parameter '{}' not found", parameter_name),
                })
        }

        fn get_node(&self) -> &INode {
            &self.node
        }

        async fn get_credentials(
            &self,
            _name: &str,
        ) -> Result<HashMap<String, GenericValue>, BarqError> {
            Ok(HashMap::new())
        }

        fn log(&self, _message: &str) {}
    }

    fn mock_poll_context(params: HashMap<String, GenericValue>) -> MockPollContext {
        MockPollContext {
            node: INode {
                id: barqflow_core::types::NodeId::new("trigger-node"),
                name: "Trigger".to_string(),
                r#type: "barqflow-nodes.trigger".to_string(),
                type_version: 1.0,
                position: [0.0, 0.0],
                parameters: barqflow_core::schema::INodeParameters::default(),
                credentials: vec![],
                disabled: false,
            },
            poll_data: IDataObject::default(),
            params,
        }
    }

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

    #[tokio::test]
    async fn test_cron_poll_uses_configured_expression() {
        let node = CronTriggerNode::new("0 * * * * *");
        let context = mock_poll_context(HashMap::from([(
            "cron".to_string(),
            serde_json::json!("0 */5 * * * *"),
        )]));

        let result = node.poll(&context).await.unwrap();
        assert_eq!(result[0][0].json.0["cron"], "0 */5 * * * *");
        assert_eq!(result[0][0].json.0["triggerType"], "cron");
    }

    #[tokio::test]
    async fn test_webhook_poll_surfaces_runtime_configuration() {
        let node = WebhookNode::new().with_method("POST").with_path("/incoming");
        let context = mock_poll_context(HashMap::from([
            ("path".to_string(), serde_json::json!("orders/new")),
            ("httpMethod".to_string(), serde_json::json!("PATCH")),
            ("responseMode".to_string(), serde_json::json!("lastNode")),
        ]));

        let result = node.poll(&context).await.unwrap();
        let payload = &result[0][0].json.0;
        assert_eq!(payload["path"], "orders/new");
        assert_eq!(payload["httpMethod"], "PATCH");
        assert_eq!(payload["responseMode"], "lastNode");
    }
}
