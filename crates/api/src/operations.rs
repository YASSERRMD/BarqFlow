use chrono::{DateTime, Duration, Utc};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration as TokioDuration;

use crate::repositories::{execution::ExecutionRepository, execution_log::ExecutionLogRepository};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ExecutionDispatchMode {
    Inline,
    Queue,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionDispatchMetricsSnapshot {
    pub mode: ExecutionDispatchMode,
    pub worker_concurrency: usize,
    pub queue_capacity: usize,
    pub queued_count: usize,
    pub running_count: usize,
    pub total_enqueued: u64,
    pub total_started: u64,
    pub total_finished: u64,
    pub total_failed_to_dispatch: u64,
    pub last_enqueued_at: Option<DateTime<Utc>>,
    pub last_started_at: Option<DateTime<Utc>>,
    pub last_finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionPruningSnapshot {
    pub enabled: bool,
    pub retention_days: u64,
    pub interval_minutes: u64,
    pub last_run_at: Option<DateTime<Utc>>,
    pub last_cutoff_at: Option<DateTime<Utc>>,
    pub last_executions_deleted: u64,
    pub last_wait_resumes_deleted: u64,
    pub last_logs_deleted: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelemetrySnapshot {
    pub enabled: bool,
    pub format: String,
    pub service_name: String,
    pub environment: String,
    pub request_id_header: String,
}

#[derive(Debug, Clone)]
pub struct PruneExecutionsSummary {
    pub cutoff: DateTime<Utc>,
    pub ran_at: DateTime<Utc>,
    pub executions_deleted: u64,
    pub wait_resumes_deleted: u64,
    pub logs_deleted: u64,
}

#[derive(Debug, Clone)]
pub struct ExecutionPruningConfig {
    pub enabled: bool,
    pub retention_days: u64,
    pub interval_minutes: u64,
}

#[derive(Debug, Clone)]
struct ExecutionDispatchMetrics {
    queued_count: usize,
    running_count: usize,
    total_enqueued: u64,
    total_started: u64,
    total_finished: u64,
    total_failed_to_dispatch: u64,
    last_enqueued_at: Option<DateTime<Utc>>,
    last_started_at: Option<DateTime<Utc>>,
    last_finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
struct ExecutionPruningState {
    last_run_at: Option<DateTime<Utc>>,
    last_cutoff_at: Option<DateTime<Utc>>,
    last_executions_deleted: u64,
    last_wait_resumes_deleted: u64,
    last_logs_deleted: u64,
}

#[derive(Clone)]
pub struct OperationsRuntime {
    dispatch_mode: ExecutionDispatchMode,
    worker_concurrency: usize,
    run_worker_concurrency: usize,
    trigger_worker_concurrency: usize,
    queue_capacity: usize,
    worker_poll_interval_ms: u64,
    worker_lease_seconds: i64,
    dispatch_metrics: Arc<RwLock<ExecutionDispatchMetrics>>,
    pruning_config: ExecutionPruningConfig,
    pruning_state: Arc<RwLock<ExecutionPruningState>>,
    telemetry: TelemetrySnapshot,
}

impl OperationsRuntime {
    pub fn from_env() -> Self {
        let dispatch_mode = match std::env::var("BARQFLOW_EXECUTION_MODE") {
            Ok(value) if value.eq_ignore_ascii_case("inline") => ExecutionDispatchMode::Inline,
            _ => ExecutionDispatchMode::Queue,
        };

        let run_worker_concurrency =
            parse_usize_env("BARQFLOW_EXECUTION_RUN_WORKER_CONCURRENCY", 4).max(1);
        let trigger_worker_concurrency =
            parse_usize_env("BARQFLOW_EXECUTION_TRIGGER_WORKER_CONCURRENCY", 2).max(1);
        let worker_concurrency = run_worker_concurrency + trigger_worker_concurrency;
        let queue_capacity = parse_usize_env("BARQFLOW_EXECUTION_QUEUE_CAPACITY", 128).max(1);
        let worker_poll_interval_ms =
            parse_u64_env("BARQFLOW_EXECUTION_POLL_INTERVAL_MS", 750).max(100);
        let worker_lease_seconds =
            parse_i64_env("BARQFLOW_EXECUTION_WORKER_LEASE_SECONDS", 300).max(30);
        let pruning_config = ExecutionPruningConfig {
            enabled: parse_bool_env("BARQFLOW_EXECUTION_PRUNING_ENABLED", true),
            retention_days: parse_u64_env("BARQFLOW_EXECUTION_RETENTION_DAYS", 30).max(1),
            interval_minutes: parse_u64_env("BARQFLOW_EXECUTION_PRUNE_INTERVAL_MINUTES", 60).max(1),
        };

        let telemetry = TelemetrySnapshot {
            enabled: parse_bool_env("BARQFLOW_TRACING_ENABLED", true),
            format: std::env::var("BARQFLOW_TRACE_FORMAT").unwrap_or_else(|_| "pretty".to_string()),
            service_name: std::env::var("BARQFLOW_SERVICE_NAME")
                .unwrap_or_else(|_| "barqflow-server".to_string()),
            environment: std::env::var("BARQFLOW_ENV")
                .unwrap_or_else(|_| "development".to_string()),
            request_id_header: "x-request-id".to_string(),
        };

        Self {
            dispatch_mode,
            worker_concurrency,
            run_worker_concurrency,
            trigger_worker_concurrency,
            queue_capacity,
            worker_poll_interval_ms,
            worker_lease_seconds,
            dispatch_metrics: Arc::new(RwLock::new(ExecutionDispatchMetrics {
                queued_count: 0,
                running_count: 0,
                total_enqueued: 0,
                total_started: 0,
                total_finished: 0,
                total_failed_to_dispatch: 0,
                last_enqueued_at: None,
                last_started_at: None,
                last_finished_at: None,
            })),
            pruning_config,
            pruning_state: Arc::new(RwLock::new(ExecutionPruningState::default())),
            telemetry,
        }
    }

    pub fn dispatch_mode(&self) -> ExecutionDispatchMode {
        self.dispatch_mode
    }

    pub fn worker_concurrency(&self) -> usize {
        self.worker_concurrency
    }

    pub fn run_worker_concurrency(&self) -> usize {
        self.run_worker_concurrency
    }

    pub fn trigger_worker_concurrency(&self) -> usize {
        self.trigger_worker_concurrency
    }

    pub fn queue_capacity(&self) -> usize {
        self.queue_capacity
    }

    pub fn worker_poll_interval(&self) -> TokioDuration {
        TokioDuration::from_millis(self.worker_poll_interval_ms)
    }

    pub fn worker_lease_seconds(&self) -> i64 {
        self.worker_lease_seconds
    }

    pub fn worker_lease_heartbeat_interval(&self) -> TokioDuration {
        TokioDuration::from_secs((self.worker_lease_seconds.max(30) / 3).max(10) as u64)
    }

    pub fn telemetry_snapshot(&self) -> TelemetrySnapshot {
        self.telemetry.clone()
    }

    pub async fn register_dispatch(&self) {
        let mut metrics = self.dispatch_metrics.write().await;
        metrics.total_enqueued += 1;
        metrics.last_enqueued_at = Some(Utc::now());
        if self.dispatch_mode == ExecutionDispatchMode::Queue {
            metrics.queued_count += 1;
        }
    }

    pub async fn mark_started(&self) {
        let mut metrics = self.dispatch_metrics.write().await;
        if self.dispatch_mode == ExecutionDispatchMode::Queue && metrics.queued_count > 0 {
            metrics.queued_count -= 1;
        }
        metrics.running_count += 1;
        metrics.total_started += 1;
        metrics.last_started_at = Some(Utc::now());
    }

    pub async fn mark_dispatch_failed(&self) {
        let mut metrics = self.dispatch_metrics.write().await;
        if self.dispatch_mode == ExecutionDispatchMode::Queue && metrics.queued_count > 0 {
            metrics.queued_count -= 1;
        }
        metrics.total_failed_to_dispatch += 1;
        metrics.last_finished_at = Some(Utc::now());
    }

    pub async fn mark_finished(&self) {
        let mut metrics = self.dispatch_metrics.write().await;
        if metrics.running_count > 0 {
            metrics.running_count -= 1;
        }
        metrics.total_finished += 1;
        metrics.last_finished_at = Some(Utc::now());
    }

    pub async fn dispatch_metrics_snapshot(&self) -> ExecutionDispatchMetricsSnapshot {
        let metrics = self.dispatch_metrics.read().await;
        ExecutionDispatchMetricsSnapshot {
            mode: self.dispatch_mode,
            worker_concurrency: self.worker_concurrency,
            queue_capacity: self.queue_capacity,
            queued_count: metrics.queued_count,
            running_count: metrics.running_count,
            total_enqueued: metrics.total_enqueued,
            total_started: metrics.total_started,
            total_finished: metrics.total_finished,
            total_failed_to_dispatch: metrics.total_failed_to_dispatch,
            last_enqueued_at: metrics.last_enqueued_at,
            last_started_at: metrics.last_started_at,
            last_finished_at: metrics.last_finished_at,
        }
    }

    pub fn pruning_cutoff(&self) -> Option<DateTime<Utc>> {
        if !self.pruning_config.enabled {
            return None;
        }

        Some(Utc::now() - Duration::days(self.pruning_config.retention_days as i64))
    }

    pub async fn should_run_pruning(&self) -> bool {
        if !self.pruning_config.enabled {
            return false;
        }

        let state = self.pruning_state.read().await;
        match state.last_run_at {
            Some(last_run) => {
                let elapsed = Utc::now() - last_run;
                elapsed >= Duration::minutes(self.pruning_config.interval_minutes as i64)
            }
            None => true,
        }
    }

    pub async fn record_pruning_run(&self, summary: &PruneExecutionsSummary) {
        let mut state = self.pruning_state.write().await;
        state.last_run_at = Some(summary.ran_at);
        state.last_cutoff_at = Some(summary.cutoff);
        state.last_executions_deleted = summary.executions_deleted;
        state.last_wait_resumes_deleted = summary.wait_resumes_deleted;
        state.last_logs_deleted = summary.logs_deleted;
    }

    pub async fn pruning_snapshot(&self) -> ExecutionPruningSnapshot {
        let state = self.pruning_state.read().await;
        ExecutionPruningSnapshot {
            enabled: self.pruning_config.enabled,
            retention_days: self.pruning_config.retention_days,
            interval_minutes: self.pruning_config.interval_minutes,
            last_run_at: state.last_run_at,
            last_cutoff_at: state.last_cutoff_at,
            last_executions_deleted: state.last_executions_deleted,
            last_wait_resumes_deleted: state.last_wait_resumes_deleted,
            last_logs_deleted: state.last_logs_deleted,
        }
    }
}

pub async fn maybe_run_execution_pruning(
    runtime: &OperationsRuntime,
    execution_repo: &ExecutionRepository,
    execution_log_repo: &ExecutionLogRepository,
) -> sqlx::Result<Option<PruneExecutionsSummary>> {
    if !runtime.should_run_pruning().await {
        return Ok(None);
    }

    let summary = run_execution_pruning(runtime, execution_repo, execution_log_repo).await?;
    Ok(Some(summary))
}

pub async fn run_execution_pruning(
    runtime: &OperationsRuntime,
    execution_repo: &ExecutionRepository,
    execution_log_repo: &ExecutionLogRepository,
) -> sqlx::Result<PruneExecutionsSummary> {
    let Some(cutoff) = runtime.pruning_cutoff() else {
        return Ok(PruneExecutionsSummary {
            cutoff: Utc::now(),
            ran_at: Utc::now(),
            executions_deleted: 0,
            wait_resumes_deleted: 0,
            logs_deleted: 0,
        });
    };

    let logs_deleted = execution_log_repo
        .count_prunable_for_executions_before(cutoff)
        .await?;
    let wait_resumes_deleted = execution_repo.delete_prunable_wait_resumes(cutoff).await?;
    let executions_deleted = execution_repo.delete_prunable_before(cutoff).await?;
    let summary = PruneExecutionsSummary {
        cutoff,
        ran_at: Utc::now(),
        executions_deleted,
        wait_resumes_deleted,
        logs_deleted,
    };
    runtime.record_pruning_run(&summary).await;
    Ok(summary)
}

fn parse_bool_env(key: &str, default: bool) -> bool {
    std::env::var(key)
        .ok()
        .and_then(|value| match value.trim().to_ascii_lowercase().as_str() {
            "1" | "true" | "yes" | "on" => Some(true),
            "0" | "false" | "no" | "off" => Some(false),
            _ => None,
        })
        .unwrap_or(default)
}

fn parse_usize_env(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(default)
}

fn parse_u64_env(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(default)
}

fn parse_i64_env(key: &str, default: i64) -> i64 {
    std::env::var(key)
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn queue_dispatch_rejects_when_capacity_is_exhausted() {
        std::env::set_var("BARQFLOW_EXECUTION_MODE", "queue");
        let runtime = OperationsRuntime::from_env();

        runtime.register_dispatch().await;
        runtime.register_dispatch().await;

        let metrics = runtime.dispatch_metrics_snapshot().await;
        assert_eq!(metrics.total_enqueued, 2);
        assert_eq!(metrics.queued_count, 2);

        std::env::remove_var("BARQFLOW_EXECUTION_MODE");
    }
}
