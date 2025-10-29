use bb8_redis::{
    RedisConnectionManager,
    bb8::{Builder, Pool},
    redis::{AsyncCommands, SetOptions},
};
use tracing::{debug, error, info, instrument};

use crate::cache::{CacheError, CacheLayer};

#[derive(Debug)]
pub struct RedisCache {
    pool: Pool<RedisConnectionManager>,
    ttl: u64,
}

impl RedisCache {
    #[instrument(skip(configure))]
    pub async fn with_builder<F>(url: &str, ttl: u64, configure: F) -> Result<Self, CacheError>
    where
        F: FnOnce(Builder<RedisConnectionManager>) -> Builder<RedisConnectionManager>,
    {
        if ttl == 0 {
            return Err(CacheError::InvalidConfig(
                "TTL must be greater than zero".into(),
            ));
        }

        debug!("Attempting to connect to Redis at: {}", url);

        let manager = RedisConnectionManager::new(url)?;
        let builder = configure(Pool::builder());
        let pool = builder.build(manager).await?;

        // Check connection
        let conn = pool.get().await.map_err(|e| CacheError::RedisPool(e))?;
        drop(conn);

        info!("Redis pool initialized successfully.");

        Ok(Self { pool, ttl })
    }
}

impl CacheLayer for RedisCache {
    #[instrument(skip(self))]
    async fn atomic_decrement(&self, key: &str) -> Result<i64, CacheError> {
        debug!("Attempting to ATOMIC DECREMENT key: {}", key);
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection from pool: {}", e);
            CacheError::RedisPool(e)
        })?;
        match conn.decr::<&str, i64, i64>(key, 1).await {
            Ok(new_value) => {
                debug!(
                    "Redis ATOMIC DECREMENT key {}: new value {}",
                    key, new_value
                );
                Ok(new_value)
            }
            Err(e) => {
                error!("Redis ATOMIC DECREMENT error for key {}: {}", key, e);
                Err(CacheError::Redis(e))
            }
        }
    }

    #[instrument(skip(self))]
    async fn atomic_increment(&self, key: &str) -> Result<i64, CacheError> {
        debug!("Attempting to ATOMIC INCREMENT key: {}", key);
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection from pool: {}", e);
            CacheError::RedisPool(e)
        })?;
        match conn.incr::<&str, i64, i64>(key, 1).await {
            Ok(new_value) => {
                debug!(
                    "Redis ATOMIC INCREMENT key {}: new value {}",
                    key, new_value
                );
                Ok(new_value)
            }
            Err(e) => {
                error!("Redis ATOMIC INCREMENT error for key {}: {}", key, e);
                Err(CacheError::Redis(e))
            }
        }
    }

    #[instrument(skip(self))]
    async fn bulk_get(&self, keys: &[&str]) -> Result<Vec<Option<String>>, CacheError> {
        debug!("Attempting to BULK GET keys: {:?}", keys);
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection from pool: {}", e);
            CacheError::RedisPool(e)
        })?;
        match conn.get::<&[&str], Vec<Option<String>>>(keys).await {
            Ok(values) => {
                debug!("Redis BULK GET successful for keys: {:?}", keys);
                Ok(values)
            }
            Err(e) => {
                error!("Redis BULK GET error for keys {:?}: {}", keys, e);
                Err(CacheError::Redis(e))
            }
        }
    }

    #[instrument(skip(self))]
    async fn bulk_set(&self, items: &[(&str, &str)]) -> Result<(), CacheError> {
        debug!("Attempting to BULK SET items: {:?}", items);
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection from pool: {}", e);
            CacheError::RedisPool(e)
        })?;

        for (key, value) in items {
            match conn.set_ex::<&str, &str, ()>(key, value, self.ttl).await {
                Ok(_) => {
                    debug!("Redis BULK SET key {} with TTL {}", key, self.ttl);
                }
                Err(e) => {
                    error!("Redis BULK SET error for key {}: {}", key, e);
                    return Err(CacheError::Redis(e));
                }
            }
        }

        Ok(())
    }

    #[instrument(skip(self))]
    async fn flush(&self) -> Result<(), CacheError> {
        debug!("Attempting to CLEAR all keys in Redis cache");
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection from pool: {}", e);
            CacheError::RedisPool(e)
        })?;
        match conn.flushdb::<()>().await {
            Ok(_) => {
                debug!("Redis CLEAR successful");
                Ok(())
            }
            Err(e) => {
                error!("Redis CLEAR error: {}", e);
                Err(CacheError::Redis(e))
            }
        }
    }

    #[instrument(skip(self))]
    async fn compare_and_swap(
        &self,
        key: &str,
        old_value: &str,
        new_value: &str,
    ) -> Result<bool, CacheError> {
        unimplemented!()
    }

    #[instrument(skip(self))]
    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        debug!("Attempting to DELETE key: {}", key);
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection from pool: {}", e);
            CacheError::RedisPool(e)
        })?;
        match conn.del::<&str, ()>(key).await {
            Ok(_) => {
                debug!("Redis DELETE key {}", key);
                Ok(())
            }
            Err(e) => {
                error!("Redis DELETE error for key {}: {}", key, e);
                Err(CacheError::Redis(e))
            }
        }
    }

    #[instrument(skip(self))]
    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        debug!("Attempting to check EXISTS for key: {}", key);
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection from pool: {}", e);
            CacheError::RedisPool(e)
        })?;
        match conn.exists(key).await {
            Ok(exists) => {
                debug!("Redis EXISTS check for key {}: {}", key, exists);
                Ok(exists)
            }
            Err(e) => {
                error!("Redis EXISTS error for key {}: {}", key, e);
                Err(CacheError::Redis(e))
            }
        }
    }

    #[instrument(skip(self))]
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        debug!("Attempting to GET key: {}", key);
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get Redis connection from pool: {}", e);
                return Err(CacheError::RedisPool(e));
            }
        };

        match conn.get(key).await {
            Ok(Some(value)) => {
                debug!("Redis Cache HIT for key: {}", key);
                Ok(Some(value))
            }
            Ok(None) => {
                debug!("Redis Cache MISS for key: {}", key);
                Ok(None)
            }
            Err(e) => {
                error!("Redis GET error for key {}: {}", key, e);
                Err(CacheError::Redis(e))
            }
        }
    }

    #[instrument(skip(self, value), fields(cache_ttl = self.ttl))]
    async fn set_if_absent(&self, key: &str, value: &str) -> Result<bool, CacheError> {
        debug!("Attempting to SETNX key: {}", key);
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection from pool: {}", e);
            CacheError::RedisPool(e)
        })?;
        let opts = SetOptions::default()
            .conditional_set(redis::ExistenceCheck::NX)
            .with_expiration(redis::SetExpiry::EX(self.ttl));
        match conn.set_options::<&str, &str, bool>(key, value, opts).await {
            Ok(was_set) => {
                if was_set {
                    debug!("Redis SETNX succeeded for key: {}", key);
                } else {
                    debug!("Redis SETNX failed (key exists) for key: {}", key);
                }
                Ok(was_set)
            }
            Err(e) => {
                error!("Redis SETNX error for key {}: {}", key, e);
                Err(CacheError::Redis(e))
            }
        }
    }

    #[instrument(skip(self, value), fields(cache_ttl = self.ttl))]
    async fn set(&self, key: &str, value: &str) -> Result<(), CacheError> {
        debug!("Attempting to SET key with value length: {}", value.len());
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection from pool: {}", e);
            CacheError::RedisPool(e)
        })?;
        match conn.set_ex::<&str, &str, ()>(key, value, self.ttl).await {
            Ok(_) => {
                debug!("Redis SET key {} with TTL {}", key, self.ttl);
                Ok(())
            }
            Err(e) => {
                error!("Redis SET error for key {}: {}", key, e);
                Err(CacheError::Redis(e))
            }
        }
    }

    #[instrument(skip(self, value), fields(cache_ttl = self.ttl))]
    async fn update(&self, key: &str, value: &str) -> Result<(), CacheError> {
        debug!("Attempting to UPDATE key: {}", key);
        let mut conn = self.pool.get().await.map_err(|e| {
            error!("Failed to get Redis connection from pool: {}", e);
            CacheError::RedisPool(e)
        })?;

        let opts = SetOptions::default()
            .conditional_set(redis::ExistenceCheck::XX)
            .with_expiration(redis::SetExpiry::EX(self.ttl));
        match conn.set_options::<&str, &str, ()>(key, value, opts).await {
            Ok(_) => {
                debug!("Redis UPDATE key {} with TTL {}", key, self.ttl);
                Ok(())
            }
            Err(e) => {
                error!("Redis UPDATE error for key {}: {}", key, e);
                Err(CacheError::Redis(e))
            }
        }
    }
}
