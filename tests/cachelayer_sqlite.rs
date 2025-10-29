mod common;

mod sqlite {
    use std::sync::Arc;

    use crate::common;
    use llm_cache_service::cache::{CacheError, CacheLayer, sqlite_cache::SqliteCache};

    #[tokio::test]
    async fn test_cache_get_returns_stored_value()
    -> Result<(), Box<dyn std::error::Error + 'static>> {
        common::setup_logger("error");
        let cache = SqliteCache::with_builder(":memory:", 2, |b| b)
            .await
            .unwrap();

        cache.set("foo", "bar").await.unwrap();
        let result = cache.get("foo").await.unwrap();
        assert_eq!(result, Some("bar".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_get_returns_none_for_missing_key()
    -> Result<(), Box<dyn std::error::Error + 'static>> {
        common::setup_logger("error");
        let cache = SqliteCache::with_builder(":memory:", 2, |b| b)
            .await
            .unwrap();

        let result = cache.get("foo").await.unwrap();
        assert_eq!(result, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_delete_removes_key() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let cache = SqliteCache::with_builder(":memory:", 2, |b| b)
            .await
            .unwrap();

        cache.set("foo", "bar").await.unwrap();
        let immediate = cache.get("foo").await.unwrap();
        assert_eq!(immediate, Some("bar".to_string()));

        cache.delete("foo").await.unwrap();
        let deleted = cache.get("foo").await.unwrap();
        assert_eq!(deleted, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_handles_multiple_keys() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let cache = SqliteCache::with_builder(":memory:", 2, |b| b)
            .await
            .unwrap();

        cache.set("foo", "bar").await.unwrap();
        cache.set("baz", "qux").await.unwrap();

        let result_foo = cache.get("foo").await.unwrap();
        let result_baz = cache.get("baz").await.unwrap();

        assert_eq!(result_foo, Some("bar".to_string()));
        assert_eq!(result_baz, Some("qux".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_concurrent_access_is_consistent() -> Result<(), Box<dyn std::error::Error>>
    {
        common::setup_logger("error");
        let cache = SqliteCache::with_builder(":memory:", 1, |b| b)
            .await
            .unwrap();
        let cache = std::sync::Arc::new(cache);

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
        let cache = SqliteCache::with_builder(":memory:", 1, |b| b)
            .await
            .unwrap();

        let large_value = "x".repeat(10_000);
        cache.set("big", &large_value).await.unwrap();
        let result = cache.get("big").await.unwrap();
        assert_eq!(result, Some(large_value));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_overwrites_existing_key() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let cache = SqliteCache::with_builder(":memory:", 1, |b| b)
            .await
            .unwrap();

        cache.set("foo", "bar").await.unwrap();
        cache.set("foo", "baz").await.unwrap();
        let result = cache.get("foo").await.unwrap();
        assert_eq!(result, Some("baz".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_handles_empty_string_value() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let cache = SqliteCache::with_builder(":memory:", 1, |b| b)
            .await
            .unwrap();

        cache.set("empty", "").await.unwrap();
        let result = cache.get("empty").await.unwrap();
        assert_eq!(result, Some("".to_string()));

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_flush_clears_all_keys() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let cache = SqliteCache::with_builder(":memory:", 1, |b| b)
            .await
            .unwrap();

        cache.set("foo", "bar").await.unwrap();
        cache.set("baz", "qux").await.unwrap();

        cache.flush().await.unwrap();

        let result_foo = cache.get("foo").await.unwrap();
        let result_baz = cache.get("baz").await.unwrap();

        assert_eq!(result_foo, None);
        assert_eq!(result_baz, None);

        Ok(())
    }

    #[tokio::test]
    async fn test_cache_handles_connection_failure() -> Result<(), Box<dyn std::error::Error>> {
        common::setup_logger("error");
        let cache = SqliteCache::with_builder("/invalid/path/to/db.sqlite", 5, |b| b).await;

        assert!(
            matches!(cache, Err(CacheError::SQLite(_))),
            "Expected SQLite error, got: {:?}",
            cache
        );

        Ok(())
    }
}
