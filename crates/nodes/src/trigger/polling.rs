use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType, IPollFunctions};
use barqflow_core::types::IDataObject;
use serde_json::json;

/// A simple polling trigger node used to simulate retrieving events dynamically
pub struct MockPollingTriggerNode;

#[async_trait]
impl INodeType for MockPollingTriggerNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(json!({
            "name": "mockPollingTrigger",
            "displayName": "Mock Polling Trigger",
            "description": "A node that simulates retrieving events periodically",
            "isTrigger": true,
            "maxInputs": 0,
            "maxOutputs": 1
        }))
    }

    async fn execute(
        &self,
        _context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        // Triggers typically do not invoke "execute", but rather start via the core event loop.
        Ok(vec![vec![]])
    }

    async fn poll(&self, context: &dyn IPollFunctions) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        // Read previous interval's stored data length
        let poll_data = context.get_poll_data().await?;
        let previous_count = poll_data.0.get("count").and_then(|v| v.as_u64()).unwrap_or(0);

        // Save updated data
        let mut new_poll_data = IDataObject::default();
        new_poll_data.0.insert("count".to_string(), json!(previous_count + 1));
        context.set_poll_data(new_poll_data).await?;

        // Return the count as the newly emitted event
        let mut output_item = IDataObject::default();
        output_item.0.insert("event_id".to_string(), json!(previous_count + 1));
        let exec_data = INodeExecutionData::new(output_item);

        Ok(vec![vec![exec_data]])
    }
}
