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

#[cfg(test)]
mod tests {
    use super::*;
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
}
