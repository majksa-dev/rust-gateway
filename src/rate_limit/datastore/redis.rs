use anyhow::{Context, Result};
use async_trait::async_trait;
use bb8_redis::{bb8::Pool, redis, RedisConnectionManager};

use crate::time::{Frequency, TimeUnit};

use super::{Datastore, RateLimit, Response};

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
    async fn get_rate_limit(&self, key: &str, quota: &Frequency) -> Result<Response> {
        let now = chrono::Utc::now().timestamp() as usize;
        let mut conn = self.pool.get().await.with_context(|| {
            format!("Failed to get connection from Redis pool for key: {}", key)
        })?;
        redis::pipe()
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
            .map(|(count, ttl): (usize, usize)| {
                if count > quota.amount {
                    Response::Limited(now + ttl)
                } else {
                    Response::Ok(RateLimit {
                        limit: quota.amount,
                        remaining: quota.amount - count,
                        reset: now + ttl,
                    })
                }
            })
            .with_context(|| format!("Failed to get rate limit for key: {}", key))
    }
}
