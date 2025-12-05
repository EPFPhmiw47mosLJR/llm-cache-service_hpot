use std::sync::Arc;

use tracing::{debug, instrument};

use crate::cache::{CacheError, CacheLayer};

pub struct CacheManager<L1: CacheLayer, L2: CacheLayer> {
    l1: Arc<L1>,
    l2: Arc<L2>,
}

impl<L1: CacheLayer, L2: CacheLayer> CacheManager<L1, L2> {
    #[instrument(skip(l1, l2))]
    pub fn new(l1: Arc<L1>, l2: Arc<L2>) -> Self {
        Self { l1, l2 }
    }

    #[instrument(skip(self), fields(key = %key))]
    pub async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        let v = match self.l1.get(key).await? {
            Some(v) => {
                debug!("CacheManager: L1 cache hit for key '{}'", key);
                Some(v)
            }
            None => match self.l2.get(key).await? {
                Some(v) => {
                    debug!(
                        "CacheManager: L2 cache hit for key '{}', warming L1 cache",
                        key
                    );
                    let _ = self.l1.set(key, &v).await;
                    Some(v)
                }
                None => {
                    debug!("CacheManager: Cache miss for key '{}'", key);
                    None
                }
            },
        };
        Ok(v)
    }

    #[instrument(skip(self), fields(key = %key))]
    pub async fn set(&self, key: &str, value: &str) -> Result<(), CacheError> {
        debug!("CacheManager: Setting key '{}' in both cache layers", key);
        let _ = self.l1.set(key, value).await?;
        let _ = self.l2.set(key, value).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::mock_cache::MockCache;

    #[tokio::test]
    async fn test_get_from_l1_hits_first() {
        let l1 = Arc::new(MockCache::default());
        let l2 = Arc::new(MockCache::default());
        let manager = CacheManager::new(l1.clone(), l2.clone());

        l1.set("foo", "bar").await.unwrap();

        let result = manager.get("foo").await.unwrap();
        assert_eq!(result, Some("bar".into()));

        // Assert L1 was queried once, L2 not touched
        assert_eq!(*l1.get_calls.lock().unwrap(), 1);
        assert_eq!(*l2.get_calls.lock().unwrap(), 0);
    }

    #[tokio::test]
    async fn test_get_from_l2_warms_l1() {
        let l1 = Arc::new(MockCache::default());
        let l2 = Arc::new(MockCache::default());
        let manager = CacheManager::new(l1.clone(), l2.clone());

        l2.set("foo", "bar").await.unwrap();

        // L1 miss, L2 hit
        let result = manager.get("foo").await.unwrap();
        assert_eq!(result, Some("bar".into()));

        // Assert: L1 and L2 were queried once
        assert_eq!(*l1.get_calls.lock().unwrap(), 1);
        assert_eq!(*l2.get_calls.lock().unwrap(), 1);

        // Assert: L1 was warmed up
        assert_eq!(l1.get("foo").await.unwrap(), Some("bar".into()));
    }

    #[tokio::test]
    async fn test_get_cache_miss_in_both() {
        let l1 = Arc::new(MockCache::default());
        let l2 = Arc::new(MockCache::default());
        let manager = CacheManager::new(l1.clone(), l2.clone());

        let result = manager.get("missing").await.unwrap();
        assert_eq!(result, None);

        assert_eq!(*l1.get_calls.lock().unwrap(), 1);
        assert_eq!(*l2.get_calls.lock().unwrap(), 1);
    }

    #[tokio::test]
    async fn test_set_writes_to_both_layers() {
        let l1 = Arc::new(MockCache::default());
        let l2 = Arc::new(MockCache::default());
        let manager = CacheManager::new(l1.clone(), l2.clone());

        manager.set("foo", "bar").await.unwrap();

        assert_eq!(l1.get("foo").await.unwrap(), Some("bar".into()));
        assert_eq!(l2.get("foo").await.unwrap(), Some("bar".into()));
    }
}
