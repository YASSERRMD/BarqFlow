use crate::contracts::{
    CredentialHealthResponse, ExecutionFlamegraphResponse, ExecutionFlamegraphSpanResponse,
    FailureClusterResponse, LatencyBucketResponse, NodeLatencyHistogramResponse,
    ObservabilityOverviewResponse, WorkflowBottleneckResponse,
};
use crate::repositories::{
    credential::CredentialRepository, execution::ExecutionRepository,
    execution_log::ExecutionLogRepository, workflow::WorkflowRepository,
};
use barqflow_db::models::{CredentialEntity, ExecutionEntity, ExecutionLogEntity, WorkflowEntity};
use chrono::{DateTime, Duration, Utc};
use serde_json::Value;
use sqlx::Result;
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use uuid::Uuid;

const DEFAULT_WINDOW_HOURS: u32 = 72;
const MAX_WINDOW_HOURS: u32 = 168;
const MAX_EXECUTIONS: usize = 400;
const MAX_LOGS: usize = 6000;
const MAX_NODE_HISTOGRAMS: usize = 12;
const MAX_FAILURE_CLUSTERS: usize = 10;
const MAX_FLAMEGRAPH_SAMPLES: usize = 3;
const LATENCY_BUCKETS: &[(u64, &str)] = &[
    (100, "<100ms"),
    (500, "100-500ms"),
    (1_000, "0.5-1s"),
    (5_000, "1-5s"),
    (15_000, "5-15s"),
    (u64::MAX, "15s+"),
];

#[derive(Clone)]
struct WorkflowContext {
    workflow_name: String,
    nodes_by_id: HashMap<String, WorkflowNodeContext>,
    nodes_by_name: HashMap<String, WorkflowNodeContext>,
}

#[derive(Clone)]
struct WorkflowNodeContext {
    node_name: String,
    node_type: String,
}

struct PendingNodeStart {
    started_at: DateTime<Utc>,
    input_items: usize,
}

#[derive(Clone)]
struct CompletedNodeSpan {
    node_name: String,
    node_type: String,
    started_at: DateTime<Utc>,
    finished_at: DateTime<Utc>,
    duration_ms: u64,
    status: String,
    input_items: usize,
    output_items: usize,
}

#[derive(Clone)]
struct NodeAggregate {
    workflow_id: Uuid,
    workflow_name: String,
    node_name: String,
    node_type: String,
    durations_ms: Vec<u64>,
    failed_runs: usize,
}

struct FailureClusterAccumulator {
    cluster_key: String,
    workflow_id: Option<Uuid>,
    workflow_name: Option<String>,
    node_name: Option<String>,
    node_type: Option<String>,
    level: String,
    event_type: Option<String>,
    message: String,
    failure_count: usize,
    execution_ids: HashSet<Uuid>,
    last_seen_at: DateTime<Utc>,
}

pub fn clamp_observability_window(hours: Option<u32>) -> u32 {
    hours
        .unwrap_or(DEFAULT_WINDOW_HOURS)
        .clamp(1, MAX_WINDOW_HOURS)
}

pub async fn build_observability_overview(
    workflow_repo: &WorkflowRepository,
    execution_repo: &ExecutionRepository,
    execution_log_repo: &ExecutionLogRepository,
    credential_repo: &CredentialRepository,
    workspace_id: Uuid,
    window_hours: u32,
) -> Result<ObservabilityOverviewResponse> {
    let generated_at = Utc::now();
    let workflows = workflow_repo.find_all_for_workspace(workspace_id).await?;
    let credentials = credential_repo.find_all_in_workspace(workspace_id).await?;
    let workflow_ids: Vec<Uuid> = workflows.iter().map(|workflow| workflow.id).collect();
    let since = generated_at - Duration::hours(window_hours as i64);

    let executions = execution_repo
        .find_recent_for_workflow_ids(&workflow_ids, since, MAX_EXECUTIONS)
        .await?;
    let execution_id_set: HashSet<Uuid> = executions.iter().map(|execution| execution.id).collect();

    let logs = execution_log_repo
        .list_recent_for_workflow_ids(&workflow_ids, since, MAX_LOGS)
        .await?
        .into_iter()
        .filter(|log| execution_id_set.contains(&log.execution_id))
        .collect::<Vec<_>>();

    Ok(assemble_observability_overview(
        &workflows,
        &executions,
        &logs,
        &credentials,
        workspace_id,
        window_hours,
        generated_at,
    ))
}

