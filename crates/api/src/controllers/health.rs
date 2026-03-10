use crate::active_workflows::ActiveCronJobs;
use crate::controllers::webhooks::{WebhookEndpoint, WebhookRegistry};
use crate::repositories::execution_dispatch::ExecutionDispatchRepository;
use crate::operations::{ExecutionDispatchMode, OperationsRuntime};
use crate::routes::ActiveExecutionManager;
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub webhook_registry: WebhookRegistry,
    pub active_cron_jobs: ActiveCronJobs,
    pub active_executions: ActiveExecutionManager,
    pub node_registry: std::sync::Arc<barqflow_registry::registry::NodeRegistry>,
    pub credential_registry: std::sync::Arc<barqflow_registry::registry::CredentialRegistry>,
    pub execution_dispatch_repo: std::sync::Arc<ExecutionDispatchRepository>,
    pub operations_runtime: OperationsRuntime,
}

#[derive(Debug, Serialize)]
pub struct TriggerHealthResponse {
    pub webhooks: WebhookHealth,
    pub cron: CronHealth,
    pub executions: ExecutionHealth,
    pub polling: PollingHealth,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
pub struct WebhookHealth {
    pub endpoint_count: usize,
    pub workflow_count: usize,
}

#[derive(Debug, Serialize)]
pub struct CronHealth {
    pub workflow_count: usize,
    pub job_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ExecutionHealth {
    pub running_count: usize,
}

#[derive(Debug, Serialize)]
pub struct PollingHealth {
    pub active_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHealthResponse {
    pub status: String,
    pub environment: String,
    pub server_time: DateTime<Utc>,
    pub dispatch_mode: String,
    pub active_executions: usize,
    pub queued_executions: usize,
    pub worker_concurrency: usize,
    pub queue_capacity: usize,
    pub webhook_endpoint_count: usize,
    pub cron_job_count: usize,
    pub node_types_count: usize,
    pub credential_types_count: usize,
    pub tracing_enabled: bool,
    pub trace_format: String,
    pub pruning_enabled: bool,
    pub retention_days: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeMetricsResponse {
    pub dispatch: RuntimeDispatchMetrics,
    pub execution_totals: RuntimeExecutionTotals,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDispatchMetrics {
    pub mode: String,
    pub worker_concurrency: usize,
    pub queue_capacity: usize,
    pub queued_count: usize,
    pub running_count: usize,
    pub total_enqueued: u64,
    pub total_started: u64,
    pub total_finished: u64,
    pub total_failed_to_dispatch: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeExecutionTotals {
    pub active_executions: usize,
    pub webhook_endpoint_count: usize,
    pub webhook_workflow_count: usize,
    pub cron_workflow_count: usize,
    pub cron_job_count: usize,
    pub node_types_count: usize,
    pub credential_types_count: usize,
}

pub fn health_routes(state: AppState) -> Router {
    Router::new()
        .route("/health/triggers", get(get_trigger_health))
        .route("/health/runtime", get(get_runtime_health))
        .route("/metrics/runtime", get(get_runtime_metrics))
        .with_state(state)
}

async fn get_trigger_health(
    State(state): State<AppState>,
) -> Result<Json<TriggerHealthResponse>, (StatusCode, String)> {
    let (webhook_endpoint_count, webhook_workflow_count) = {
        let registry = state.webhook_registry.read().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Registry lock poisoned".into(),
            )
        })?;
        summarize_webhooks(&registry)
    };

    let (cron_workflow_count, cron_job_count) = {
        let cron_jobs = state.active_cron_jobs.read().await;
        summarize_cron_jobs(&cron_jobs)
    };

    let execution_count = {
        let active_executions = state.active_executions.read().await;
        active_executions.len()
    };

    Ok(Json(TriggerHealthResponse {
        webhooks: WebhookHealth {
            endpoint_count: webhook_endpoint_count,
            workflow_count: webhook_workflow_count,
        },
        cron: CronHealth {
            workflow_count: cron_workflow_count,
            job_count: cron_job_count,
        },
        executions: ExecutionHealth {
            running_count: execution_count,
        },
        polling: PollingHealth { active_count: 0 },
        generated_at: Utc::now().to_rfc3339(),
    }))
}

async fn get_runtime_health(
    State(state): State<AppState>,
) -> Result<Json<RuntimeHealthResponse>, (StatusCode, String)> {
    let dispatch = state.operations_runtime.dispatch_metrics_snapshot().await;
    let queued_executions = state
        .execution_dispatch_repo
        .count_open_items()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .max(0) as usize;
    let pruning = state.operations_runtime.pruning_snapshot().await;
    let telemetry = state.operations_runtime.telemetry_snapshot();
    let active_executions = state.active_executions.read().await.len();
    let webhook_endpoint_count = {
        let registry = state.webhook_registry.read().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Registry lock poisoned".into(),
            )
        })?;
        registry.len()
    };
    let cron_job_count = {
        let cron_jobs = state.active_cron_jobs.read().await;
        cron_jobs.values().map(|job_ids| job_ids.len()).sum::<usize>()
    };

