use anyhow::Result;
use async_trait::async_trait;

pub use memory::InMemoryDatastore;
pub use redis::RedisDatastore;

mod memory;
mod redis;

pub enum Response {
    Hit(Cache, usize),
    Expired(Option<String>),
    Miss,
}

#[async_trait]
pub trait Datastore {
    async fn fetch_cache(&self, key: &str) -> Result<Response>;

    async fn save_cache(
        &self,
        key: &str,
        value: String,
        headers: String,
        etag: Option<String>,
        expires_in: usize,
    ) -> Result<()>;

    async fn refresh_cache(&self, key: &str, expires_at: usize) -> Result<Response>;
}

#[derive(Clone, Debug)]
pub struct Cache {
    pub response: String,
    pub headers: String,
}