fn assemble_observability_overview(
    workflows: &[WorkflowEntity],
    executions: &[ExecutionEntity],
    logs: &[ExecutionLogEntity],
    credentials: &[CredentialEntity],
    workspace_id: Uuid,
    window_hours: u32,
    generated_at: DateTime<Utc>,
) -> ObservabilityOverviewResponse {
    let workflow_contexts = build_workflow_contexts(workflows);

    let mut sorted_logs = logs.to_vec();
    sorted_logs.sort_by_key(|log| log.created_at);

    let mut pending_starts: HashMap<(Uuid, String), VecDeque<PendingNodeStart>> = HashMap::new();
    let mut spans_by_execution: HashMap<Uuid, Vec<CompletedNodeSpan>> = HashMap::new();
    let mut node_aggregates: HashMap<(Uuid, String), NodeAggregate> = HashMap::new();
    let mut failure_clusters: HashMap<String, FailureClusterAccumulator> = HashMap::new();

    for log in &sorted_logs {
        let workflow_context = workflow_contexts.get(&log.workflow_id);
        let resolved_node = resolve_node_context(
            workflow_context,
            log.node_id.as_deref(),
            log.node_name.as_deref(),
        );
        let node_key = resolve_node_key(log.node_id.as_deref(), log.node_name.as_deref());

        match log.event_type.as_deref() {
            Some("nodeStarted") => {
                let start = PendingNodeStart {
                    started_at: log.created_at,
                    input_items: extract_usize(log.payload.get("inputItems")),
                };
                pending_starts
                    .entry((log.execution_id, node_key))
                    .or_default()
                    .push_back(start);
            }
            Some("nodeFinished") => {
                let success = log
                    .payload
                    .get("success")
                    .and_then(Value::as_bool)
                    .unwrap_or(!log.level.eq_ignore_ascii_case("error"));
                let skipped = log
                    .payload
                    .get("skipped")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                let status = if skipped {
                    "skipped"
                } else if success {
                    "success"
                } else {
                    "failed"
                };

                if let Some(queue) = pending_starts.get_mut(&(log.execution_id, node_key.clone())) {
                    if let Some(start) = queue.pop_front() {
                        let duration_ms = (log.created_at - start.started_at)
                            .num_milliseconds()
                            .max(0) as u64;
                        let output_items = extract_usize(log.payload.get("outputItems"));
                        let span = CompletedNodeSpan {
                            node_name: resolved_node.node_name.clone(),
                            node_type: resolved_node.node_type.clone(),
                            started_at: start.started_at,
                            finished_at: log.created_at,
                            duration_ms,
                            status: status.to_string(),
                            input_items: start.input_items,
                            output_items,
                        };

                        spans_by_execution
                            .entry(log.execution_id)
                            .or_default()
                            .push(span.clone());

                        let aggregate_key = (log.workflow_id, resolved_node.node_name.clone());
                        let aggregate =
                            node_aggregates
                                .entry(aggregate_key)
                                .or_insert_with(|| NodeAggregate {
                                    workflow_id: log.workflow_id,
                                    workflow_name: workflow_context
                                        .map(|context| context.workflow_name.clone())
                                        .unwrap_or_else(|| "Unknown workflow".to_string()),
                                    node_name: resolved_node.node_name.clone(),
                                    node_type: resolved_node.node_type.clone(),
                                    durations_ms: Vec::new(),
                                    failed_runs: 0,
                                });
                        aggregate.durations_ms.push(duration_ms);
                        if status == "failed" {
                            aggregate.failed_runs += 1;
                        }
                    }
                }

                if status == "failed" {
                    record_failure_cluster(
                        &mut failure_clusters,
                        log,
                        workflow_context,
                        &resolved_node,
                        Some(
                            extract_error_message(&log.payload)
                                .unwrap_or_else(|| log.message.clone()),
                        ),
                    );
                }
            }
            Some("failed") => {
                record_failure_cluster(
                    &mut failure_clusters,
                    log,
                    workflow_context,
                    &resolved_node,
                    None,
                );
            }
            _ => {}
        }
    }

    let mut execution_durations_by_workflow: HashMap<Uuid, Vec<u64>> = HashMap::new();
    let mut successful_execution_count = 0usize;
    let mut failed_execution_count = 0usize;
    let mut stopped_execution_count = 0usize;
    let mut queued_execution_count = 0usize;
    let mut running_execution_count = 0usize;
    let mut waiting_execution_count = 0usize;

    for execution in executions {
        match execution.status.to_ascii_lowercase().as_str() {
            "success" => successful_execution_count += 1,
            "failed" | "error" | "crashed" => failed_execution_count += 1,
            "stopped" | "cancelled" => stopped_execution_count += 1,
            "queued" => queued_execution_count += 1,
            "running" => running_execution_count += 1,
            "waiting" => waiting_execution_count += 1,
            _ => {}
        }

        if let Some(duration_ms) = execution_duration_ms(execution) {
            execution_durations_by_workflow
                .entry(execution.workflow_id)
                .or_default()
                .push(duration_ms);
        }
    }

    let terminal_execution_count =
        successful_execution_count + failed_execution_count + stopped_execution_count;
    let average_execution_duration_ms = average_u64(
        &execution_durations_by_workflow
            .values()
            .flat_map(|durations| durations.iter().copied())
            .collect::<Vec<_>>(),
    );

    let mut node_latency_histograms = node_aggregates
        .values()
        .map(|aggregate| NodeLatencyHistogramResponse {
            workflow_id: aggregate.workflow_id,
            workflow_name: aggregate.workflow_name.clone(),
            node_name: aggregate.node_name.clone(),
            node_type: aggregate.node_type.clone(),
            samples: aggregate.durations_ms.len(),
            failed_runs: aggregate.failed_runs,
            avg_duration_ms: average_u64(&aggregate.durations_ms),
            p95_duration_ms: percentile_u64(&aggregate.durations_ms, 95.0),
            max_duration_ms: aggregate
                .durations_ms
                .iter()
                .copied()
                .max()
                .unwrap_or_default(),
            histogram: build_latency_histogram(&aggregate.durations_ms),
        })
        .collect::<Vec<_>>();
    node_latency_histograms.sort_by(compare_latency_histograms);
    node_latency_histograms.truncate(MAX_NODE_HISTOGRAMS);

    let mut bottlenecks_by_workflow: HashMap<Uuid, WorkflowBottleneckResponse> = HashMap::new();
    for aggregate in node_aggregates.values() {
        let avg_duration_ms = average_u64(&aggregate.durations_ms);
        let workflow_avg = execution_durations_by_workflow
            .get(&aggregate.workflow_id)
            .map(|durations| average_u64(durations))
            .unwrap_or_default();
        let bottleneck = WorkflowBottleneckResponse {
            workflow_id: aggregate.workflow_id,
            workflow_name: aggregate.workflow_name.clone(),
            node_name: aggregate.node_name.clone(),
            node_type: aggregate.node_type.clone(),
            samples: aggregate.durations_ms.len(),
            failure_count: aggregate.failed_runs,
            avg_duration_ms,
            p95_duration_ms: percentile_u64(&aggregate.durations_ms, 95.0),
            contribution_rate: percentage(avg_duration_ms, workflow_avg.max(1)),
        };

        let should_replace = bottlenecks_by_workflow
            .get(&aggregate.workflow_id)
            .map(|existing| compare_bottlenecks(&bottleneck, existing) == Ordering::Less)
            .unwrap_or(true);
        if should_replace {
            bottlenecks_by_workflow.insert(aggregate.workflow_id, bottleneck);
        }
    }

    let mut workflow_bottlenecks = bottlenecks_by_workflow.into_values().collect::<Vec<_>>();
    workflow_bottlenecks.sort_by(compare_bottlenecks);

    let mut failure_clusters = failure_clusters
        .into_values()
        .map(|cluster| FailureClusterResponse {
            cluster_key: cluster.cluster_key,
            workflow_id: cluster.workflow_id,
            workflow_name: cluster.workflow_name,
            node_name: cluster.node_name,
            node_type: cluster.node_type,
            level: cluster.level,
            event_type: cluster.event_type,
            message: cluster.message,
            failure_count: cluster.failure_count,
            affected_execution_count: cluster.execution_ids.len(),
            last_seen_at: cluster.last_seen_at,
        })
        .collect::<Vec<_>>();
    failure_clusters.sort_by(|left, right| {
        right
            .failure_count
            .cmp(&left.failure_count)
            .then_with(|| right.last_seen_at.cmp(&left.last_seen_at))
    });
    failure_clusters.truncate(MAX_FAILURE_CLUSTERS);

    let mut credential_health = credentials
        .iter()
        .map(score_credential_health)
        .collect::<Vec<_>>();
    credential_health.sort_by(compare_credentials);

    let mut execution_flamegraphs = executions
        .iter()
        .filter_map(|execution| {
            let mut spans = spans_by_execution.get(&execution.id)?.clone();
            spans.sort_by_key(|span| span.started_at);
            let workflow_name = workflow_contexts
                .get(&execution.workflow_id)
                .map(|context| context.workflow_name.clone())
                .unwrap_or_else(|| "Unknown workflow".to_string());
            let total_duration_ms = execution_total_duration_ms(execution, &spans);
            let flamegraph_spans = spans
                .into_iter()
                .map(|span| ExecutionFlamegraphSpanResponse {
                    node_name: span.node_name,
                    node_type: span.node_type,
                    offset_ms: (span.started_at - execution.started_at)
                        .num_milliseconds()
                        .max(0) as u64,
                    duration_ms: span.duration_ms,
                    status: span.status,
                    started_at: span.started_at,
                    finished_at: span.finished_at,
                    input_items: span.input_items,
                    output_items: span.output_items,
                })
                .collect::<Vec<_>>();

            Some(ExecutionFlamegraphResponse {
                execution_id: execution.id,
                workflow_id: execution.workflow_id,
                workflow_name,
                status: execution.status.clone(),
                started_at: execution.started_at,
                stopped_at: execution.stopped_at,
                total_duration_ms,
                spans: flamegraph_spans,
            })
        })
        .collect::<Vec<_>>();
    execution_flamegraphs.sort_by(compare_flamegraphs);
    execution_flamegraphs.truncate(MAX_FLAMEGRAPH_SAMPLES);

    ObservabilityOverviewResponse {
        workspace_id,
        generated_at,
        window_hours,
        workflow_count: workflows.len(),
        execution_count: executions.len(),
        terminal_execution_count,
        successful_execution_count,
        failed_execution_count,
        stopped_execution_count,
        queued_execution_count,
        running_execution_count,
        waiting_execution_count,
        success_rate: percentage(
            successful_execution_count as u64,
            terminal_execution_count as u64,
        ),
        failure_rate: percentage(
            failed_execution_count as u64,
            terminal_execution_count as u64,
        ),
        average_execution_duration_ms,
        node_latency_histograms,
        workflow_bottlenecks,
        failure_clusters,
        credential_health,
        execution_flamegraphs,
    }
}

