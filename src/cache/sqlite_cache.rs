use sqlx::{
    Pool, Sqlite,
    sqlite::{SqliteConnectOptions, SqlitePoolOptions},
};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{debug, error, info, instrument};

use crate::cache::{CacheError, CacheLayer};

// TODO: Implement:
// - fn atomic_decrement
// - fn atomic_increment
// - fn bulk_get
// - fn bulk_set
// - fn clear
// - fn compare_and_swap
// - fn exists
// - fn set_if_absent
// - fn update

#[derive(Debug)]
pub struct SqliteCache {
    pool: Pool<Sqlite>,
    ttl: u64,
}

#[derive(Debug, sqlx::FromRow)]
struct CacheEntry {
    key: String,
    value: String,
    expires_at: i64,
}

impl SqliteCache {
    #[instrument(skip(configure))]
    pub async fn with_builder<F>(url: &str, ttl: u64, configure: F) -> Result<Self, CacheError>
    where
        F: FnOnce(SqlitePoolOptions) -> SqlitePoolOptions,
    {
        debug!("Attempting to connect to SQLite at: {}", url);

        let options = SqliteConnectOptions::from_str(url)?.create_if_missing(true);

        let builder = configure(SqlitePoolOptions::new());
        let pool = builder.connect_with(options).await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS cache (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                expires_at INTEGER NOT NULL
            );
            "#,
        )
        .execute(&pool)
        .await?;

        info!("SQLite pool initialized successfully.");

        Ok(Self { pool, ttl })
    }

    fn now_secs() -> i64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs() as i64
    }
}

impl CacheLayer for SqliteCache {
    #[instrument(skip(self))]
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let now = Self::now_secs();

        match sqlx::query_as!(
            CacheEntry,
            r#"
            SELECT
                key as "key!",
                value as "value!",
                expires_at as "expires_at!"
            FROM cache
            WHERE key = ? AND expires_at > ?
            "#,
            key,
            now
        )
        .fetch_one(&self.pool)
        .await
        {
            Ok(entry) => {
                debug!("SQLite Cache HIT for key: {}", key);
                Ok(Some(entry.value))
            }
            Err(sqlx::Error::RowNotFound) => {
                debug!("SQLite Cache MISS for key: {}", key);
                Ok(None)},
            Err(e) => {
                error!("SQLite GET error for key {}: {}", key, e);
                Err(CacheError::SQLite(e))
            }
        }
    }

    #[instrument(skip(self, value))]
    async fn set(&self, key: &str, value: &str) -> Result<(), CacheError> {
        let now = Self::now_secs();
        let expires_at = now + self.ttl as i64;

        if let Err(e) = sqlx::query!(
            r#"
            INSERT INTO cache (key, value, expires_at)
            VALUES (?, ?, ?)
            ON CONFLICT(key) DO UPDATE SET
                value = excluded.value,
                expires_at = excluded.expires_at;
            "#,
            key,
            value,
            expires_at
        )
        .execute(&self.pool)
        .await
        {
            error!("SQLite SET error for key {}: {}", key, e);
            Err(CacheError::SQLite(e))
        } else {
            debug!("SQLite SET key {} with TTL {}", key, self.ttl);
            Ok(())
        }
    }

    #[instrument(skip(self))]
    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        if let Err(e) = sqlx::query!(
            r#"
            DELETE FROM cache WHERE key = ?;
            "#,
            key
        )
        .execute(&self.pool)
        .await
        {
            error!("SQLite DELETE error for key {}: {}", key, e);
            Err(CacheError::SQLite(e))
        } else {
            debug!("SQLite DELETE key {}", key);
            Ok(())
        }
    }

    #[instrument(skip(self))]
    async fn atomic_decrement(&self, key: &str) -> Result<i64, CacheError> {
        unimplemented!()
    }

    #[instrument(skip(self))]
    async fn atomic_increment(&self, key: &str) -> Result<i64, CacheError> {
        unimplemented!()
    }

    #[instrument(skip(self))]
    async fn bulk_get(&self, keys: &[&str]) -> Result<Vec<Option<String>>, CacheError> {
        unimplemented!()
    }

    #[instrument(skip(self))]
    async fn bulk_set(&self, items: &[(&str, &str)]) -> Result<(), CacheError> {
        unimplemented!()
    }

    #[instrument(skip(self))]
    async fn flush(&self) -> Result<(), CacheError> {
        debug!("Flushing all entries from SQLite cache");
        if let Err(e) = sqlx::query!(
            r#"
            DELETE FROM cache;
            "#
        )
        .execute(&self.pool)
        .await
        {
            error!("SQLite FLUSH error: {}", e);
            Err(CacheError::SQLite(e))
        } else {
            debug!("SQLite FLUSH successful");
            Ok(())
        }
    }

    #[instrument(skip(self))]
    async fn compare_and_swap(
        &self,
        key: &str,
        expected: &str,
        new_value: &str,
    ) -> Result<bool, CacheError> {
        unimplemented!()
    }

    #[instrument(skip(self))]
    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        unimplemented!()
    }

    #[instrument(skip(self))]
    async fn set_if_absent(&self, key: &str, value: &str) -> Result<bool, CacheError> {
        unimplemented!()
    }

    #[instrument(skip(self))]
    async fn update(&self, key: &str, value: &str) -> Result<(), CacheError> {
        unimplemented!()
    }
}
