//! Binary Data Filesystem Abstraction
//!
//! This module handles storing and retrieving binary data on the filesystem,
//! used for large files that shouldn't be kept in memory.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

/// Configuration for binary data storage.
#[derive(Debug, Clone)]
pub struct BinaryStorageConfig {
    /// Root directory where binary data is stored
    pub root_dir: PathBuf,
}

impl BinaryStorageConfig {
    /// Create a new binary storage configuration.
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
        }
    }

    /// Create a new binary storage configuration with a default temp directory.
    pub fn temp() -> Self {
        Self {
            root_dir: std::env::temp_dir().join("barqflow_binary"),
        }
    }

    /// Ensure the storage directory exists.
    pub fn ensure_dir_exists(&self) -> io::Result<()> {
        fs::create_dir_all(&self.root_dir)?;
        Ok(())
    }

    /// Get the full path for a binary file by ID.
    pub fn get_file_path(&self, id: &str) -> PathBuf {
        self.root_dir.join(id)
    }
}

impl Default for BinaryStorageConfig {
    fn default() -> Self {
        Self::temp()
    }
}

/// Store binary data to the filesystem.
///
/// # Arguments
/// * `config` - The storage configuration
/// * `data` - The binary data to store
///
/// # Returns
/// * The ID of the stored file
pub async fn store_binary_to_fs(
    config: &BinaryStorageConfig,
    data: bytes::Bytes,
) -> io::Result<String> {
    config.ensure_dir_exists()?;

    let id = Uuid::new_v4().to_string();
    let file_path = config.get_file_path(&id);

    let mut file = tokio::fs::File::create(&file_path).await?;
    file.write_all(&data).await?;
    file.sync_all().await?;

    Ok(id)
}

/// Read binary data from the filesystem.
///
/// # Arguments
/// * `config` - The storage configuration
/// * `id` - The ID of the file to read
///
/// # Returns
/// * The binary data as a vector of bytes
pub async fn read_binary_from_fs(config: &BinaryStorageConfig, id: &str) -> io::Result<Vec<u8>> {
    let file_path = config.get_file_path(id);

    let mut file = tokio::fs::File::open(&file_path).await?;
    let metadata = file.metadata().await?;
    let file_size = metadata.len() as usize;

    let mut buffer = Vec::with_capacity(file_size);
    file.read_to_end(&mut buffer).await?;

    Ok(buffer)
}

/// Read binary data from the filesystem in chunks (streaming).
///
/// # Arguments
/// * `config` - The storage configuration
/// * `id` - The ID of the file to read
/// * `chunk_size` - The size of each chunk to read
///
/// # Returns
/// * A stream of byte chunks
pub async fn read_binary_from_fs_chunked(
    config: &BinaryStorageConfig,
    id: &str,
    chunk_size: usize,
) -> io::Result<Vec<Vec<u8>>> {
    let file_path = config.get_file_path(id);

    let mut file = tokio::fs::File::open(&file_path).await?;
    let mut chunks = Vec::new();

    loop {
        let mut buffer = vec![0u8; chunk_size];
        let n = file.read(&mut buffer).await?;

        if n == 0 {
            break;
        }

        if n < chunk_size {
            buffer.truncate(n);
        }

        chunks.push(buffer);
    }

    Ok(chunks)
}

/// Delete binary data from the filesystem.
///
/// # Arguments
/// * `config` - The storage configuration
/// * `id` - The ID of the file to delete
pub async fn delete_binary_from_fs(config: &BinaryStorageConfig, id: &str) -> io::Result<()> {
    let file_path = config.get_file_path(id);
    tokio::fs::remove_file(&file_path).await?;
    Ok(())
}

/// Check if a binary file exists.
///
/// # Arguments
/// * `config` - The storage configuration
/// * `id` - The ID of the file to check
///
/// # Returns
/// * true if the file exists, false otherwise
pub async fn binary_exists(config: &BinaryStorageConfig, id: &str) -> bool {
    let file_path = config.get_file_path(id);
    tokio::fs::metadata(&file_path).await.is_ok()
}

/// Store binary data under an execution-scoped subdirectory.
///
/// Files are stored at `<root>/<execution_id>/<uuid>` so all artifacts for
/// one execution can be cleaned up with a single [`delete_execution_artifacts`]
/// call when the execution completes.
pub async fn store_binary_for_execution(
    config: &BinaryStorageConfig,
    execution_id: Uuid,
    data: bytes::Bytes,
) -> io::Result<String> {
    let exec_dir = config.root_dir.join(execution_id.to_string());
    fs::create_dir_all(&exec_dir)?;

    let id = Uuid::new_v4().to_string();
    let file_path = exec_dir.join(&id);

    let mut file = tokio::fs::File::create(&file_path).await?;
    file.write_all(&data).await?;
    file.sync_all().await?;

    Ok(id)
}

/// Delete all binary artifacts stored for a specific execution.
///
/// Removes the entire `<root>/<execution_id>` subdirectory and returns the
/// number of files that were deleted. Returns `0` without error if no
/// artifacts existed for that execution.
pub async fn delete_execution_artifacts(
    config: &BinaryStorageConfig,
    execution_id: Uuid,
) -> io::Result<u64> {
    let exec_dir = config.root_dir.join(execution_id.to_string());
    if !exec_dir.exists() {
        return Ok(0);
    }

    let mut count = 0u64;
    let mut entries = tokio::fs::read_dir(&exec_dir).await?;
    while let Some(_entry) = entries.next_entry().await? {
        count += 1;
    }

    tokio::fs::remove_dir_all(&exec_dir).await?;
    Ok(count)
}

