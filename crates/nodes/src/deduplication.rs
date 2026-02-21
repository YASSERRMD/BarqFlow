//! Deduplication Service
//!
//! Implements deduplication for workflow executions.

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Deduplication service for preventing duplicate executions
pub struct DeduplicationService {
    seen_executions: Arc<RwLock<HashSet<String>>>,
    max_cache_size: usize,
}

impl DeduplicationService {
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            seen_executions: Arc::new(RwLock::new(HashSet::new())),
            max_cache_size,
        }
    }

    pub async fn is_duplicate(&self, key: &str) -> bool {
        let seen = self.seen_executions.read().await;
        seen.contains(key)
    }

    pub async fn mark_seen(&self, key: String) {
        let mut seen = self.seen_executions.write().await;
        if seen.len() >= self.max_cache_size {
            if let Some(first) = seen.iter().next().cloned() {
                seen.remove(&first);
            }
        }
        seen.insert(key);
    }

    pub async fn clear(&self) {
        let mut seen = self.seen_executions.write().await;
        seen.clear();
    }
}

impl Default for DeduplicationService {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deduplication() {
        let service = DeduplicationService::new(10);
        
        assert!(!service.is_duplicate("exec-1").await);
        
        service.mark_seen("exec-1".to_string()).await;
        assert!(service.is_duplicate("exec-1").await);
    }

    #[tokio::test]
    async fn test_clear() {
        let service = DeduplicationService::new(10);
        service.mark_seen("exec-1".to_string()).await;
        service.clear().await;
        assert!(!service.is_duplicate("exec-1").await);
    }
}
