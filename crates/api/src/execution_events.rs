use async_trait::async_trait;
use barqflow_core::contracts::ExecutionEvent;
use barqflow_exec::runner::ExecutionEventReporter;
use serde_json::{json, Value};
use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

pub const EXECUTION_META_KEY: &str = "__barqflow";
const EXECUTION_EVENTS_KEY: &str = "events";

#[derive(Clone)]
struct ExecutionEventSession {
    sender: broadcast::Sender<ExecutionEvent>,
    history: Arc<RwLock<Vec<ExecutionEvent>>>,
}

#[derive(Clone, Default)]
pub struct ExecutionEventHub {
    sessions: Arc<RwLock<HashMap<Uuid, ExecutionEventSession>>>,
}

impl ExecutionEventHub {
    pub fn new() -> Self {
        Self::default()
    }

    async fn ensure_session(&self, execution_id: Uuid) -> ExecutionEventSession {
        if let Some(existing) = self.sessions.read().await.get(&execution_id).cloned() {
            return existing;
        }

        let mut sessions = self.sessions.write().await;
        sessions
            .entry(execution_id)
            .or_insert_with(|| {
                let (sender, _) = broadcast::channel(256);
                ExecutionEventSession {
                    sender,
                    history: Arc::new(RwLock::new(Vec::new())),
                }
            })
            .clone()
    }

    pub async fn append(&self, event: ExecutionEvent) {
        let session = self.ensure_session(event.execution_id).await;
        {
            let mut history = session.history.write().await;
            history.push(event.clone());
        }
        let _ = session.sender.send(event);
    }

    pub async fn snapshot(&self, execution_id: Uuid) -> Vec<ExecutionEvent> {
        let Some(session) = self.sessions.read().await.get(&execution_id).cloned() else {
            return Vec::new();
        };
        let history = session.history.read().await.clone();
        sort_and_dedup_events(history)
    }

    pub async fn subscribe(&self, execution_id: Uuid) -> broadcast::Receiver<ExecutionEvent> {
        self.ensure_session(execution_id).await.sender.subscribe()
    }

    pub async fn remove(&self, execution_id: Uuid) {
        self.sessions.write().await.remove(&execution_id);
    }
}

#[derive(Clone)]
pub struct HubExecutionEventReporter {
    hub: ExecutionEventHub,
}

impl HubExecutionEventReporter {
    pub fn new(hub: ExecutionEventHub) -> Self {
        Self { hub }
    }
}

#[async_trait]
impl ExecutionEventReporter for HubExecutionEventReporter {
    async fn report(&self, event: ExecutionEvent) {
        self.hub.append(event).await;
    }
}

pub fn extract_execution_events(payload: &Value) -> Vec<ExecutionEvent> {
    let Some(root) = payload.as_object() else {
        return Vec::new();
    };

    let Some(meta) = root.get(EXECUTION_META_KEY).and_then(|value| value.as_object()) else {
        return Vec::new();
    };

    let Some(raw_events) = meta.get(EXECUTION_EVENTS_KEY) else {
        return Vec::new();
    };

    let parsed = serde_json::from_value::<Vec<ExecutionEvent>>(raw_events.clone()).unwrap_or_default();
    sort_and_dedup_events(parsed)
}

pub fn with_execution_event_history(payload: Value, events: &[ExecutionEvent]) -> Value {
    let mut root = match payload {
        Value::Object(map) => map,
        Value::Null => serde_json::Map::new(),
        other => {
            let mut map = serde_json::Map::new();
            map.insert("value".to_string(), other);
            map
        }
    };

    let mut meta = root
        .remove(EXECUTION_META_KEY)
        .and_then(|value| value.as_object().cloned())
        .unwrap_or_default();

    let normalized_events = sort_and_dedup_events(events.to_vec());
    meta.insert(
        EXECUTION_EVENTS_KEY.to_string(),
        serde_json::to_value(&normalized_events).unwrap_or_else(|_| Value::Array(Vec::new())),
    );
    meta.insert("eventCount".to_string(), json!(normalized_events.len()));
    if let Some(last_event) = normalized_events.last() {
        meta.insert(
            "lastEventType".to_string(),
            serde_json::to_value(last_event.event_type).unwrap_or(Value::Null),
        );
        meta.insert(
            "lastStatus".to_string(),
            serde_json::to_value(last_event.status).unwrap_or(Value::Null),
        );
        meta.insert("lastTimestamp".to_string(), json!(last_event.timestamp));
    }

    root.insert(EXECUTION_META_KEY.to_string(), Value::Object(meta));
    Value::Object(root)
}

pub fn merge_execution_events(existing: Vec<ExecutionEvent>, incoming: Vec<ExecutionEvent>) -> Vec<ExecutionEvent> {
    let mut merged = existing;
    merged.extend(incoming);
    sort_and_dedup_events(merged)
}

fn sort_and_dedup_events(events: Vec<ExecutionEvent>) -> Vec<ExecutionEvent> {
    let mut by_sequence = BTreeMap::new();
    for event in events {
        by_sequence.insert(event.sequence, event);
    }
    by_sequence.into_values().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use barqflow_core::contracts::{ExecutionEventType, ExecutionStatus};
    use barqflow_core::types::{IDataObject, RunId, WorkflowId};
    use chrono::{DateTime, Utc};
    use uuid::Uuid;

    fn sample_event(sequence: u64) -> ExecutionEvent {
        ExecutionEvent {
            execution_id: Uuid::parse_str("f3180e91-f1fb-4c8f-9108-c10840d1943a").unwrap(),
            workflow_id: WorkflowId(
                Uuid::parse_str("7f595f8b-61d2-4cb3-b9f7-d4590f258fb2").unwrap(),
            ),
            run_id: RunId(Uuid::parse_str("4be8e6d1-a5d0-45b0-9ad0-9891503176de").unwrap()),
            event_type: ExecutionEventType::NodeFinished,
            status: ExecutionStatus::Running,
            node_id: None,
            node_name: Some("HTTP Request".to_string()),
            message: "Node completed".to_string(),
            timestamp: DateTime::parse_from_rfc3339("2026-03-10T10:05:00Z")
                .unwrap()
                .with_timezone(&Utc),
            sequence,
            data: IDataObject::from(json!({ "outputItems": 1 })),
        }
    }

    #[test]
    fn event_history_round_trips_through_execution_payload() {
        let payload = json!({
            "HTTP Request": {
                "success": true
            }
        });
        let events = vec![sample_event(2), sample_event(1)];

        let decorated = with_execution_event_history(payload, &events);
        let extracted = extract_execution_events(&decorated);

        assert_eq!(extracted.len(), 2);
        assert_eq!(extracted[0].sequence, 1);
        assert_eq!(extracted[1].sequence, 2);
        assert!(decorated["__barqflow"]["eventCount"].is_number());
    }

    #[test]
    fn merge_execution_events_deduplicates_by_sequence() {
        let first = vec![sample_event(1), sample_event(2)];
        let second = vec![sample_event(2), sample_event(3)];

        let merged = merge_execution_events(first, second);

        assert_eq!(merged.len(), 3);
        assert_eq!(merged[0].sequence, 1);
        assert_eq!(merged[2].sequence, 3);
    }
}
