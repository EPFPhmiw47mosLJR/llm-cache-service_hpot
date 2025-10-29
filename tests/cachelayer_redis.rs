mod common;
// TODO: Add tests for:
// - fn atomic_decrement
// - fn atomic_increment
// - fn bulk_get
// - fn bulk_set
// - fn clear
// - fn compare_and_swap
// - fn exists
// - fn set_if_absent
// - fn update
// TODO: Add comments to each test explaining its purpose.

mod redis {
    use crate::common;
    use llm_cache_service::cache::{CacheError, CacheLayer, redis_cache::RedisCache};
    use std::{sync::Arc, time::Duration};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_cache_get_returns_stored_value()
    -> Result<(), Box<dyn std::error::Error + 'static>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 5, |b| b).await.unwrap();

        cache.set("foo", "bar").await.unwrap();
        let result = cache.get("foo").await.unwrap();
        assert_eq!(result, Some("bar".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_get_returns_none_for_missing_key()
    -> Result<(), Box<dyn std::error::Error + 'static>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 5, |b| b).await.unwrap();

        let result = cache.get("foo").await.unwrap();
        assert_eq!(result, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_key_expires_after_ttl() -> Result<(), Box<dyn std::error::Error + 'static>>
    {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 5, |b| b).await.unwrap();

        cache.set("foo", "bar").await.unwrap();
        let immediate = cache.get("foo").await.unwrap();
        assert_eq!(immediate, Some("bar".to_string()));

        sleep(Duration::from_secs(6)).await;
        let expired = cache.get("foo").await.unwrap();
        assert_eq!(expired, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_delete_removes_key() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 5, |b| b).await.unwrap();

        cache.set("foo", "bar").await.unwrap();
        cache.delete("foo").await.unwrap();
        let result = cache.get("foo").await.unwrap();
        assert_eq!(result, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_handles_multiple_keys() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 5, |b| b).await.unwrap();

        cache.set("foo", "bar").await.unwrap();
        cache.set("baz", "qux").await.unwrap();

        assert_eq!(cache.get("foo").await.unwrap(), Some("bar".to_string()));
        assert_eq!(cache.get("baz").await.unwrap(), Some("qux".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_ttl_resets_on_set() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 1, |b| b).await.unwrap();

        cache.set("foo", "bar").await.unwrap();
        sleep(Duration::from_millis(500)).await;
        cache.set("foo", "baz").await.unwrap();
        sleep(Duration::from_millis(700)).await;

        let result = cache.get("foo").await.unwrap();
        assert_eq!(result, Some("baz".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_concurrent_access_is_consistent() -> Result<(), Box<dyn std::error::Error>>
    {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 5, |b| b).await.unwrap();
        let cache = Arc::new(cache);

        let handles: Vec<_> = (0..10)
            .map(|i| {
                let cache = Arc::clone(&cache);
                tokio::spawn(async move {
                    let key = format!("key{}", i);
                    cache.set(&key, &format!("val{}", i)).await.unwrap();
                    cache.get(&key).await.unwrap()
                })
            })
            .collect();

        for (i, handle) in handles.into_iter().enumerate() {
            let result = handle.await.unwrap();
            assert_eq!(
                result,
                Some(format!("val{}", i)),
                "Mismatch for key key{}",
                i
            );
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_stores_large_values() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 5, |b| b).await.unwrap();

        let large_value = "x".repeat(10_000);
        cache.set("big", &large_value).await.unwrap();
        let result = cache.get("big").await.unwrap();
        assert_eq!(result, Some(large_value));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_overwrites_existing_key() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 5, |b| b).await.unwrap();

        cache.set("foo", "bar").await.unwrap();
        cache.set("foo", "baz").await.unwrap();
        let result = cache.get("foo").await.unwrap();
        assert_eq!(result, Some("baz".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_handles_empty_string_value() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 5, |b| b).await.unwrap();

        cache.set("empty", "").await.unwrap();
        let result = cache.get("empty").await.unwrap();
        assert_eq!(result, Some("".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_rejects_zero_ttl() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 0, |b| b).await;

        assert!(
            matches!(cache, Err(CacheError::InvalidConfig(_))),
            "Expected InvalidConfig error, got: {:?}",
            cache
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_partial_ttl_update_does_not_affect_other_keys()
    -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 1, |b| b).await.unwrap();

        cache.set("foo", "bar").await.unwrap();
        cache.set("baz", "qux").await.unwrap();
        sleep(Duration::from_millis(500)).await;
        cache.set("foo", "baz").await.unwrap();
        sleep(Duration::from_millis(700)).await;

        let result_baz = cache.get("baz").await.unwrap();
        assert_eq!(result_baz, None);
        let result_foo = cache.get("foo").await.unwrap();
        assert_eq!(result_foo, Some("baz".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_flush_clears_all_keys() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let (host, host_port, _container) = common::setup_redis_testcontainer().await?;
        let url = format!("redis://{host}:{host_port}");
        let cache = RedisCache::with_builder(&url, 5, |b| b).await.unwrap();

        cache.set("foo", "bar").await.unwrap();
        cache.set("baz", "qux").await.unwrap();
        cache.flush().await.unwrap();

        let result_foo = cache.get("foo").await.unwrap();
        assert_eq!(result_foo, None);
        let result_baz = cache.get("baz").await.unwrap();
        assert_eq!(result_baz, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_handles_connection_failure() {
        common::setup_logger("error");

        let result = RedisCache::with_builder("redis://127.0.0.1:9999", 5, |b| b).await;
        assert!(
            matches!(result, Err(CacheError::RedisPool(bb8::RunError::TimedOut))),
            "Expected RedisPool(TimedOut), got: {:?}",
            result
        );
    }
}
