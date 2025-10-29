mod common;

mod cachemanager_redis_sqlite {
    use std::sync::Arc;

    use llm_cache_service::cache::{
        CacheLayer, manager::CacheManager, redis_cache::RedisCache, sqlite_cache::SqliteCache,
    };

    use crate::common;

    #[tokio::test]
    async fn cache_manager_reads_and_writes_across_layers()
    -> Result<(), Box<dyn std::error::Error + 'static>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let redis_cache = Arc::new(RedisCache::with_builder(&url, 3600, |b| b).await.unwrap());
        let sqlite_cache = Arc::new(
            SqliteCache::with_builder(":memory:", 3600, |b| b)
                .await
                .unwrap(),
        );
        let cache_manager = CacheManager::new(redis_cache.clone(), sqlite_cache.clone());

        redis_cache.flush().await?;
        sqlite_cache.flush().await?;

        cache_manager.set("user:1", "Arthur Dent").await?;

        let redis_val = redis_cache.get("user:1").await?;
        let sqlite_val = sqlite_cache.get("user:1").await?;
        assert_eq!(redis_val, Some("Arthur Dent".into()));
        assert_eq!(sqlite_val, Some("Arthur Dent".into()));

        let v = cache_manager.get("user:1").await?;
        assert_eq!(v, Some("Arthur Dent".to_string()));

        redis_cache.flush().await?;

        assert_eq!(redis_cache.get("user:1").await?, None);
        assert_eq!(
            sqlite_cache.get("user:1").await?,
            Some("Arthur Dent".into())
        );

        let v = cache_manager.get("user:1").await?;
        assert_eq!(v, Some("Arthur Dent".into()));

        let redis_val = redis_cache.get("user:1").await?;
        assert_eq!(redis_val, Some("Arthur Dent".into()));

        Ok(())
    }

    #[tokio::test]
    async fn cache_manager_cache_miss_propagates()
    -> Result<(), Box<dyn std::error::Error + 'static>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let redis_cache = Arc::new(RedisCache::with_builder(&url, 3600, |b| b).await.unwrap());
        let sqlite_cache = Arc::new(
            SqliteCache::with_builder(":memory:", 3600, |b| b)
                .await
                .unwrap(),
        );
        let cache_manager = CacheManager::new(redis_cache.clone(), sqlite_cache.clone());

        redis_cache.flush().await?;
        sqlite_cache.flush().await?;

        let result = cache_manager.get("nonexistent:key").await?;
        assert_eq!(result, None);

        Ok(())
    }

    #[tokio::test]
    async fn cache_manager_set_overwrites_both_layers()
    -> Result<(), Box<dyn std::error::Error + 'static>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let redis_cache = Arc::new(RedisCache::with_builder(&url, 3600, |b| b).await.unwrap());
        let sqlite_cache = Arc::new(
            SqliteCache::with_builder(":memory:", 3600, |b| b)
                .await
                .unwrap(),
        );
        let cache_manager = CacheManager::new(redis_cache.clone(), sqlite_cache.clone());

        redis_cache.flush().await?;
        sqlite_cache.flush().await?;

        cache_manager.set("user:2", "Ford Prefect").await?;
        assert_eq!(
            cache_manager.get("user:2").await?,
            Some("Ford Prefect".into())
        );

        cache_manager.set("user:2", "Trillian").await?;

        let redis_val = redis_cache.get("user:2").await?;
        let sqlite_val = sqlite_cache.get("user:2").await?;

        assert_eq!(redis_val, Some("Trillian".into()));
        assert_eq!(sqlite_val, Some("Trillian".into()));

        Ok(())
    }
}
