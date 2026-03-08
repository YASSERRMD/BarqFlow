use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;

pub struct ExecuteWorkflowNode;

#[async_trait]
impl INodeType for ExecuteWorkflowNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "executeWorkflow",
            "displayName": "Execute Workflow",
            "description": "Execute another workflow and pass through its output"
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

        if workflow_id.is_empty() {
            return Err(BarqError::NodeOperationError {
                node_name: "executeWorkflow".to_string(),
                message: "A target workflow ID must be provided".to_string(),
            });
        }

        let _mode = context
            .get_node_parameter("mode", Some(serde_json::json!("wait")))
            .await
            .ok()
            .and_then(|v| v.as_str().map(|s| s.to_string()))
            .unwrap_or_else(|| "wait".to_string());

        let default_input_items = context.get_input_data(0).await.unwrap_or_default();
        let default_input =
            serde_json::to_value(&default_input_items).unwrap_or(serde_json::Value::Null);

        let input_data = match context.get_node_parameter("inputData", None).await {
            Ok(custom) => {
                if custom.is_null() {
                    default_input
                } else if let Some(raw) = custom.as_str() {
                    let trimmed = raw.trim();
                    if trimmed.is_empty() {
                        default_input
                    } else {
                        serde_json::from_str(trimmed).unwrap_or_else(|_| custom.clone())
                    }
                } else {
                    custom
                }
            }
            Err(_) => default_input,
        };

        Err(BarqError::ExecuteSubWorkflow {
            workflow_id,
            input_data,
        })
    }
}
