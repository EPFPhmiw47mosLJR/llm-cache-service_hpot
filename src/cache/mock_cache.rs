use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::cache::{CacheLayer, CacheError};

#[derive(Debug, Clone, Default)]
pub struct MockCache {
    pub data: Arc<Mutex<HashMap<String, String>>>,
    pub get_calls: Arc<Mutex<usize>>,
    pub set_calls: Arc<Mutex<usize>>,
}

impl CacheLayer for MockCache {
    async fn get(&self, key: &str) -> Result<Option<String>, CacheError> {
        *self.get_calls.lock().unwrap() += 1;
        Ok(self.data.lock().unwrap().get(key).cloned())
    }

    async fn set(&self, key: &str, value: &str) -> Result<(), CacheError> {
        *self.set_calls.lock().unwrap() += 1;
        self.data
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    async fn atomic_decrement(&self, key: &str) -> Result<i64, CacheError> {
        unimplemented!()
    }

    async fn atomic_increment(&self, key: &str) -> Result<i64, CacheError> {
        unimplemented!()
    }

    async fn bulk_get(&self, keys: &[&str]) -> Result<Vec<Option<String>>, CacheError> {
        unimplemented!()
    }

    async fn bulk_set(&self, items: &[(&str, &str)]) -> Result<(), CacheError> {
        unimplemented!()
    }

    async fn flush(&self) -> Result<(), CacheError> {
        unimplemented!()
    }

    async fn compare_and_swap(
        &self,
        key: &str,
        expected: &str,
        new_value: &str,
    ) -> Result<bool, CacheError> {
        unimplemented!()
    }

    async fn delete(&self, key: &str) -> Result<(), CacheError> {
        unimplemented!()
    }

    async fn exists(&self, key: &str) -> Result<bool, CacheError> {
        unimplemented!()
    }

    async fn set_if_absent(&self, key: &str, value: &str) -> Result<bool, CacheError> {
        unimplemented!()
    }

    async fn update(&self, key: &str, value: &str) -> Result<(), CacheError> {
        unimplemented!()
    }
}
