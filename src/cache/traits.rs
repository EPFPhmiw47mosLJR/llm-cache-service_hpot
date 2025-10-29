use crate::cache::CacheError;

pub trait CacheLayer: Send + Sync {
    fn atomic_decrement(
        &self,
        key: &str,
    ) -> impl std::future::Future<Output = Result<i64, CacheError>> + Send;
    fn atomic_increment(
        &self,
        key: &str,
    ) -> impl std::future::Future<Output = Result<i64, CacheError>> + Send;
    fn bulk_get(
        &self,
        keys: &[&str],
    ) -> impl std::future::Future<Output = Result<Vec<Option<String>>, CacheError>> + Send;
    fn bulk_set(
        &self,
        items: &[(&str, &str)],
    ) -> impl std::future::Future<Output = Result<(), CacheError>> + Send;
    fn flush(&self) -> impl std::future::Future<Output = Result<(), CacheError>> + Send;
    fn compare_and_swap(
        &self,
        key: &str,
        expected: &str,
        new_value: &str,
    ) -> impl std::future::Future<Output = Result<bool, CacheError>> + Send;
    fn delete(&self, key: &str)
    -> impl std::future::Future<Output = Result<(), CacheError>> + Send;
    fn exists(
        &self,
        key: &str,
    ) -> impl std::future::Future<Output = Result<bool, CacheError>> + Send;
    fn get(
        &self,
        key: &str,
    ) -> impl std::future::Future<Output = Result<Option<String>, CacheError>> + Send;
    fn set_if_absent(
        &self,
        key: &str,
        value: &str,
    ) -> impl std::future::Future<Output = Result<bool, CacheError>> + Send;
    fn set(
        &self,
        key: &str,
        value: &str,
    ) -> impl std::future::Future<Output = Result<(), CacheError>> + Send;
    fn update(
        &self,
        key: &str,
        value: &str,
    ) -> impl std::future::Future<Output = Result<(), CacheError>> + Send;
}
