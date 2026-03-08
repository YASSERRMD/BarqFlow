use crate::controllers::webhooks::{WebhookEndpoint, WebhookRegistry};
use crate::credentials_provider::RepositoryCredentialProvider;
use crate::repositories::credential::CredentialRepository;
use crate::repositories::workflow::WorkflowRepository;
use barqflow_core::schema::{INode, INodeConnections, IWorkflowSettings, WorkflowDef};
use barqflow_core::types::{RunId, WorkflowId};
use barqflow_db::models::WorkflowEntity;
use barqflow_exec::runner::{ExecutionConfig, WorkflowRunContext, WorkflowRunner};
use barqflow_registry::registry::NodeRegistry;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{info, warn};
use uuid::Uuid;

pub type ActiveCronJobs = Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>;

#[derive(Clone)]
pub struct ActiveWorkflowManager {
    pub workflow_repo: Arc<WorkflowRepository>,
    pub credential_repo: Arc<CredentialRepository>,
    pub node_registry: Arc<NodeRegistry>,
    pub webhook_registry: WebhookRegistry,
    pub job_scheduler: JobScheduler,
    pub active_cron_jobs: ActiveCronJobs,
}

impl ActiveWorkflowManager {
    pub fn new(
        workflow_repo: Arc<WorkflowRepository>,
        credential_repo: Arc<CredentialRepository>,
        node_registry: Arc<NodeRegistry>,
        webhook_registry: WebhookRegistry,
        job_scheduler: JobScheduler,
        active_cron_jobs: ActiveCronJobs,
    ) -> Self {
        Self {
            workflow_repo,
            credential_repo,
            node_registry,
            webhook_registry,
            job_scheduler,
            active_cron_jobs,
        }
    }

    pub async fn activate(&self, workflow_id: Uuid) -> Result<WorkflowEntity, String> {
        let workflow = self
            .workflow_repo
            .find_by_id(workflow_id)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Workflow not found".to_string())?;

        self.register_runtime(&workflow).await?;

        self.workflow_repo
            .toggle_active(workflow_id, true)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Workflow not found".to_string())
    }

    pub async fn deactivate(&self, workflow_id: Uuid) -> Result<WorkflowEntity, String> {
        self.unregister_runtime(workflow_id).await?;

        self.workflow_repo
            .toggle_active(workflow_id, false)
            .await
            .map_err(|e| e.to_string())?
            .ok_or_else(|| "Workflow not found".to_string())
    }

    pub async fn reconcile_on_boot(&self) -> Result<(), String> {
        let active = self
            .workflow_repo
            .find_all_by_active(true)
            .await
            .map_err(|e| e.to_string())?;

        for workflow in active {
            self.register_runtime(&workflow).await?;
        }

        Ok(())
    }

    pub async fn register_runtime(&self, workflow: &WorkflowEntity) -> Result<(), String> {
        self.unregister_runtime(workflow.id).await?;

        let nodes: Vec<INode> =
            serde_json::from_value(workflow.nodes.clone()).map_err(|e| e.to_string())?;

        self.register_webhooks(workflow.id, &nodes)?;
        self.register_cron_jobs(workflow.clone(), nodes).await?;

        Ok(())
    }

    pub async fn unregister_runtime(&self, workflow_id: Uuid) -> Result<(), String> {
        {
            let mut registry = self
                .webhook_registry
                .write()
                .map_err(|_| "Failed to lock webhook registry".to_string())?;
            registry.retain(|_, endpoint| endpoint.workflow_id != workflow_id);
        }

        let cron_ids = {
            let mut cron_jobs = self.active_cron_jobs.write().await;
            cron_jobs.remove(&workflow_id).unwrap_or_default()
        };

        for job_id in cron_ids {
            if let Err(err) = self.job_scheduler.remove(&job_id).await {
                warn!(
                    "Failed to remove cron job '{}' for workflow '{}': {}",
                    job_id, workflow_id, err
                );
            }
        }

        Ok(())
    }

