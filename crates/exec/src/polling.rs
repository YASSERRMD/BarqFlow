use barqflow_core::errors::BarqError;
use barqflow_core::schema::{INodeExecutionData, WorkflowDef};
use barqflow_core::traits::INodeType;
use barqflow_core::types::IDataObject;
use barqflow_registry::registry::NodeRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration, MissedTickBehavior};
use tracing::{debug, error, info};

use crate::context::PollExecutionContext;
use crate::deduplication::{DeduplicationManager, DeduplicationMode};

pub struct ActivePoller {
    workflow_id: String,
    node_id: String,
}

pub struct PollingEngine {
    registry: Arc<NodeRegistry>,
    active_pollers: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
    static_data: Arc<RwLock<Option<IDataObject>>>,
}

impl PollingEngine {
    pub fn new(registry: Arc<NodeRegistry>, static_data: Arc<RwLock<Option<IDataObject>>>) -> Self {
        Self {
            registry,
            active_pollers: Arc::new(RwLock::new(HashMap::new())),
            static_data,
        }
    }

    /// Register a node for continuous polling.
    ///
    /// # Arguments
    /// * `workflow` - The parent workflow configuration
    /// * `node_id` - The specific ID of the polling Trigger configuration
    /// * `interval_seconds` - Time spanning between sequential trigger checks
    /// * `callback` - The function fired exclusively when new items are retrieved by the poller
    pub async fn register_poller<F>(
        &self,
        workflow: WorkflowDef,
        node_id: barqflow_core::types::NodeId,
        interval_seconds: u64,
        callback: F,
    ) -> Result<(), BarqError>
    where
        F: Fn(Vec<Vec<INodeExecutionData>>) + Send + Sync + 'static,
    {
        let workflow_id = workflow.id;

        let target_node = workflow
            .nodes
            .iter()
            .find(|n| n.id == node_id)
            .ok_or_else(|| BarqError::NodeOperationError {
                node_name: node_id.to_string(),
                message: "Target polling node not found in workflow definition".into(),
            })?
            .clone();

        let node_info = self
            .registry
            .get_node_by_name_with_fallback(&target_node.r#type, target_node.type_version)
            .ok_or_else(|| BarqError::NodeOperationError {
                node_name: target_node.name.clone(),
                message: format!(
                    "Polling Node type '{}' version {} not found in registry",
                    target_node.r#type, target_node.type_version
                ),
            })?;

        let node_impl = node_info.node_impl.clone();
        let static_data = self.static_data.clone();

        let mut interval_timer = interval(Duration::from_secs(interval_seconds));
        interval_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

        let mut dedup_mode_opt = None;
        let mut dedup_key_opt = None;
        if let Some(mode_val) = target_node.parameters.0.get("deduplicationMode") {
            if let Some(s) = mode_val.as_str() {
                match s {
                    "array_of_ids" => dedup_mode_opt = Some(DeduplicationMode::ArrayOfIds),
                    "incremented_key" => dedup_mode_opt = Some(DeduplicationMode::IncrementedKey),
                    _ => {}
                }
            }
        }
        if let Some(key_val) = target_node.parameters.0.get("deduplicationKey") {
            dedup_key_opt = key_val.as_str().map(|s| s.to_string());
        }

        let handle = tokio::spawn(async move {
            info!(
                "Started polling loop for node {} in workflow {} every {}s",
                target_node.name, workflow_id, interval_seconds
            );

            loop {
                interval_timer.tick().await;

                let ctx = PollExecutionContext::new(target_node.clone(), static_data.clone());
                match node_impl.poll(&ctx).await {
                    Ok(mut events) => {
                        // Triggers usually return empty 2D arrays if nothing new arrived
                        let has_items = events.iter().any(|arr| !arr.is_empty());
                        if has_items {
                            if let (Some(mode), Some(key_path)) = (&dedup_mode_opt, &dedup_key_opt)
                            {
                                let mut new_branches = Vec::new();
                                for branch in events {
                                    let branch_data: Vec<IDataObject> =
                                        branch.into_iter().map(|d| d.json).collect();

                                    let mut state_lock = static_data.write().await;
                                    if state_lock.is_none() {
                                        *state_lock = Some(IDataObject::default());
                                    }
                                    let global_state = state_lock.as_mut().unwrap();

                                    let dedup_state_key = format!("{}_dedup", target_node.id);
                                    let mut dedup_state = IDataObject::default();

                                    let map = &global_state.0;
                                    if let Some(v) = map.get(&dedup_state_key) {
                                        dedup_state = IDataObject::from(v.clone());
                                    }

                                    let filtered = DeduplicationManager::filter_new_events(
                                        branch_data,
                                        mode.clone(),
                                        key_path,
                                        &mut dedup_state,
                                    );

                                    global_state.0.insert(
                                        dedup_state_key,
                                        serde_json::Value::Object(dedup_state.0),
                                    );

                                    let new_branch: Vec<INodeExecutionData> =
                                        filtered.into_iter().map(INodeExecutionData::new).collect();
                                    new_branches.push(new_branch);
                                }
                                events = new_branches;
                            }

                            let has_items_after = events.iter().any(|arr| !arr.is_empty());
                            if has_items_after {
                                debug!(
                                    "Poller {} discovered {} new event branches",
                                    target_node.name,
                                    events.len()
                                );
                                callback(events);
                            }
                        }
                    }
                    Err(e) => {
                        error!("Polling task {} encountered error: {}", target_node.name, e);
                    }
                }
            }
        });

        let mut lock = self.active_pollers.write().await;
        // Key uniquely identifies this process instance running the poll loop
        let key = format!("{}_{}", workflow.id, node_id);

        if let Some(existing) = lock.insert(key.clone(), handle) {
            existing.abort();
            debug!("Aborted prior polling loop instance for {}", key);
        }

        Ok(())
    }

    /// Stops polling operations for a specific trigger
    pub async fn stop_poller(&self, workflow_id: &str, node_id: &str) {
        let key = format!("{}_{}", workflow_id, node_id);
        let mut lock = self.active_pollers.write().await;

        if let Some(handle) = lock.remove(&key) {
            handle.abort();
            info!("Stopped polling loop for {}", key);
        }
    }
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;
    use barqflow_core::traits::{IPollFunctions, INodeType};

    use super::*;
    use barqflow_core::schema::{INode, INodeParameters, WorkflowDef};
    use barqflow_core::types::{NodeId, WorkflowId};
    use barqflow_registry::node_properties::INodeProperties;
    use barqflow_registry::registry::NodeInfo;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use uuid::Uuid;

    // A mock node for tracking counts
    struct TestPollingTrigger;

    #[async_trait]
    impl INodeType for TestPollingTrigger {
        fn get_description(&self) -> IDataObject {
            IDataObject::from(json!({ "name": "testTrigger", "description": "" }))
        }

        async fn execute(
            &self,
            _context: &dyn barqflow_core::traits::IExecuteFunctions,
        ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
            Ok(vec![vec![]])
        }

        async fn poll(
            &self,
            context: &dyn IPollFunctions,
        ) -> Result<Vec<Vec<INodeExecutionData>>, BarqError> {
            let poll_data_res = context.get_poll_data().await;
            let mut count = 0;
            if let Ok(data) = poll_data_res {
                if let Some(v) = data.0.get("count") {
                    count = v.as_u64().unwrap_or(0);
                }
            }

            count += 1;
            let mut new_poll_data = IDataObject::default();
            new_poll_data.0.insert("count".to_string(), json!(count));
            context.set_poll_data(new_poll_data).await?;

            let output_item =
                INodeExecutionData::new(IDataObject::from(json!({ "trigger_count": count })));
            Ok(vec![vec![output_item]])
        }
    }

    #[tokio::test]
    async fn test_polling_engine() {
        let registry = Arc::new(NodeRegistry::new());
        let node_info = NodeInfo {
            name: "testTrigger".to_string(),
            display_name: "Test Trigger".to_string(),
            version: 1.0,
            description: "".to_string(),
            properties: INodeProperties {
                display_name: None,
                properties: vec![],
                required_values: None,
            },
            is_trigger: true,
            max_inputs: 0,
            node_impl: Arc::new(TestPollingTrigger),
        };
        registry.register_node(node_info).unwrap();

        let static_data = Arc::new(RwLock::new(Some(IDataObject::default())));
        let engine = PollingEngine::new(registry.clone(), static_data.clone());

        let node_id = NodeId("test-node-123".to_string());
        let workflow = WorkflowDef {
            id: WorkflowId(Uuid::new_v4()),
            name: "Poll WF".to_string(),
            nodes: vec![INode {
                id: node_id.clone(),
                name: "Trigger Node".to_string(),
                r#type: "testTrigger".to_string(),
                type_version: 1.0,
                position: [0.0, 0.0],
                parameters: INodeParameters(HashMap::new()),
                disabled: false,
            }],
            connections: HashMap::new(),
            settings: Default::default(),
            active: true,
        };

        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        engine
            .register_poller(workflow.clone(), node_id.clone(), 1, move |_events| {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            })
            .await
            .unwrap();

        // Wait for two ticks (first is immediate upon start, second is after 1 second)
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

        let total_hits = counter.load(Ordering::Relaxed);
        assert!(total_hits >= 2, "Poller should have fired at least 2 times");

        // Verify static memory updated correctly
        let data_lock = static_data.read().await;
        let data = data_lock.as_ref().unwrap();
        let node_memory = data
            .0
            .get(&node_id.to_string())
            .unwrap()
            .as_object()
            .unwrap();
        let count = node_memory.get("count").unwrap().as_u64().unwrap();
        assert!(
            count >= 2,
            "Static data should have been updated deeply across polls"
        );

        // Stop poller cleanly
        engine
            .stop_poller(&workflow.id.to_string(), &node_id.to_string())
            .await;
    }
}