fn build_workflow_contexts(workflows: &[WorkflowEntity]) -> HashMap<Uuid, WorkflowContext> {
    workflows
        .iter()
        .map(|workflow| {
            let mut nodes_by_id = HashMap::new();
            let mut nodes_by_name = HashMap::new();

            for node in workflow
                .nodes
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(workflow_node_from_value)
            {
                nodes_by_id.insert(node.id.clone(), node.clone().into());
                nodes_by_name.insert(node.node_name.clone(), node.into());
            }

            (
                workflow.id,
                WorkflowContext {
                    workflow_name: workflow.name.clone(),
                    nodes_by_id,
                    nodes_by_name,
                },
            )
        })
        .collect()
}

fn workflow_node_from_value(value: &Value) -> Option<WorkflowNodeContextValue> {
    Some(WorkflowNodeContextValue {
        id: value.get("id")?.as_str()?.to_string(),
        node_name: value.get("name")?.as_str()?.to_string(),
        node_type: value
            .get("type")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string(),
    })
}

#[derive(Clone)]
struct WorkflowNodeContextValue {
    id: String,
    node_name: String,
    node_type: String,
}

impl From<WorkflowNodeContextValue> for WorkflowNodeContext {
    fn from(value: WorkflowNodeContextValue) -> Self {
        Self {
            node_name: value.node_name,
            node_type: value.node_type,
        }
    }
}

