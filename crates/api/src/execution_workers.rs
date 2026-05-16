use crate::controllers::executions::{
    execute_queued_dispatch_item, AppState as ExecutionControllerState,
};
use crate::repositories::execution_dispatch::ExecutionQueueKind;
use tokio::time::sleep;
use tracing::{info, warn};

/// Spawn background worker tasks that drain the execution dispatch queue.
///
/// No-ops in Inline dispatch mode (the HTTP handler executes synchronously).
/// In Queue mode, spawns one Tokio task per configured concurrency slot for
/// each queue kind (Run / Trigger), each running [`worker_loop`] forever.
pub fn start_dispatch_workers(state: ExecutionControllerState) {
    if state.operations_runtime.dispatch_mode() == crate::operations::ExecutionDispatchMode::Inline
    {
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
