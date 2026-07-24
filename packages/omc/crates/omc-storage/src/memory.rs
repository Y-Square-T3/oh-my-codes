use crate::Storage;
use async_trait::async_trait;
use omc_core::error::Result;
use std::collections::BTreeMap;
use std::sync::Mutex;

pub struct MemoryStorage {
    data: Mutex<BTreeMap<String, Vec<u8>>>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            data: Mutex::new(BTreeMap::new()),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Storage for MemoryStorage {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let data = self.data.lock().unwrap();
        Ok(data.get(key).cloned())
    }

    async fn put(&self, key: &str, value: &[u8]) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        data.insert(key.to_string(), value.to_vec());
        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut data = self.data.lock().unwrap();
        data.remove(key);
        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let data = self.data.lock().unwrap();
        let keys: Vec<String> = data
            .range(prefix.to_string()..)
            .take_while(|(k, _)| k.starts_with(prefix))
            .map(|(k, _)| k.clone())
            .collect();
        Ok(keys)
    }
}
