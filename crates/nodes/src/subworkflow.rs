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
            "description": "Execute another workflow"
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

        // We fetch the current input data that triggered this node, assuming index 0.
        // IExecuteFunctions needs to be able to fetch "all items from input index 0" or we can just 
        // rely on the Runner loop data injection for ExecuteSubWorkflow.
        let items = context.get_input_data(0);
        let mut extracted_items = Vec::new();

        for item in items.map_or(vec![], |v| v.to_vec()) {
            extracted_items.push(item);
        }

        let input_data = serde_json::to_value(&extracted_items).unwrap_or(serde_json::Value::Null);

        // Throw specialized error to trap execution and force runner to process sub workflow
        Err(BarqError::ExecuteSubWorkflow {
            workflow_id,
            input_data,
        })
    }
}
