//! Execution Checkpointing for Wait Nodes
//!
//! Implements checkpointing for suspend/resume execution workflows,
//! enabling Wait nodes to pause execution and resume later.

use barqflow_core::types::RunId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::io::AsyncWriteExt;
use tracing::{debug, info};

/// Checkpoint data for a suspended execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionCheckpoint {
    /// Unique identifier for this execution run
    pub run_id: RunId,
    /// The workflow ID being executed
    pub workflow_id: String,
    /// Timestamp when execution was suspended
    pub suspended_at: DateTime<Utc>,
    /// Index of the node that was being executed when suspended
    pub current_node_index: usize,
    /// Data that was being processed at suspension time
    pub node_data: serde_json::Value,
    /// Static workflow data
    pub static_data: Option<serde_json::Value>,
    /// Wait configuration if suspended at a Wait node
    pub wait_config: Option<WaitConfig>,
    /// How long to wait before resuming (for Wait nodes)
    pub resume_after: Option<DateTime<Utc>>,
}

/// Configuration for a Wait node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WaitConfig {
    /// Type of wait (Time, Webhook, External)
    pub wait_type: WaitType,
    /// Duration in milliseconds (for Time type)
    pub duration_ms: Option<u64>,
    /// Webhook URL/path (for Webhook type)
    pub webhook_path: Option<String>,
    /// External system identifier (for External type)
    pub external_id: Option<String>,
}

/// Type of wait operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum WaitType {
    /// Wait for a specified duration
    Time,
    /// Wait for webhook trigger
    Webhook,
    /// Wait for external system callback
    External,
    /// Wait for sub-workflow execution to complete
    SubWorkflow,
}

/// Checkpoint storage backend.
#[derive(Debug, Clone)]
pub enum CheckpointStorage {
    /// Filesystem-based storage
    Filesystem { base_path: PathBuf },
    /// In-memory storage (for testing)
    Memory,
}

/// Checkpoint manager for saving and loading execution state.
pub struct CheckpointManager {
    storage: CheckpointStorage,
    /// In-memory cache for checkpoints (used with Memory storage or as cache)
    cache: HashMap<RunId, ExecutionCheckpoint>,
}

impl CheckpointManager {
    /// Create a new checkpoint manager.
    ///
    /// # Arguments
    /// * `storage` - The storage backend to use
    pub fn new(storage: CheckpointStorage) -> Self {
        Self {
            storage,
            cache: HashMap::new(),
        }
    }

    /// Create a checkpoint manager with filesystem storage.
    ///
    /// # Arguments
    /// * `base_path` - Directory to store checkpoints
    pub fn with_filesystem(base_path: PathBuf) -> Self {
        Self::new(CheckpointStorage::Filesystem { base_path })
    }

    /// Create a checkpoint manager with in-memory storage.
    pub fn with_memory() -> Self {
        Self::new(CheckpointStorage::Memory)
    }

    /// Save a checkpoint.
    ///
    /// # Arguments
    /// * `checkpoint` - The checkpoint to save
    ///
    /// # Returns
    /// Ok(()) if saved successfully, Err otherwise
    pub async fn save_checkpoint(&mut self, checkpoint: ExecutionCheckpoint) -> Result<(), String> {
        let run_id = checkpoint.run_id;
        debug!("Saving checkpoint for run {}", run_id);

        // Clone the path before we borrow self for the match
        let storage = self.storage.clone();

        match &storage {
            CheckpointStorage::Filesystem { base_path } => {
                self.save_to_filesystem(base_path, &checkpoint).await?
            }
            CheckpointStorage::Memory => {
                self.cache.insert(checkpoint.run_id, checkpoint);
            }
        }

        info!("Checkpoint saved for run {}", run_id);
        Ok(())
    }

    /// Load a checkpoint.
    ///
    /// # Arguments
    /// * `run_id` - The run ID to load
    ///
    /// # Returns
    /// Some(checkpoint) if found, None otherwise
    pub async fn load_checkpoint(&self, run_id: &RunId) -> Option<ExecutionCheckpoint> {
        debug!("Loading checkpoint for run {}", run_id);

        match &self.storage {
            CheckpointStorage::Filesystem { base_path } => {
                self.load_from_filesystem(base_path, run_id).await
            }
            CheckpointStorage::Memory => self.cache.get(run_id).cloned(),
        }
    }

