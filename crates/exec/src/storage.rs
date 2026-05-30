use barqflow_core::errors::BarqError;
use std::path::{Path, PathBuf};
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct BinaryStorageConfig {
    pub storage_dir: PathBuf,
}

impl Default for BinaryStorageConfig {
    fn default() -> Self {
        // use .barqflow/binaries in the user's home or current working dir
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let storage_dir = Path::new(&home).join(".barqflow").join("binaries");
        Self { storage_dir }
    }
}

pub struct BinaryStorage {
    config: BinaryStorageConfig,
}

impl BinaryStorage {
    /// Create a new BinaryStorage manager, ensuring the storage directory exists.
    pub async fn new(config: BinaryStorageConfig) -> std::io::Result<Self> {
        fs::create_dir_all(&config.storage_dir).await?;
        Ok(Self { config })
    }

    /// Stores raw binary data mapped to a UUID into the filesystem directory
    pub async fn store_binary_to_fs(&self, data: &[u8]) -> Result<String, BarqError> {
        let id = Uuid::new_v4().to_string();
        let file_path = self.config.storage_dir.join(&id);

        fs::write(&file_path, data).await.map_err(|e| {
            BarqError::InternalError(format!("Failed to write binary data to FS: {}", e))
        })?;

        Ok(id)
    }

    /// Recursively retrieves previously stored binary data
    pub async fn read_binary_from_fs(&self, id: &str) -> Result<Vec<u8>, BarqError> {
        // Prevent directory traversal attacks natively by matching alphanumeric UUID only
        if id.contains('/') || id.contains('\\') || id.contains("..") {
            return Err(BarqError::InternalError(
                "Invalid binary ID provided".into(),
            ));
        }

        let file_path = self.config.storage_dir.join(id);

        fs::read(&file_path).await.map_err(|e| {
            BarqError::InternalError(format!("Failed to read binary data {} from FS: {}", id, e))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[tokio::test]
    async fn test_store_and_read_binary() {
        let unique_run = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let temp_dir = std::env::temp_dir().join(format!("barqflow_test_{}", unique_run));

        let config = BinaryStorageConfig {
            storage_dir: temp_dir.clone(),
        };

        let storage = BinaryStorage::new(config).await.unwrap();
        let data = b"Hello BarqFlow Binary Storage Engine!";

        let id = storage
            .store_binary_to_fs(data)
            .await
            .expect("Failed to store");
        let retrieved = storage
            .read_binary_from_fs(&id)
            .await
            .expect("Failed to retrieve");

        assert_eq!(data.to_vec(), retrieved);

        // Cleanup
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    }

    #[tokio::test]
    async fn test_directory_traversal_prevention() {
        let config = BinaryStorageConfig::default();
        let storage = BinaryStorage::new(config).await.unwrap();

        let result = storage.read_binary_from_fs("../../../etc/passwd").await;
        assert!(result.is_err());
    }
}
