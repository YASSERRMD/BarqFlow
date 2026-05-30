use barqflow_core::errors::BarqError;
use barqflow_core::types::WorkflowId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::{error, info};
use uuid::Uuid;

pub type JobId = Uuid;

pub struct WorkflowCronScheduler {
    scheduler: JobScheduler,
    job_map: Arc<RwLock<HashMap<WorkflowId, Vec<JobId>>>>,
}

impl WorkflowCronScheduler {
    pub async fn new() -> Result<Self, BarqError> {
        let scheduler = JobScheduler::new().await.map_err(|e| {
            BarqError::InternalError(format!("Failed to create JobScheduler: {}", e))
        })?;

        Ok(Self {
            scheduler,
            job_map: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn start(&self) -> Result<(), BarqError> {
        self.scheduler.start().await.map_err(|e| {
            BarqError::InternalError(format!("Failed to start JobScheduler: {}", e))
        })?;
        info!("WorkflowCronScheduler started");
        Ok(())
    }

    pub async fn add_workflow_schedule<F>(
        &self,
        workflow_id: WorkflowId,
        cron_expression: &str,
        callback: F,
    ) -> Result<JobId, BarqError>
    where
        F: Fn() + Send + Sync + 'static,
    {
        let cb = Arc::new(callback);
        let job = Job::new_async(cron_expression, move |_uuid, _l| {
            let cb = cb.clone();
            Box::pin(async move {
                cb();
            })
        })
        .map_err(|e| {
            BarqError::InternalError(format!(
                "Invalid cron expression '{}': {}",
                cron_expression, e
            ))
        })?;

        let guid = job.guid();

        self.scheduler
            .add(job)
            .await
            .map_err(|e| BarqError::InternalError(format!("Failed to add job: {}", e)))?;

        let mut map = self.job_map.write().await;
        map.entry(workflow_id).or_default().push(guid);

        info!(
            "Added schedule for workflow {} with ID {}",
            workflow_id.0, guid
        );

        Ok(guid)
    }

    pub async fn remove_workflow_schedules(
        &self,
        workflow_id: &WorkflowId,
    ) -> Result<(), BarqError> {
        let mut map = self.job_map.write().await;
        if let Some(jobs) = map.remove(workflow_id) {
            for job_id in jobs {
                if let Err(e) = self.scheduler.remove(&job_id).await {
                    error!("Failed to remove job {}: {}", job_id, e);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = WorkflowCronScheduler::new().await;
        assert!(scheduler.is_ok());
    }
}
