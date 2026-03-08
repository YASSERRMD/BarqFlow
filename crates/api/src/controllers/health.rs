use crate::active_workflows::ActiveCronJobs;
use crate::controllers::webhooks::{WebhookEndpoint, WebhookRegistry};
use crate::routes::ActiveExecutionManager;
use axum::{
    extract::State,
    http::StatusCode,
    routing::get,
    Json, Router,
};
use chrono::Utc;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub webhook_registry: WebhookRegistry,
    pub active_cron_jobs: ActiveCronJobs,
    pub active_executions: ActiveExecutionManager,
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

pub fn health_routes(state: AppState) -> Router {
    Router::new()
        .route("/health/triggers", get(get_trigger_health))
        .with_state(state)
}

async fn get_trigger_health(
    State(state): State<AppState>,
) -> Result<Json<TriggerHealthResponse>, (StatusCode, String)> {
    let (webhook_endpoint_count, webhook_workflow_count) = {
        let registry = state
            .webhook_registry
            .read()
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Registry lock poisoned".into()))?;
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
        // Polling runtime manager is not wired yet; report explicit zero until integrated.
        polling: PollingHealth { active_count: 0 },
        generated_at: Utc::now().to_rfc3339(),
    }))
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
}