    fn register_webhooks(&self, workflow_id: Uuid, nodes: &[INode]) -> Result<(), String> {
        let mut registry = self
            .webhook_registry
            .write()
            .map_err(|_| "Failed to lock webhook registry".to_string())?;

        for node in nodes {
            if !is_webhook_node_type(&node.r#type) {
                continue;
            }

            let path = node
                .parameters
                .0
                .get("path")
                .and_then(|p| p.as_str())
                .unwrap_or_else(|| node.id.0.as_str())
                .to_string();

            let http_method = node
                .parameters
                .0
                .get("httpMethod")
                .and_then(|m| m.as_str())
                .unwrap_or("ANY")
                .to_string();

            registry.insert(
                path.clone(),
                WebhookEndpoint {
                    workflow_id,
                    node_id: node.id.to_string(),
                    http_method,
                },
            );

            info!(
                "Registered webhook '/webhook/{}' for workflow '{}'",
                path, workflow_id
            );
        }

        Ok(())
    }

    async fn register_cron_jobs(
        &self,
        workflow: WorkflowEntity,
        nodes: Vec<INode>,
    ) -> Result<(), String> {
        let mut job_ids = Vec::new();

        for node in &nodes {
            if !is_cron_node_type(&node.r#type) {
                continue;
            }

            let cron_expr = node
                .parameters
                .0
                .get("cron")
                .and_then(|p| p.as_str())
                .unwrap_or("0 * * * * *")
                .to_string();

            let workflow_clone = workflow.clone();
            let nodes_clone = nodes.clone();
            let node_registry = Arc::clone(&self.node_registry);
            let credential_repo = Arc::clone(&self.credential_repo);

            let job = Job::new_async(cron_expr.as_str(), move |_uuid, _l| {
                let workflow_clone = workflow_clone.clone();
                let nodes_clone = nodes_clone.clone();
                let node_registry = Arc::clone(&node_registry);
                let credential_repo = Arc::clone(&credential_repo);

                Box::pin(async move {
                    let connections: HashMap<String, INodeConnections> =
                        serde_json::from_value(workflow_clone.connections.clone())
                            .unwrap_or_default();
                    let settings: IWorkflowSettings =
                        serde_json::from_value(workflow_clone.settings.clone())
                            .unwrap_or_default();

                    let workflow_def = WorkflowDef {
                        id: WorkflowId(workflow_clone.id),
                        name: workflow_clone.name.clone(),
                        nodes: nodes_clone.clone(),
                        connections: connections.into_iter().collect(),
                        active: workflow_clone.active,
                        settings,
                    };

                    let credential_provider = Arc::new(RepositoryCredentialProvider::new(
                        Arc::clone(&credential_repo),
                        &nodes_clone,
                    ));
                    let runner =
                        WorkflowRunner::new(node_registry, ExecutionConfig::default())
                            .with_credential_provider(credential_provider);
                    let ctx = WorkflowRunContext {
                        run_id: RunId::new(),
                        workflow: workflow_def,
                        static_data: None,
                        manual: false,
                        execution_id: None,
                        cancellation_token: None,
                    };

                    if let Err(err) = runner.run_workflow(ctx).await {
                        warn!(
                            "Scheduled workflow '{}' execution failed: {}",
                            workflow_clone.id, err
                        );
                    }
                })
            })
            .map_err(|e| format!("Failed to build cron job '{}': {}", cron_expr, e))?;

            let job_id = job.guid();
            self.job_scheduler
                .add(job)
                .await
                .map_err(|e| format!("Failed to add cron job '{}': {}", cron_expr, e))?;
            job_ids.push(job_id);

            info!(
                "Registered cron '{}' for workflow '{}'",
                cron_expr, workflow.id
            );
        }

        if !job_ids.is_empty() {
            let mut active = self.active_cron_jobs.write().await;
            active.insert(workflow.id, job_ids);
        }

        Ok(())
    }
}

fn is_webhook_node_type(node_type: &str) -> bool {
    matches!(node_type, "webhook" | "barqflow-nodes.webhook")
}

fn is_cron_node_type(node_type: &str) -> bool {
    matches!(node_type, "cron" | "barqflow-nodes.cronTrigger")
}