    Ok(Json(RuntimeHealthResponse {
        status: "ok".to_string(),
        environment: std::env::var("BARQFLOW_ENV").unwrap_or_else(|_| "development".to_string()),
        server_time: Utc::now(),
        dispatch_mode: dispatch_mode_label(dispatch.mode),
        active_executions,
        queued_executions,
        worker_concurrency: dispatch.worker_concurrency,
        queue_capacity: dispatch.queue_capacity,
        webhook_endpoint_count,
        cron_job_count,
        node_types_count: state.node_registry.get_all_node_names().len(),
        credential_types_count: state.credential_registry.get_all_credentials().len(),
        tracing_enabled: telemetry.enabled,
        trace_format: telemetry.format,
        pruning_enabled: pruning.enabled,
        retention_days: pruning.retention_days,
    }))
}

async fn get_runtime_metrics(
    State(state): State<AppState>,
) -> Result<Json<RuntimeMetricsResponse>, (StatusCode, String)> {
    let dispatch = state.operations_runtime.dispatch_metrics_snapshot().await;
    let queued_count = state
        .execution_dispatch_repo
        .count_open_items()
        .await
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()))?
        .max(0) as usize;
    let (webhook_endpoint_count, webhook_workflow_count) = {
        let registry = state.webhook_registry.read().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Registry lock poisoned".into(),
            )
        })?;
        summarize_webhooks(&registry)
    };
    let (cron_workflow_count, cron_job_count) = {
        let cron_jobs = state.active_cron_jobs.read().await;
        summarize_cron_jobs(&cron_jobs)
    };
    let active_executions = state.active_executions.read().await.len();

    Ok(Json(RuntimeMetricsResponse {
        dispatch: RuntimeDispatchMetrics {
            mode: dispatch_mode_label(dispatch.mode),
            worker_concurrency: dispatch.worker_concurrency,
            queue_capacity: dispatch.queue_capacity,
            queued_count,
            running_count: dispatch.running_count,
            total_enqueued: dispatch.total_enqueued,
            total_started: dispatch.total_started,
            total_finished: dispatch.total_finished,
            total_failed_to_dispatch: dispatch.total_failed_to_dispatch,
        },
        execution_totals: RuntimeExecutionTotals {
            active_executions,
            webhook_endpoint_count,
            webhook_workflow_count,
            cron_workflow_count,
            cron_job_count,
            node_types_count: state.node_registry.get_all_node_names().len(),
            credential_types_count: state.credential_registry.get_all_credentials().len(),
        },
        generated_at: Utc::now(),
    }))
}

fn dispatch_mode_label(mode: ExecutionDispatchMode) -> String {
    match mode {
        ExecutionDispatchMode::Inline => "inline".to_string(),
        ExecutionDispatchMode::Queue => "queue".to_string(),
    }
}

fn summarize_webhooks(registry: &HashMap<String, WebhookEndpoint>) -> (usize, usize) {
    let mut workflow_ids = HashSet::new();
    for endpoint in registry.values() {
        workflow_ids.insert(endpoint.workflow_id);
    }
    (registry.len(), workflow_ids.len())
}