    /// Delete a checkpoint.
    ///
    /// # Arguments
    /// * `run_id` - The run ID to delete
    ///
    /// # Returns
    /// Ok(()) if deleted or didn't exist, Err otherwise
    pub async fn delete_checkpoint(&mut self, run_id: &RunId) -> Result<(), String> {
        debug!("Deleting checkpoint for run {}", run_id);

        // Clone the path before we borrow self for the match
        let storage = self.storage.clone();

        match &storage {
            CheckpointStorage::Filesystem { base_path } => {
                self.delete_from_filesystem(base_path, run_id).await?
            }
            CheckpointStorage::Memory => {
                self.cache.remove(run_id);
            }
        }

        info!("Checkpoint deleted for run {}", run_id);
        Ok(())
    }

    /// List all checkpointed runs.
    ///
    /// # Returns
    /// Vector of run IDs that have checkpoints
    pub async fn list_checkpoints(&self) -> Vec<RunId> {
        match &self.storage {
            CheckpointStorage::Filesystem { base_path } => {
                self.list_from_filesystem(base_path).await
            }
            CheckpointStorage::Memory => self.cache.keys().copied().collect(),
        }
    }

    /// Check if a checkpoint is ready to be resumed.
    ///
    /// # Arguments
    /// * `run_id` - The run ID to check
    ///
    /// # Returns
    /// true if ready to resume, false otherwise
    pub async fn is_ready_to_resume(&self, run_id: &RunId) -> bool {
        if let Some(checkpoint) = self.load_checkpoint(run_id).await {
            if let Some(resume_after) = checkpoint.resume_after {
                return Utc::now() >= resume_after;
            }
            // If no resume_after, it's ready (e.g., webhook triggered)
            return true;
        }
        false
    }

    /// Save checkpoint to filesystem.
    async fn save_to_filesystem(
        &mut self,
        base_path: &PathBuf,
        checkpoint: &ExecutionCheckpoint,
    ) -> Result<(), String> {
        // Ensure directory exists
        fs::create_dir_all(base_path)
            .await
            .map_err(|e| format!("Failed to create checkpoint directory: {}", e))?;

        // Create file path
        let file_path = base_path.join(format!("checkpoint_{}.json", checkpoint.run_id));

        // Serialize checkpoint
        let json = serde_json::to_string_pretty(checkpoint)
            .map_err(|e| format!("Failed to serialize checkpoint: {}", e))?;

        // Write to file
        let mut file = fs::File::create(&file_path)
            .await
            .map_err(|e| format!("Failed to create checkpoint file: {}", e))?;

        file.write_all(json.as_bytes())
            .await
            .map_err(|e| format!("Failed to write checkpoint: {}", e))?;

        // Also cache in memory
        self.cache.insert(checkpoint.run_id, checkpoint.clone());

        Ok(())
    }

    /// Load checkpoint from filesystem.
    async fn load_from_filesystem(
        &self,
        base_path: &PathBuf,
        run_id: &RunId,
    ) -> Option<ExecutionCheckpoint> {
        // Check cache first
        if let Some(cached) = self.cache.get(run_id) {
            return Some(cached.clone());
        }

        // Load from file
        let file_path = base_path.join(format!("checkpoint_{}.json", run_id));

        let content = fs::read_to_string(&file_path).await.ok()?;

        let checkpoint: ExecutionCheckpoint = serde_json::from_str(&content).ok()?;

        // Note: We can't cache here because we only have &self, not &mut self
        // The cache will be populated on save

        Some(checkpoint)
    }

    /// Delete checkpoint from filesystem.
    async fn delete_from_filesystem(
        &mut self,
        base_path: &PathBuf,
        run_id: &RunId,
    ) -> Result<(), String> {
        // Remove from cache
        self.cache.remove(run_id);

        // Delete file
        let file_path = base_path.join(format!("checkpoint_{}.json", run_id));

        fs::remove_file(&file_path)
            .await
            .map_err(|e| format!("Failed to delete checkpoint file: {}", e))?;

        Ok(())
    }