fn resolve_node_context(
    workflow_context: Option<&WorkflowContext>,
    node_id: Option<&str>,
    node_name: Option<&str>,
) -> WorkflowNodeContext {
    if let Some(context) = workflow_context {
        if let Some(node_id) = node_id {
            if let Some(node) = context.nodes_by_id.get(node_id) {
                return node.clone();
            }
        }
        if let Some(node_name) = node_name {
            if let Some(node) = context.nodes_by_name.get(node_name) {
                return node.clone();
            }
        }
    }

    WorkflowNodeContext {
        node_name: node_name.unwrap_or("Unknown node").to_string(),
        node_type: "unknown".to_string(),
    }
}

fn resolve_node_key(node_id: Option<&str>, node_name: Option<&str>) -> String {
    node_id
        .map(ToString::to_string)
        .or_else(|| node_name.map(ToString::to_string))
        .unwrap_or_else(|| "unknown-node".to_string())
}

fn execution_duration_ms(execution: &ExecutionEntity) -> Option<u64> {
    execution.stopped_at.map(|stopped_at| {
        (stopped_at - execution.started_at)
            .num_milliseconds()
            .max(0) as u64
    })
}

fn execution_total_duration_ms(execution: &ExecutionEntity, spans: &[CompletedNodeSpan]) -> u64 {
    execution_duration_ms(execution).unwrap_or_else(|| {
        spans
            .last()
            .map(|span| {
                (span.finished_at - execution.started_at)
                    .num_milliseconds()
                    .max(0) as u64
            })
            .unwrap_or_default()
    })
}

