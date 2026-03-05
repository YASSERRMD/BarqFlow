//! Scheduler & Polling Engine
//!
//! Implements tokio-cron-scheduler integration for scheduled triggers.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Schedule ID mapped to workflow ID
pub type ScheduleId = String;

/// Scheduler for managing cron jobs
pub struct Scheduler {
    active_schedules: Arc<RwLock<HashMap<ScheduleId, String>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            active_schedules: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_schedule(&self, schedule_id: &str, workflow_id: &str) {
        let mut schedules = self.active_schedules.write().await;
        schedules.insert(schedule_id.to_string(), workflow_id.to_string());
    }

    pub async fn remove_schedule(&self, schedule_id: &str) {
        let mut schedules = self.active_schedules.write().await;
        schedules.remove(schedule_id);
    }

    pub async fn list_schedules(&self) -> Vec<(ScheduleId, String)> {
        let schedules = self.active_schedules.read().await;
        schedules
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    pub async fn get_workflow_for_schedule(&self, schedule_id: &str) -> Option<String> {
        let schedules = self.active_schedules.read().await;
        schedules.get(schedule_id).cloned()
    }
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let scheduler = Scheduler::new();
        let schedules = scheduler.list_schedules().await;
        assert!(schedules.is_empty());
    }

    #[tokio::test]
    async fn test_add_remove_schedule() {
        let scheduler = Scheduler::new();

        scheduler.add_schedule("hourly", "workflow-1").await;
        let schedules = scheduler.list_schedules().await;
        assert_eq!(schedules.len(), 1);

        scheduler.remove_schedule("hourly").await;
        let schedules = scheduler.list_schedules().await;
        assert!(schedules.is_empty());
    }
}
