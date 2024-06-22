use crate::utils::time::Frequency;
use anyhow::Result;
use async_trait::async_trait;

mod memory;
mod redis;

pub use memory::InMemoryDatastore;
pub use redis::RedisDatastore;

pub enum Response {
    Ok(RateLimit),
    Limited(usize),
}

#[async_trait]
pub trait Datastore {
    async fn get_rate_limit(&self, key: &str, quota: &Frequency) -> Result<Response>;
}

#[derive(Clone, Debug)]
pub struct RateLimit {
    pub limit: usize,
    pub remaining: usize,
    pub reset: usize,
}
