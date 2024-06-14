use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use bb8_redis::{bb8::Pool, redis, RedisConnectionManager};
use essentials::error;

use crate::utils::time::{Frequency, TimeUnit};

pub type Result = std::result::Result<RateLimit, usize>;

#[async_trait]
pub trait Datastore {
    async fn get_rate_limit(&self, key: &str, quota: &Frequency) -> Result;
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
    async fn get_rate_limit(&self, key: &str, quota: &Frequency) -> Result {
        let now = chrono::Utc::now().timestamp() as usize;
        let mut data = match __self.data.lock() {
            Ok(data) => data,
            Err(e) => {
                error!("Failed to lock data: {}", e);
                return Ok(RateLimit {
                    limit: quota.amount,
                    remaining: quota.amount,
                    reset: 0,
                });
            }
        };
        if let Some(rate_limit) = data.get_mut(key) {
            if rate_limit.reset < now {
                rate_limit.remaining = quota.amount - 1;
                rate_limit.reset = now + quota.interval.convert(TimeUnit::Seconds).amount;
            } else if rate_limit.remaining > 0 {
                rate_limit.remaining -= 1;
            } else {
                return Err(rate_limit.reset);
            }
            Ok(rate_limit.clone())
        } else {
            let rate_limit = RateLimit {
                limit: quota.amount,
                remaining: quota.amount - 1,
                reset: now + quota.interval.convert(TimeUnit::Seconds).amount,
            };
            data.insert(key.to_string(), rate_limit.clone());
            Ok(rate_limit)
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
    async fn get_rate_limit(&self, key: &str, quota: &Frequency) -> Result {
        let now = chrono::Utc::now().timestamp() as usize;
        let mut conn = match self.pool.get().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get connection from pool: {}", e);
                return Ok(RateLimit {
                    limit: quota.amount,
                    remaining: quota.amount - 1,
                    reset: 0,
                });
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
            Ok((count, ttl)) => {
                if count > quota.amount {
                    Err(now + ttl)
                } else {
                    Ok(RateLimit {
                        limit: quota.amount,
                        remaining: quota.amount - count,
                        reset: now + ttl,
                    })
                }
            }
            Err(e) => {
                error!("Failed to execute pipeline: {}", e);
                Ok(RateLimit {
                    limit: quota.amount,
                    remaining: quota.amount - 1,
                    reset: 0,
                })
            }
        }
    }
}