    /// List checkpoints from filesystem.
    async fn list_from_filesystem(&self, base_path: &PathBuf) -> Vec<RunId> {
        let mut checkpoints = Vec::new();

        if let Ok(mut entries) = fs::read_dir(base_path).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Ok(name) = entry.file_name().into_string() {
                    if name.starts_with("checkpoint_") && name.ends_with(".json") {
                        // Extract run_id from filename
                        let run_id_str = name
                            .strip_prefix("checkpoint_")
                            .and_then(|s| s.strip_suffix(".json"))
                            .unwrap_or("");

                        if let Ok(uuid) = uuid::Uuid::parse_str(run_id_str) {
                            checkpoints.push(RunId(uuid));
                        }
                    }
                }
            }
        }

        checkpoints
    }
}

impl Default for CheckpointManager {
    fn default() -> Self {
        Self::with_memory()
    }
}

/// Builder for creating ExecutionCheckpoint instances.
pub struct ExecutionCheckpointBuilder {
    run_id: Option<RunId>,
    workflow_id: Option<String>,
    current_node_index: Option<usize>,
    node_data: Option<serde_json::Value>,
    static_data: Option<serde_json::Value>,
    wait_config: Option<WaitConfig>,
    resume_after: Option<DateTime<Utc>>,
}

impl Default for ExecutionCheckpointBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionCheckpointBuilder {
    pub fn new() -> Self {
        Self {
            run_id: None,
            workflow_id: None,
            current_node_index: None,
            node_data: None,
            static_data: None,
            wait_config: None,
            resume_after: None,
        }
    }

    pub fn with_run_id(mut self, run_id: RunId) -> Self {
        self.run_id = Some(run_id);
        self
    }