fn extract_usize(value: Option<&Value>) -> usize {
    value
        .and_then(Value::as_u64)
        .map(|value| value as usize)
        .unwrap_or_default()
}

fn extract_error_message(payload: &Value) -> Option<String> {
    payload
        .get("error")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|message| !message.is_empty())
        .map(ToString::to_string)
}

fn build_latency_histogram(durations_ms: &[u64]) -> Vec<LatencyBucketResponse> {
    let mut counts = vec![0usize; LATENCY_BUCKETS.len()];

    for duration in durations_ms {
        for (index, (upper_bound, _label)) in LATENCY_BUCKETS.iter().enumerate() {
            if duration <= upper_bound {
                counts[index] += 1;
                break;
            }
        }
    }

    LATENCY_BUCKETS
        .iter()
        .enumerate()
        .map(|(index, (_upper_bound, label))| LatencyBucketResponse {
            label: (*label).to_string(),
            count: counts[index],
        })
        .collect()
}

fn average_u64(values: &[u64]) -> u64 {
    if values.is_empty() {
        return 0;
    }
    values.iter().sum::<u64>() / values.len() as u64
}

fn percentile_u64(values: &[u64], percentile: f64) -> u64 {
    if values.is_empty() {
        return 0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let index = (((sorted.len() - 1) as f64) * (percentile / 100.0)).round() as usize;
    sorted[index.min(sorted.len() - 1)]
}

fn percentage(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        return 0.0;
    }

    (((numerator as f64 / denominator as f64) * 1000.0).round()) / 10.0
}

fn compare_latency_histograms(
    left: &NodeLatencyHistogramResponse,
    right: &NodeLatencyHistogramResponse,
) -> Ordering {
    right
        .p95_duration_ms
        .cmp(&left.p95_duration_ms)
        .then_with(|| right.avg_duration_ms.cmp(&left.avg_duration_ms))
        .then_with(|| right.failed_runs.cmp(&left.failed_runs))
        .then_with(|| left.workflow_name.cmp(&right.workflow_name))
}

fn compare_bottlenecks(
    left: &WorkflowBottleneckResponse,
    right: &WorkflowBottleneckResponse,
) -> Ordering {
    right
        .avg_duration_ms
        .cmp(&left.avg_duration_ms)
        .then_with(|| right.p95_duration_ms.cmp(&left.p95_duration_ms))
        .then_with(|| right.failure_count.cmp(&left.failure_count))
        .then_with(|| left.workflow_name.cmp(&right.workflow_name))
}

fn compare_credentials(
    left: &CredentialHealthResponse,
    right: &CredentialHealthResponse,
) -> Ordering {
    credential_rank(&left.health)
        .cmp(&credential_rank(&right.health))
        .then_with(|| right.usage_count.cmp(&left.usage_count))
        .then_with(|| left.name.cmp(&right.name))
}

