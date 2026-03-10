use crate::controllers::executions::{execute_queued_dispatch_item, AppState as ExecutionControllerState};
use crate::execution_events::with_execution_event_history;
use crate::operations::maybe_run_execution_pruning;
use crate::repositories::execution_dispatch::ExecutionQueueKind;
use axum::http::StatusCode;
use barqflow_db::models::{ExecutionEntity, WorkflowEntity};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time::sleep;
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueuedExecutionPayload {
    pub workflow_name: String,
    pub active: bool,
    pub nodes: serde_json::Value,
    pub connections: serde_json::Value,
    pub settings: serde_json::Value,
    pub manual: bool,
    pub stop_after_node_id: Option<String>,
    pub static_data: Option<serde_json::Value>,
}

pub async fn enqueue_workflow_execution(
    state: &ExecutionControllerState,
    workspace_id: Uuid,
    workflow: &WorkflowEntity,
    queue_kind: ExecutionQueueKind,
    source: &str,
    manual: bool,
    stop_after_node_id: Option<String>,
    static_data: Option<serde_json::Value>,
) -> Result<ExecutionEntity, (StatusCode, String)> {
    if let Err(error) = maybe_run_execution_pruning(
        &state.operations_runtime,
        &state.execution_repo,
        &state.execution_log_repo,
    )
    .await
    {
        warn!(error = %error, "Execution pruning run failed");
    }

    let queue_depth = state
        .execution_dispatch_repo
        .count_open_items()
        .await
        .map_err(internal_error)?;
    if queue_depth >= state.operations_runtime.queue_capacity() as i64 {
        state.operations_runtime.mark_dispatch_failed().await;
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!(
                "Execution queue is full (capacity: {})",
                state.operations_runtime.queue_capacity()
            ),
        ));
    }

    state.operations_runtime.register_dispatch().await;

    let new_exec = state
        .execution_repo
        .create(workflow.id, "queued", serde_json::Value::Null)
        .await
        .map_err(internal_error)?;

    let payload = QueuedExecutionPayload {
        workflow_name: workflow.name.clone(),
        active: workflow.active,
        nodes: workflow.nodes.clone(),
        connections: workflow.connections.clone(),
        settings: workflow.settings.clone(),
        manual,
        stop_after_node_id: stop_after_node_id.clone(),
        static_data,
    };

    let queue_item = state
        .execution_dispatch_repo
        .create(
            new_exec.id,
            workspace_id,
            workflow.id,
            queue_kind,
            source,
            queue_priority(queue_kind, manual),
            serde_json::to_value(payload).map_err(internal_error)?,
            Utc::now(),
        )
        .await
        .map_err(internal_error)?;

    let queued_event = crate::controllers::executions::build_execution_event(
        new_exec.id,
        workflow.id,
        barqflow_core::types::RunId(new_exec.id),
        1,
        barqflow_core::contracts::ExecutionEventType::Queued,
        barqflow_core::contracts::ExecutionStatus::Queued,
        "Execution queued",
        None,
        None,
        json!({
            "manual": manual,
            "workflowName": workflow.name.clone(),
            "stopAfterNodeId": stop_after_node_id,
            "queueItemId": queue_item.id,
            "queueKind": queue_kind.as_str(),
            "source": source,
        }),
    );

    crate::controllers::executions::append_execution_event(state, queued_event.clone()).await;

    let updated = state
        .execution_repo
        .update_status_and_data(
            new_exec.id,
            "queued",
            with_execution_event_history(
                json!({
                    "queued": true,
                    "manual": manual,
                    "workflowName": workflow.name.clone(),
                    "stopAfterNodeId": queued_event.data.0.get("stopAfterNodeId").cloned(),
                    "dispatch": {
                        "queueItemId": queue_item.id,
                        "queueKind": queue_kind.as_str(),
                        "source": source,
                        "queuedAt": queued_event.timestamp,
                    },
                }),
                &[queued_event],
            ),
        )
        .await
        .map_err(internal_error)?
        .ok_or_else(|| (StatusCode::NOT_FOUND, "Execution not found after queueing".to_string()))?;

    Ok(updated)
}

pub fn start_dispatch_workers(state: ExecutionControllerState) {
    if state.operations_runtime.dispatch_mode() == crate::operations::ExecutionDispatchMode::Inline {
        return;
    }

    let run_workers = state.operations_runtime.run_worker_concurrency();
    let trigger_workers = state.operations_runtime.trigger_worker_concurrency();

    for index in 0..run_workers {
        let worker_state = state.clone();
        let worker_id = format!("run-worker-{}", index + 1);
        tokio::spawn(async move {
            worker_loop(worker_state, ExecutionQueueKind::Run, worker_id).await;
        });
    }

    for index in 0..trigger_workers {
        let worker_state = state.clone();
        let worker_id = format!("trigger-worker-{}", index + 1);
        tokio::spawn(async move {
            worker_loop(worker_state, ExecutionQueueKind::Trigger, worker_id).await;
        });
    }

    info!(run_workers, trigger_workers, "Execution dispatch workers started");
}

async fn worker_loop(
    state: ExecutionControllerState,
    queue_kind: ExecutionQueueKind,
    worker_id: String,
) {
    let poll_interval = state.operations_runtime.worker_poll_interval();
    let lease_seconds = state.operations_runtime.worker_lease_seconds();
    let heartbeat_interval = state.operations_runtime.worker_lease_heartbeat_interval();

    loop {
        match state
            .execution_dispatch_repo
            .claim_next(queue_kind, &worker_id, lease_seconds)
            .await
        {
            Ok(Some(queue_item)) => {
                state.operations_runtime.mark_started().await;

                let heartbeat_repo = state.execution_dispatch_repo.clone();
                let queue_item_id = queue_item.id;
                let heartbeat = tokio::spawn(async move {
                    loop {
                        sleep(heartbeat_interval).await;
                        if heartbeat_repo
                            .renew_lease(queue_item_id, lease_seconds)
                            .await
                            .ok()
                            .flatten()
                            .is_none()
                        {
                            break;
                        }
                    }
                });

                let result = execute_queued_dispatch_item(state.clone(), queue_item.clone()).await;
                heartbeat.abort();

                match result {
                    Ok(()) => {
                        let _ = state.execution_dispatch_repo.mark_completed(queue_item.id).await;
                    }
                    Err((status, message)) => {
                        warn!(
                            queue_kind = queue_kind.as_str(),
                            execution_id = %queue_item.execution_id,
                            status_code = status.as_u16(),
                            error = %message,
                            "Execution dispatch worker failed"
                        );
                        let _ = state
                            .execution_dispatch_repo
                            .mark_failed(queue_item.id, &message)
                            .await;
                    }
                }

                state.operations_runtime.mark_finished().await;
            }
            Ok(None) => sleep(poll_interval).await,
            Err(error) => {
                warn!(
                    queue_kind = queue_kind.as_str(),
                    worker_id = %worker_id,
                    error = %error,
                    "Execution dispatch worker claim failed"
                );
                sleep(poll_interval).await;
            }
        }
    }
}

fn queue_priority(queue_kind: ExecutionQueueKind, manual: bool) -> i32 {
    match (queue_kind, manual) {
        (ExecutionQueueKind::Run, true) => 10,
        (ExecutionQueueKind::Run, false) => 20,
        (ExecutionQueueKind::Trigger, _) => 50,
    }
}

fn internal_error(error: impl ToString) -> (StatusCode, String) {
    (StatusCode::INTERNAL_SERVER_ERROR, error.to_string())
}