    pub fn with_workflow_id(mut self, workflow_id: String) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }

    pub fn with_node_index(mut self, index: usize) -> Self {
        self.current_node_index = Some(index);
        self
    }

    pub fn with_node_data(mut self, data: serde_json::Value) -> Self {
        self.node_data = Some(data);
        self
    }

    pub fn with_static_data(mut self, data: serde_json::Value) -> Self {
        self.static_data = Some(data);
        self
    }

    pub fn with_wait_config(mut self, config: WaitConfig) -> Self {
        self.wait_config = Some(config);
        self
    }

    pub fn with_resume_after(mut self, time: DateTime<Utc>) -> Self {
        self.resume_after = Some(time);
        self
    }

    pub fn build(self) -> Result<ExecutionCheckpoint, String> {
        Ok(ExecutionCheckpoint {
            run_id: self.run_id.ok_or("run_id is required")?,
            workflow_id: self.workflow_id.ok_or("workflow_id is required")?,
            suspended_at: Utc::now(),
            current_node_index: self.current_node_index.unwrap_or(0),
            node_data: self.node_data.unwrap_or(serde_json::Value::Null),
            static_data: self.static_data,
            wait_config: self.wait_config,
            resume_after: self.resume_after,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_memory_checkpoint_save_load() {
        let mut manager = CheckpointManager::with_memory();
        let run_id = RunId::new();

        let checkpoint = ExecutionCheckpointBuilder::new()
            .with_run_id(run_id)
            .with_workflow_id("test-workflow".to_string())
            .with_node_index(5)
            .with_node_data(json!({"test": "data"}))
            .build()
            .unwrap();

        manager.save_checkpoint(checkpoint).await.unwrap();

        let loaded = manager.load_checkpoint(&run_id).await;
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().workflow_id, "test-workflow");
    }

    #[tokio::test]
    async fn test_memory_checkpoint_delete() {
        let mut manager = CheckpointManager::with_memory();
        let run_id = RunId::new();

        let checkpoint = ExecutionCheckpointBuilder::new()
            .with_run_id(run_id)
            .with_workflow_id("test-workflow".to_string())
            .build()
            .unwrap();

        manager.save_checkpoint(checkpoint).await.unwrap();
        manager.delete_checkpoint(&run_id).await.unwrap();

        let loaded = manager.load_checkpoint(&run_id).await;
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_memory_checkpoint_list() {
        let mut manager = CheckpointManager::with_memory();

        let run_id1 = RunId::new();
        let run_id2 = RunId::new();

        let checkpoint1 = ExecutionCheckpointBuilder::new()
            .with_run_id(run_id1)
            .with_workflow_id("workflow1".to_string())
            .build()
            .unwrap();

        let checkpoint2 = ExecutionCheckpointBuilder::new()
            .with_run_id(run_id2)
            .with_workflow_id("workflow2".to_string())
            .build()
            .unwrap();

        manager.save_checkpoint(checkpoint1).await.unwrap();
        manager.save_checkpoint(checkpoint2).await.unwrap();

        let list = manager.list_checkpoints().await;
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_filesystem_checkpoint() {
        let temp_dir = TempDir::new().unwrap();
        let mut manager = CheckpointManager::with_filesystem(temp_dir.path().to_path_buf());

        let run_id = RunId::new();

        let checkpoint = ExecutionCheckpointBuilder::new()
            .with_run_id(run_id)
            .with_workflow_id("fs-workflow".to_string())
            .with_node_data(json!({"key": "value"}))
            .build()
            .unwrap();

        manager.save_checkpoint(checkpoint).await.unwrap();

        // Create new manager to test persistence
        let manager2 = CheckpointManager::with_filesystem(temp_dir.path().to_path_buf());
        let loaded = manager2.load_checkpoint(&run_id).await;

        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().node_data, json!({"key": "value"}));
    }

    #[tokio::test]
    async fn test_is_ready_to_resume() {
        let mut manager = CheckpointManager::with_memory();
        let run_id = RunId::new();

        // Create checkpoint with past resume time
        let past_time = Utc::now() - chrono::Duration::seconds(60);
        let checkpoint = ExecutionCheckpointBuilder::new()
            .with_run_id(run_id)
            .with_workflow_id("test".to_string())
            .with_resume_after(past_time)
            .build()
            .unwrap();

        manager.save_checkpoint(checkpoint).await.unwrap();

        assert!(manager.is_ready_to_resume(&run_id).await);
    }

    #[tokio::test]
    async fn test_is_not_ready_to_resume() {
        let mut manager = CheckpointManager::with_memory();
        let run_id = RunId::new();

        // Create checkpoint with future resume time
        let future_time = Utc::now() + chrono::Duration::seconds(60);
        let checkpoint = ExecutionCheckpointBuilder::new()
            .with_run_id(run_id)
            .with_workflow_id("test".to_string())
            .with_resume_after(future_time)
            .build()
            .unwrap();

        manager.save_checkpoint(checkpoint).await.unwrap();

        assert!(!manager.is_ready_to_resume(&run_id).await);
    }

    #[tokio::test]
    async fn test_checkpoint_builder() {
        let run_id = RunId::new();

        let checkpoint = ExecutionCheckpointBuilder::new()
            .with_run_id(run_id)
            .with_workflow_id("builder-test".to_string())
            .with_node_index(10)
            .with_node_data(json!({"item": 1}))
            .build()
            .unwrap();

        assert_eq!(checkpoint.current_node_index, 10);
        assert_eq!(checkpoint.workflow_id, "builder-test");
    }

    #[tokio::test]
    async fn test_checkpoint_builder_missing_required() {
        let result = ExecutionCheckpointBuilder::new()
            .with_workflow_id("test".to_string())
            .build();

        assert!(result.is_err());
    }

    #[test]
    fn test_wait_config_serialization() {
        let config = WaitConfig {
            wait_type: WaitType::Time,
            duration_ms: Some(5000),
            webhook_path: None,
            external_id: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"waitType\":\"time\""));
        assert!(json.contains("\"durationMs\":5000"));
    }

    #[test]
    fn test_execution_checkpoint_serialization() {
        let run_id = RunId::new();
        let checkpoint = ExecutionCheckpoint {
            run_id,
            workflow_id: "test".to_string(),
            suspended_at: Utc::now(),
            current_node_index: 0,
            node_data: json!({"data": "test"}),
            static_data: None,
            wait_config: None,
            resume_after: None,
        };

        let json = serde_json::to_string(&checkpoint).unwrap();
        let deserialized: ExecutionCheckpoint = serde_json::from_str(&json).unwrap();

        assert_eq!(checkpoint.run_id, deserialized.run_id);
        assert_eq!(checkpoint.workflow_id, deserialized.workflow_id);
    }
}
