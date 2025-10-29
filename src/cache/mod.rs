pub mod manager;
pub mod mock_cache;
pub mod redis_cache;
pub mod sqlite_cache;
pub mod traits;

use std::sync::Arc;

pub use traits::CacheLayer;

use bb8::RunError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CacheError {
    #[error("SQLite error: {0}")]
    SQLite(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Redis connection pool error: {0}")]
    RedisPool(#[from] RunError<redis::RedisError>),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid cache configuration: {0}")]
    InvalidConfig(String),
}

pub struct TenantCache<C: CacheLayer + Send + Sync> {
    pub tenant: String,
    pub inner: Arc<C>,
}

impl<C: CacheLayer + Send + Sync> TenantCache<C> {
    pub fn new(tenant: String, inner: Arc<C>) -> Self {
        Self { tenant, inner }
    }
}

impl<C: CacheLayer + Send + Sync> CacheLayer for TenantCache<C> {
    async fn atomic_decrement(&self, key: &str) -> Result<i64, CacheError> {
        let namespaced_key = format!("{}:{}", self.tenant, key);
        self.inner.atomic_decrement(&namespaced_key).await
    }

    async fn atomic_increment(&self, key: &str) -> Result<i64, CacheError> {
        let namespaced_key = format!("{}:{}", self.tenant, key);
        self.inner.atomic_increment(&namespaced_key).await
    }

    async fn bulk_get(&self, keys: &[&str]) -> Result<Vec<Option<String>>, CacheError> {
        let namespaced_keys: Vec<String> = keys
            .iter()
            .map(|key| format!("{}:{}", self.tenant, key))
            .collect();
        let namespaced_key_refs: Vec<&str> = namespaced_keys.iter().map(|s| s.as_str()).collect();
        self.inner.bulk_get(&namespaced_key_refs).await
    }

    async fn bulk_set(&self, items: &[(&str, &str)]) -> Result<(), CacheError> {
        let namespaced_items: Vec<(String, &str)> = items
            .iter()
            .map(|(key, value)| (format!("{}:{}", self.tenant, key), *value))
            .collect();
        let namespaced_item_refs: Vec<(&str, &str)> = namespaced_items
            .iter()
            .map(|(key, value)| (key.as_str(), *value))
            .collect();
        self.inner.bulk_set(&namespaced_item_refs).await
    }

    async fn flush(&self) -> Result<(), CacheError> {
        // drop all keys for this tenant
        unimplemented!();
    }

    async fn compare_and_swap(
        &self,
        key: &str,
        expected: &str,
        new_value: &str,
    ) -> Result<bool, CacheError> {
        let namespaced_key = format!("{}:{}", self.tenant, key);
        self.inner
            .compare_and_swap(&namespaced_key, expected, new_value)
            .await
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        let namespaced_key = format!("{}:{}", self.tenant, key);
        self.inner.delete(&namespaced_key).await
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        let namespaced_key = format!("{}:{}", self.tenant, key);
        self.inner.exists(&namespaced_key).await
    }

    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let namespaced_key = format!("{}:{}", self.tenant, key);
        self.inner.get(&namespaced_key).await
    }

    async fn set_if_absent(&self, key: &str, value: &str) -> Result<bool, CacheError> {
        let namespaced_key = format!("{}:{}", self.tenant, key);
        self.inner.set_if_absent(&namespaced_key, value).await
    }

    async fn set(&self, key: &str, value: &str) -> Result<(), CacheError> {
        let namespaced_key = format!("{}:{}", self.tenant, key);
        self.inner.set(&namespaced_key, value).await
    }

    async fn update(&self, key: &str, value: &str) -> Result<(), CacheError> {
        let namespaced_key = format!("{}:{}", self.tenant, key);
        self.inner.update(&namespaced_key, value).await
    }
}
