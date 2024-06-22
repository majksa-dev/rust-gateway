use crate::utils::time::{Frequency, TimeUnit};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use super::{Datastore, RateLimit, Response};

#[derive(Debug, Default)]
pub struct InMemoryDatastore {
    data: Arc<Mutex<HashMap<String, RateLimit>>>,
}

unsafe impl Send for InMemoryDatastore {}

#[async_trait]
impl Datastore for InMemoryDatastore {
    async fn get_rate_limit(&self, key: &str, quota: &Frequency) -> Result<Response> {
        let now = chrono::Utc::now().timestamp() as usize;
        let mut data = self
            .data
            .lock()
            .map_err(|_| anyhow!("Failed to acquire lock for key: {}", key))?;
        let rate_limit = if let Some(rate_limit) = data.get_mut(key) {
            if rate_limit.reset < now {
                rate_limit.remaining = quota.amount - 1;
                rate_limit.reset = now + quota.interval.convert(TimeUnit::Seconds).amount;
            } else if rate_limit.remaining > 0 {
                rate_limit.remaining -= 1;
            } else {
                return Ok(Response::Limited(rate_limit.reset));
            }
            rate_limit.clone()
        } else {
            let rate_limit = RateLimit {
                limit: quota.amount,
                remaining: quota.amount - 1,
                reset: now + quota.interval.convert(TimeUnit::Seconds).amount,
            };
            data.insert(key.to_string(), rate_limit.clone());
            rate_limit
        };
        Ok(Response::Ok(rate_limit))
    }
}

impl InMemoryDatastore {
    pub fn new() -> Self {
        Self::default()
    }
}
