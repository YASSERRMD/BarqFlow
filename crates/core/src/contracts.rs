use crate::types::{IDataObject, NodeId, RunId, WorkflowId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// High-level status of an execution as exposed through BarqFlow contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExecutionStatus {
    Queued,
    Running,
    Waiting,
    Success,
    Failed,
    Stopped,
    Cancelled,
}

/// Lifecycle event types emitted for an execution stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ExecutionEventType {
    Queued,
    Started,
    NodeStarted,
    NodeFinished,
    Waiting,
    Resumed,
    Failed,
    Stopped,
    Completed,
}

/// Wire contract for execution lifecycle events.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionEvent {
    pub execution_id: Uuid,
    pub workflow_id: WorkflowId,
    pub run_id: RunId,
    pub event_type: ExecutionEventType,
    pub status: ExecutionStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_id: Option<NodeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_name: Option<String>,
    pub message: String,
    pub timestamp: DateTime<Utc>,
    pub sequence: u64,
    #[serde(default)]
    pub data: IDataObject,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn execution_event_serializes_as_camel_case_contract() {
        let event = ExecutionEvent {
            execution_id: Uuid::parse_str("f3180e91-f1fb-4c8f-9108-c10840d1943a").unwrap(),
            workflow_id: WorkflowId(
                Uuid::parse_str("7f595f8b-61d2-4cb3-b9f7-d4590f258fb2").unwrap(),
            ),
            run_id: RunId(Uuid::parse_str("4be8e6d1-a5d0-45b0-9ad0-9891503176de").unwrap()),
            event_type: ExecutionEventType::NodeFinished,
            status: ExecutionStatus::Running,
            node_id: Some(NodeId::new("http-request-1")),
            node_name: Some("HTTP Request".to_string()),
            message: "Node completed".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2026-03-10T10:05:00Z")
                .unwrap()
                .with_timezone(&Utc),
            sequence: 12,
            data: IDataObject::from(serde_json::json!({
                "outputItems": 1
            })),
        };

        let value = serde_json::to_value(&event).unwrap();

        assert_eq!(value["executionId"], "f3180e91-f1fb-4c8f-9108-c10840d1943a");
        assert_eq!(value["workflowId"], "7f595f8b-61d2-4cb3-b9f7-d4590f258fb2");
        assert_eq!(value["runId"], "4be8e6d1-a5d0-45b0-9ad0-9891503176de");
        assert_eq!(value["eventType"], "nodeFinished");
        assert_eq!(value["status"], "running");
        assert_eq!(value["nodeId"], "http-request-1");
        assert_eq!(value["nodeName"], "HTTP Request");
        assert_eq!(value["data"]["outputItems"], 1);
    }
}