/// Delete flat binary artifacts (created via [`store_binary_to_fs`]) whose
/// modification time is older than `max_age` from now.
///
/// Skips subdirectories (which belong to the execution-scoped layout).
/// Returns the number of files deleted.
pub async fn delete_artifacts_older_than(
    config: &BinaryStorageConfig,
    max_age: std::time::Duration,
) -> io::Result<u64> {
    if !config.root_dir.exists() {
        return Ok(0);
    }

    let cutoff = std::time::SystemTime::now()
        .checked_sub(max_age)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "max_age overflow"))?;

    let mut deleted = 0u64;
    let mut entries = tokio::fs::read_dir(&config.root_dir).await?;
    while let Some(entry) = entries.next_entry().await? {
        let meta = entry.metadata().await?;
        if !meta.is_file() {
            continue;
        }
        if let Ok(modified) = meta.modified() {
            if modified < cutoff && tokio::fs::remove_file(entry.path()).await.is_ok() {
                deleted += 1;
            }
        }
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_store_and_read_binary() {
        let temp_dir = TempDir::new().unwrap();
        let config = BinaryStorageConfig::new(temp_dir.path());

        let data = bytes::Bytes::from("Hello, World!");
        let id = store_binary_to_fs(&config, data.clone()).await.unwrap();

        assert!(binary_exists(&config, &id).await);

        let read_data = read_binary_from_fs(&config, &id).await.unwrap();
        assert_eq!(read_data, data.to_vec());
    }

    #[tokio::test]
    async fn test_store_and_delete_binary() {
        let temp_dir = TempDir::new().unwrap();
        let config = BinaryStorageConfig::new(temp_dir.path());

        let data = bytes::Bytes::from("Test data");
        let id = store_binary_to_fs(&config, data).await.unwrap();

        assert!(binary_exists(&config, &id).await);

        delete_binary_from_fs(&config, &id).await.unwrap();
        assert!(!binary_exists(&config, &id).await);
    }

    #[tokio::test]
    async fn test_read_binary_chunked() {
        let temp_dir = TempDir::new().unwrap();
        let config = BinaryStorageConfig::new(temp_dir.path());

        let data =
            bytes::Bytes::from("This is a longer piece of data that will be split into chunks");
        let id = store_binary_to_fs(&config, data.clone()).await.unwrap();

        let chunks = read_binary_from_fs_chunked(&config, &id, 10).await.unwrap();

        let reconstructed: Vec<u8> = chunks.into_iter().flatten().collect();
        assert_eq!(reconstructed, data.to_vec());
    }

    #[tokio::test]
    async fn test_nonexistent_binary() {
        let temp_dir = TempDir::new().unwrap();
        let config = BinaryStorageConfig::new(temp_dir.path());

        let result = read_binary_from_fs(&config, "nonexistent").await;
        assert!(result.is_err());

        let exists = binary_exists(&config, "nonexistent").await;
        assert!(!exists);
    }

    #[tokio::test]
    async fn test_store_and_delete_execution_artifacts() {
        let temp_dir = TempDir::new().unwrap();
        let config = BinaryStorageConfig::new(temp_dir.path());
        let exec_id = Uuid::new_v4();

        let id1 = store_binary_for_execution(&config, exec_id, bytes::Bytes::from("file1"))
            .await
            .unwrap();
        let id2 = store_binary_for_execution(&config, exec_id, bytes::Bytes::from("file2"))
            .await
            .unwrap();

        let exec_dir = config.root_dir.join(exec_id.to_string());
        assert!(exec_dir.join(&id1).exists());
        assert!(exec_dir.join(&id2).exists());

        let deleted = delete_execution_artifacts(&config, exec_id).await.unwrap();
        assert_eq!(deleted, 2);
        assert!(!exec_dir.exists());
    }

    #[tokio::test]
    async fn test_delete_execution_artifacts_missing_dir_returns_zero() {
        let temp_dir = TempDir::new().unwrap();
        let config = BinaryStorageConfig::new(temp_dir.path());

        let count = delete_execution_artifacts(&config, Uuid::new_v4())
            .await
            .unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_delete_artifacts_older_than_removes_old_files() {
        let temp_dir = TempDir::new().unwrap();
        let config = BinaryStorageConfig::new(temp_dir.path());

        let id = store_binary_to_fs(&config, bytes::Bytes::from("old data"))
            .await
            .unwrap();
        assert!(binary_exists(&config, &id).await);

        // Duration::ZERO means cutoff == now; every file whose mtime < now is deleted.
        let deleted = delete_artifacts_older_than(&config, Duration::ZERO)
            .await
            .unwrap();
        assert_eq!(deleted, 1);
        assert!(!binary_exists(&config, &id).await);
    }

    #[tokio::test]
    async fn test_delete_artifacts_older_than_skips_subdirectories() {
        let temp_dir = TempDir::new().unwrap();
        let config = BinaryStorageConfig::new(temp_dir.path());
        let exec_id = Uuid::new_v4();

        // Store one execution-scoped file (in a subdirectory).
        store_binary_for_execution(&config, exec_id, bytes::Bytes::from("exec data"))
            .await
            .unwrap();

        // Age-based cleanup must not touch execution subdirectories.
        let deleted = delete_artifacts_older_than(&config, Duration::ZERO)
            .await
            .unwrap();
        assert_eq!(deleted, 0, "should not delete files inside execution subdirs");
        assert!(config.root_dir.join(exec_id.to_string()).exists());
    }
}
