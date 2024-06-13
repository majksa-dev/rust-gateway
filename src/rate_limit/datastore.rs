use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use bb8_redis::{bb8::Pool, redis, RedisConnectionManager};
use essentials::error;

use crate::utils::time::{Frequency, TimeUnit};

#[async_trait]
pub trait Datastore {
    async fn get_rate_limit(&self, key: &str, quota: &Frequency) -> RateLimit;
}

#[derive(Clone, Debug)]
pub struct RateLimit {
    pub limit: usize,
    pub remaining: usize,
    pub reset: usize,
}

#[derive(Debug, Default)]
pub struct InMemoryDatastore {
    data: Arc<Mutex<HashMap<String, RateLimit>>>,
}

unsafe impl Send for InMemoryDatastore {}

#[async_trait]
impl Datastore for InMemoryDatastore {
    async fn get_rate_limit(&self, key: &str, quota: &Frequency) -> RateLimit {
        let now = chrono::Utc::now().timestamp() as usize;
        let mut data = match __self.data.lock() {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to lock data: {}", e);
                return RateLimit {
                    limit: quota.amount,
                    remaining: quota.amount,
                    reset: 0,
                };
            }
        };
        if let Some(rate_limit) = data.get_mut(key) {
            if rate_limit.reset < now {
                rate_limit.remaining = quota.amount;
                rate_limit.reset = now + quota.interval.convert(TimeUnit::Seconds).amount;
            } else if rate_limit.remaining > 0 {
                rate_limit.remaining -= 1;
            }
            rate_limit.clone()
        } else {
            let rate_limit = RateLimit {
                limit: quota.amount,
                remaining: quota.amount,
                reset: now + quota.interval.convert(TimeUnit::Seconds).amount,
            };
            data.insert(key.to_string(), rate_limit.clone());
            rate_limit
        }
    }
}

impl InMemoryDatastore {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct RedisDatastore {
    pool: Pool<RedisConnectionManager>,
}

impl RedisDatastore {
    pub fn new(pool: Pool<RedisConnectionManager>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Datastore for RedisDatastore {
    async fn get_rate_limit(&self, key: &str, quota: &Frequency) -> RateLimit {
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get connection from pool: {}", e);
                return RateLimit {
                    limit: quota.amount,
                    remaining: quota.amount,
                    reset: 0,
                };
            }
        };
        match redis::pipe()
            .atomic()
            .cmd("SET")
            .arg(key)
            .arg(0)
            .arg("EX")
            .arg(quota.interval.convert(TimeUnit::Seconds).amount)
            .arg("NX")
            .ignore()
            .cmd("INCR")
            .arg(key)
            .cmd("TTL")
            .arg(key)
            .query_async(&mut *conn)
            .await
            .map(|(count, ttl): (usize, usize)| (count, ttl))
        {
            Ok((count, ttl)) => RateLimit {
                limit: quota.amount,
                remaining: if count > quota.amount {
                    0
                } else {
                    quota.amount - count
                },
                reset: chrono::Utc::now()
                    .timestamp()
                    .checked_add(ttl as i64)
                    .unwrap_or(0) as usize,
            },
            Err(e) => {
                error!("Failed to execute pipeline: {}", e);
                RateLimit {
                    limit: quota.amount,
                    remaining: quota.amount,
                    reset: 0,
                }
            }
        }
    }
}