fn compare_flamegraphs(
    left: &ExecutionFlamegraphResponse,
    right: &ExecutionFlamegraphResponse,
) -> Ordering {
    flamegraph_priority(&right.status)
        .cmp(&flamegraph_priority(&left.status))
        .then_with(|| right.total_duration_ms.cmp(&left.total_duration_ms))
        .then_with(|| right.started_at.cmp(&left.started_at))
}

fn flamegraph_priority(status: &str) -> u8 {
    match status.to_ascii_lowercase().as_str() {
        "failed" | "error" | "crashed" => 4,
        "waiting" => 3,
        "running" => 2,
        "stopped" | "cancelled" => 1,
        _ => 0,
    }
}

fn credential_rank(health: &str) -> u8 {
    match health {
        "critical" => 0,
        "warning" => 1,
        "idle" => 2,
        _ => 3,
    }
}

fn score_credential_health(credential: &CredentialEntity) -> CredentialHealthResponse {
    let mut issues = Vec::new();
    let now = Utc::now();
    let last_test_status = credential.last_test_status.clone();

    if matches!(
        credential
            .last_test_status
            .as_deref()
            .map(|status| status.to_ascii_lowercase()),
        Some(status) if status == "invalid" || status == "error"
    ) {
        issues.push("Latest validation failed".to_string());
    }

    match credential.last_tested_at {
        None => issues.push("Credential has never been validated".to_string()),
        Some(last_tested_at) if (now - last_tested_at) > Duration::days(30) => {
            issues.push("Validation result is older than 30 days".to_string())
        }
        _ => {}
    }

    if let Some(rotated_at) = credential.rotated_at {
        if (now - rotated_at) > Duration::days(120) {
            issues.push("Credential has not been rotated for more than 120 days".to_string());
        }
    } else {
        issues.push("Credential has no recorded rotation event".to_string());
    }

    if credential.usage_count == 0 {
        issues.push("Credential has not been used yet".to_string());
    } else if let Some(last_used_at) = credential.last_used_at {
        if (now - last_used_at) > Duration::days(45) {
            issues.push("Credential appears inactive for more than 45 days".to_string());
        }
    }

    let health = if matches!(
        last_test_status
            .as_deref()
            .map(|status| status.to_ascii_lowercase()),
        Some(status) if status == "invalid" || status == "error"
    ) {
        "critical"
    } else if credential.usage_count == 0 {
        "idle"
    } else if issues.is_empty() {
        "healthy"
    } else {
        "warning"
    };

    CredentialHealthResponse {
        credential_id: credential.id,
        name: credential.name.clone(),
        credential_type: credential.cred_type.clone(),
        health: health.to_string(),
        issues,
        last_test_status: credential.last_test_status.clone(),
        last_tested_at: credential.last_tested_at,
        last_used_at: credential.last_used_at,
        rotated_at: credential.rotated_at,
        usage_count: credential.usage_count,
    }
}

fn record_failure_cluster(
    clusters: &mut HashMap<String, FailureClusterAccumulator>,
    log: &ExecutionLogEntity,
    workflow_context: Option<&WorkflowContext>,
    node_context: &WorkflowNodeContext,
    message_override: Option<String>,
) {
    let raw_message = message_override.unwrap_or_else(|| log.message.clone());
    let normalized_message = normalize_failure_message(&raw_message);
    let cluster_key = format!(
        "{}:{}:{}",
        log.workflow_id,
        node_context.node_name.to_ascii_lowercase(),
        normalized_message
    );

    let cluster =
        clusters
            .entry(cluster_key.clone())
            .or_insert_with(|| FailureClusterAccumulator {
                cluster_key,
                workflow_id: Some(log.workflow_id),
                workflow_name: workflow_context.map(|context| context.workflow_name.clone()),
                node_name: Some(node_context.node_name.clone()),
                node_type: Some(node_context.node_type.clone()),
                level: log.level.clone(),
                event_type: log.event_type.clone(),
                message: raw_message.clone(),
                failure_count: 0,
                execution_ids: HashSet::new(),
                last_seen_at: log.created_at,
            });

    cluster.failure_count += 1;
    cluster.execution_ids.insert(log.execution_id);
    if log.created_at > cluster.last_seen_at {
        cluster.last_seen_at = log.created_at;
    }
}

