use async_trait::async_trait;
use barqflow_core::errors::BarqError;
use barqflow_core::schema::INodeExecutionData;
use barqflow_core::traits::{IExecuteFunctions, INodeType};
use barqflow_core::types::IDataObject;
use uuid::Uuid;

pub struct WaitNode;

#[async_trait]
impl INodeType for WaitNode {
    fn get_description(&self) -> IDataObject {
        IDataObject::from(serde_json::json!({
            "name": "Wait",
            "description": "Suspend execution for a specific time or event"
        }))
    }

    async fn execute(
        &self,
        context: &dyn IExecuteFunctions,
    ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
        let resume_string = context
            .get_node_parameter("resume", None)
            .await
            .map(|v| v.as_str().unwrap_or("time").to_string())
            .unwrap_or_else(|_| "time".to_string());

        let wait_config = if resume_string == "time" {
            let amount = context
                .get_node_parameter("amount", None)
                .await
                .unwrap_or(serde_json::json!(1))
                .as_u64()
                .unwrap_or(1);
            let unit = context
                .get_node_parameter("unit", None)
                .await
                .map(|v| v.as_str().unwrap_or("seconds").to_string())
                .unwrap_or_else(|_| "seconds".to_string());

            let multiplier = match unit.as_str() {
                "milliseconds" => 1,
                "seconds" => 1000,
                "minutes" => 60000,
                "hours" => 3600000,
                "days" => 86400000,
                _ => 1000,
            };

            serde_json::json!({
                "waitType": "time",
                "durationMs": amount * multiplier,
                "webhookPath": null,
                "externalId": null
            })
        } else if resume_string == "webhook" {
            let resume_token = Uuid::new_v4().to_string();
            serde_json::json!({
                "waitType": "webhook",
                "durationMs": null,
                "webhookPath": resume_token,
                "externalId": null
            })
        } else {
            return Err(BarqError::NodeOperationError {
                node_name: "Wait".to_string(),
                message: format!("Unsupported resume condition: {}", resume_string),
            });
        };

        let node_name = context.get_node().name.clone();

        // Throw the suspend error to trap the execution stack gracefully inside run_node
        Err(BarqError::SuspendExecution {
            node_name,
            wait_config,
        })
    }
}
