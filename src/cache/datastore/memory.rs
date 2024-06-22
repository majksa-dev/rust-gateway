use super::{Cache, Datastore, Response};
use anyhow::Result;
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Debug, Default)]
pub struct InMemoryDatastore {
    data: Arc<Mutex<HashMap<String, InMemoryValue>>>,
}

unsafe impl Send for InMemoryDatastore {}

#[derive(Debug)]
struct InMemoryValue {
    value: String,
    headers: String,
    etag: Option<String>,
    expiry: usize,
}

impl From<&InMemoryValue> for Cache {
    fn from(cache: &InMemoryValue) -> Self {
        Self {
            response: cache.value.clone(),
            headers: cache.headers.clone(),
        }
    }
}

impl From<&mut InMemoryValue> for Cache {
    fn from(cache: &mut InMemoryValue) -> Self {
        Self {
            response: cache.value.clone(),
            headers: cache.headers.clone(),
        }
    }
}

#[async_trait]
impl Datastore for InMemoryDatastore {
    async fn fetch_cache(&self, key: &str) -> Result<Response> {
        let data = self
            .data
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock data for key: {}", key))?;
        let now = chrono::Utc::now().timestamp() as usize;
        let response = if let Some(cache) = data.get(key) {
            let expires_in = cache.expiry.saturating_sub(now);
            if expires_in > 0 {
                Response::Hit(cache.into(), expires_in)
            } else {
                Response::Expired(cache.etag.clone())
            }
        } else {
            Response::Miss
        };
        Ok(response)
    }

    async fn save_cache(
        &self,
        key: &str,
        value: String,
        headers: String,
        etag: Option<String>,
        expires_in: usize,
    ) -> Result<()> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock data for key: {}", key))?;
        data.insert(
            key.to_string(),
            InMemoryValue {
                value,
                headers,
                expiry: chrono::Utc::now().timestamp() as usize + expires_in,
                etag,
            },
        );
        Ok(())
    }

    async fn refresh_cache(&self, key: &str, expires_at: usize) -> Result<Response> {
        let mut data = self
            .data
            .lock()
            .map_err(|_| anyhow::anyhow!("Failed to lock data for key: {}", key))?;
        let now = chrono::Utc::now().timestamp() as usize;
        let response = if let Some(cache) = data.get_mut(key) {
            cache.expiry = expires_at;
            let expires_in = cache.expiry.saturating_sub(now);
            if expires_in > 0 {
                Response::Hit(cache.into(), expires_in)
            } else {
                Response::Expired(cache.etag.clone())
            }
        } else {
            Response::Miss
        };
        Ok(response)
    }
}