fn normalize_failure_message(message: &str) -> String {
    message
        .chars()
        .map(|character| {
            if character.is_ascii_digit() {
                '#'
            } else {
                character.to_ascii_lowercase()
            }
        })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn assemble_overview_derives_latency_failures_and_flamegraphs() {
        let workspace_id = Uuid::new_v4();
        let workflow_id = Uuid::new_v4();
        let execution_success_id = Uuid::new_v4();
        let execution_failed_id = Uuid::new_v4();
        let generated_at = DateTime::parse_from_rfc3339("2026-03-10T20:00:00Z")
            .unwrap()
            .with_timezone(&Utc);

        let workflows = vec![WorkflowEntity {
            id: workflow_id,
            workspace_id,
            owner_user_id: None,
            name: "Incident Response".to_string(),
            active: true,
            nodes: json!([
                {"id": "node-trigger", "name": "Trigger", "type": "manualTrigger"},
                {"id": "node-http", "name": "Fetch Incident", "type": "httpRequest"},
                {"id": "node-slack", "name": "Notify Slack", "type": "slack"}
            ]),
            connections: json!({}),
            settings: json!({}),
            created_at: generated_at,
            updated_at: generated_at,
        }];

        let executions = vec![
            ExecutionEntity {
                id: execution_success_id,
                workflow_id,
                status: "success".to_string(),
                data: json!({}),
                started_at: DateTime::parse_from_rfc3339("2026-03-10T18:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                stopped_at: Some(
                    DateTime::parse_from_rfc3339("2026-03-10T18:00:10Z")
                        .unwrap()
                        .with_timezone(&Utc),
                ),
            },
            ExecutionEntity {
                id: execution_failed_id,
                workflow_id,
                status: "failed".to_string(),
                data: json!({}),
                started_at: DateTime::parse_from_rfc3339("2026-03-10T19:00:00Z")
                    .unwrap()
                    .with_timezone(&Utc),
                stopped_at: Some(
                    DateTime::parse_from_rfc3339("2026-03-10T19:00:12Z")
                        .unwrap()
                        .with_timezone(&Utc),
                ),
            },
        ];

        let logs = vec![
            ExecutionLogEntity {
                id: Uuid::new_v4(),
                execution_id: execution_success_id,
                workflow_id,
                level: "info".to_string(),
                event_type: Some("nodeStarted".to_string()),
                message: "Node 'Fetch Incident' started".to_string(),
                node_id: Some("node-http".to_string()),
                node_name: Some("Fetch Incident".to_string()),
                payload: json!({"inputItems": 1}),
                created_at: DateTime::parse_from_rfc3339("2026-03-10T18:00:01Z")
                    .unwrap()
                    .with_timezone(&Utc),
            },
            ExecutionLogEntity {
                id: Uuid::new_v4(),
                execution_id: execution_success_id,
                workflow_id,
                level: "info".to_string(),
                event_type: Some("nodeFinished".to_string()),
                message: "Node 'Fetch Incident' completed".to_string(),
                node_id: Some("node-http".to_string()),
                node_name: Some("Fetch Incident".to_string()),
                payload: json!({"success": true, "outputItems": 1}),
                created_at: DateTime::parse_from_rfc3339("2026-03-10T18:00:06Z")
                    .unwrap()
                    .with_timezone(&Utc),
            },
            ExecutionLogEntity {
                id: Uuid::new_v4(),
                execution_id: execution_success_id,
                workflow_id,
                level: "info".to_string(),
                event_type: Some("nodeStarted".to_string()),
                message: "Node 'Notify Slack' started".to_string(),
                node_id: Some("node-slack".to_string()),
                node_name: Some("Notify Slack".to_string()),
                payload: json!({"inputItems": 1}),
                created_at: DateTime::parse_from_rfc3339("2026-03-10T18:00:06Z")
                    .unwrap()
                    .with_timezone(&Utc),
            },
            ExecutionLogEntity {
                id: Uuid::new_v4(),
                execution_id: execution_success_id,
                workflow_id,
                level: "info".to_string(),
                event_type: Some("nodeFinished".to_string()),
                message: "Node 'Notify Slack' completed".to_string(),
                node_id: Some("node-slack".to_string()),
                node_name: Some("Notify Slack".to_string()),
                payload: json!({"success": true, "outputItems": 1}),
                created_at: DateTime::parse_from_rfc3339("2026-03-10T18:00:08Z")
                    .unwrap()
                    .with_timezone(&Utc),
            },
            ExecutionLogEntity {
                id: Uuid::new_v4(),
                execution_id: execution_failed_id,
                workflow_id,
                level: "info".to_string(),
                event_type: Some("nodeStarted".to_string()),
                message: "Node 'Fetch Incident' started".to_string(),
                node_id: Some("node-http".to_string()),
                node_name: Some("Fetch Incident".to_string()),
                payload: json!({"inputItems": 1}),
                created_at: DateTime::parse_from_rfc3339("2026-03-10T19:00:01Z")
                    .unwrap()
                    .with_timezone(&Utc),
            },
            ExecutionLogEntity {
                id: Uuid::new_v4(),
                execution_id: execution_failed_id,
                workflow_id,
                level: "error".to_string(),
                event_type: Some("nodeFinished".to_string()),
                message: "Node 'Fetch Incident' failed".to_string(),
                node_id: Some("node-http".to_string()),
                node_name: Some("Fetch Incident".to_string()),
                payload: json!({"success": false, "error": "HTTP 503 from upstream", "outputItems": 0}),
                created_at: DateTime::parse_from_rfc3339("2026-03-10T19:00:09Z")
                    .unwrap()
                    .with_timezone(&Utc),
            },
            ExecutionLogEntity {
                id: Uuid::new_v4(),
                execution_id: execution_failed_id,
                workflow_id,
                level: "error".to_string(),
                event_type: Some("failed".to_string()),
                message: "Execution failed: HTTP 503 from upstream".to_string(),
                node_id: Some("node-http".to_string()),
                node_name: Some("Fetch Incident".to_string()),
                payload: json!({}),
                created_at: DateTime::parse_from_rfc3339("2026-03-10T19:00:12Z")
                    .unwrap()
                    .with_timezone(&Utc),
            },
        ];

        let credentials = vec![CredentialEntity {
            id: Uuid::new_v4(),
            workspace_id,
            owner_user_id: None,
            name: "Ops Slack".to_string(),
            cred_type: "slackApi".to_string(),
            data: json!({}),
            created_at: generated_at,
            updated_at: generated_at,
            last_tested_at: Some(generated_at - Duration::days(40)),
            last_test_status: Some("valid".to_string()),
            last_test_message: None,
            last_used_at: Some(generated_at - Duration::days(5)),
            usage_count: 12,
            rotated_at: Some(generated_at - Duration::days(150)),
        }];

        let overview = assemble_observability_overview(
            &workflows,
            &executions,
            &logs,
            &credentials,
            workspace_id,
            72,
            generated_at,
        );

        assert_eq!(overview.workflow_count, 1);
        assert_eq!(overview.execution_count, 2);
        assert_eq!(overview.successful_execution_count, 1);
        assert_eq!(overview.failed_execution_count, 1);
        assert_eq!(overview.terminal_execution_count, 2);
        assert_eq!(overview.success_rate, 50.0);
        assert_eq!(overview.failure_rate, 50.0);
        assert_eq!(overview.node_latency_histograms.len(), 2);
        assert_eq!(
            overview.node_latency_histograms[0].node_name,
            "Fetch Incident"
        );
        assert_eq!(overview.node_latency_histograms[0].failed_runs, 1);
        assert_eq!(overview.workflow_bottlenecks.len(), 1);
        assert_eq!(overview.workflow_bottlenecks[0].node_name, "Fetch Incident");
        assert_eq!(overview.failure_clusters[0].affected_execution_count, 1);
        assert_eq!(overview.credential_health[0].health, "warning");
        assert_eq!(overview.execution_flamegraphs.len(), 2);
        assert_eq!(overview.execution_flamegraphs[0].status, "failed");
        assert_eq!(
            overview.execution_flamegraphs[0].spans[0].node_type,
            "httpRequest"
        );
    }

    #[test]
    fn clamp_window_enforces_supported_range() {
        assert_eq!(clamp_observability_window(None), 72);
        assert_eq!(clamp_observability_window(Some(0)), 1);
        assert_eq!(clamp_observability_window(Some(999)), 168);
    }
}