fn summarize_cron_jobs(active_cron_jobs: &HashMap<Uuid, Vec<Uuid>>) -> (usize, usize) {
    let workflow_count = active_cron_jobs.len();
    let job_count = active_cron_jobs
        .values()
        .map(|job_ids| job_ids.len())
        .sum::<usize>();
    (workflow_count, job_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::routes::ActiveExecutionControl;
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use tokio_util::sync::CancellationToken;
    use tower::ServiceExt;

    fn test_state(
        webhook_registry: WebhookRegistry,
        active_cron_jobs: ActiveCronJobs,
        active_executions: ActiveExecutionManager,
    ) -> AppState {
        let node_registry = std::sync::Arc::new(barqflow_registry::registry::NodeRegistry::new());
        let credential_registry =
            std::sync::Arc::new(barqflow_registry::registry::CredentialRegistry::new());
        let dispatch_pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://barqflow:barqflow@localhost/barqflow")
            .expect("lazy dispatch pool");

        AppState {
            webhook_registry,
            active_cron_jobs,
            active_executions,
            node_registry,
            credential_registry,
            execution_dispatch_repo: std::sync::Arc::new(ExecutionDispatchRepository::new(
                dispatch_pool,
            )),
            operations_runtime: OperationsRuntime::from_env(),
        }
    }

    #[test]
    fn summarize_webhooks_counts_unique_workflows() {
        let workflow_a = Uuid::new_v4();
        let workflow_b = Uuid::new_v4();

        let mut registry = HashMap::new();
        registry.insert(
            "hook-a".to_string(),
            WebhookEndpoint {
                workflow_id: workflow_a,
                node_id: "node-a".to_string(),
                http_method: "POST".to_string(),
            },
        );
        registry.insert(
            "hook-b".to_string(),
            WebhookEndpoint {
                workflow_id: workflow_a,
                node_id: "node-b".to_string(),
                http_method: "POST".to_string(),
            },
        );
        registry.insert(
            "hook-c".to_string(),
            WebhookEndpoint {
                workflow_id: workflow_b,
                node_id: "node-c".to_string(),
                http_method: "GET".to_string(),
            },
        );

        let (endpoint_count, workflow_count) = summarize_webhooks(&registry);
        assert_eq!(endpoint_count, 3);
        assert_eq!(workflow_count, 2);
    }

    #[test]
    fn summarize_cron_jobs_counts_jobs_across_workflows() {
        let mut active = HashMap::new();
        active.insert(Uuid::new_v4(), vec![Uuid::new_v4(), Uuid::new_v4()]);
        active.insert(Uuid::new_v4(), vec![Uuid::new_v4()]);

        let (workflow_count, job_count) = summarize_cron_jobs(&active);
        assert_eq!(workflow_count, 2);
        assert_eq!(job_count, 3);
    }

    #[tokio::test]
    async fn trigger_health_endpoint_reports_runtime_counts() {
        let workflow_id = Uuid::new_v4();
        let mut webhook_map = HashMap::new();
        webhook_map.insert(
            "hook-a".to_string(),
            WebhookEndpoint {
                workflow_id,
                node_id: "node-a".to_string(),
                http_method: "POST".to_string(),
            },
        );

        let webhook_registry = std::sync::Arc::new(std::sync::RwLock::new(webhook_map));
        let active_cron_jobs = std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::from([(
            workflow_id,
            vec![Uuid::new_v4()],
        )])));

        let noop_task = tokio::spawn(async {});
        let execution_id = Uuid::new_v4();
        let active_executions = std::sync::Arc::new(tokio::sync::RwLock::new(HashMap::from([(
            execution_id,
            ActiveExecutionControl {
                cancellation_token: CancellationToken::new(),
                abort_handle: noop_task.abort_handle(),
            },
        )])));

        let app = health_routes(test_state(
            webhook_registry,
            active_cron_jobs,
            active_executions,
        ));

        let request = Request::builder()
            .uri("/health/triggers")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let payload: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(payload["webhooks"]["endpoint_count"], 1);
        assert_eq!(payload["webhooks"]["workflow_count"], 1);
        assert_eq!(payload["cron"]["workflow_count"], 1);
        assert_eq!(payload["cron"]["job_count"], 1);
        assert_eq!(payload["executions"]["running_count"], 1);
        assert_eq!(payload["polling"]["active_count"], 0);
        assert!(payload["generated_at"].as_str().is_some());
    }
}
