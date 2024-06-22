use super::{Cache, Datastore, Response};
use anyhow::{Context, Result};
use async_trait::async_trait;
use bb8_redis::{
    bb8::Pool,
    redis::{self, AsyncCommands},
    RedisConnectionManager,
};
use essentials::debug;

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
    async fn fetch_cache(&self, key: &str) -> Result<Response> {
        let mut conn = self.pool.get().await.with_context(|| {
            format!("Failed to get connection from Redis pool for key: {}", key)
        })?;
        let (expires_in, exists) = redis::pipe()
            .atomic()
            .ttl(format!("{key}:expire"))
            .exists(format!("{key}:value"))
            .query_async(&mut *conn)
            .await
            .map(|(expires_in, exists): (isize, bool)| (expires_in, exists))
            .with_context(|| format!("Failed to fetch cache for key: {}", key))?;
        debug!(key, expires_in, exists, "Fetched cache");
        let response = if expires_in < 0 {
            if exists {
                Response::Expired(
                    conn.get(format!("{key}:etag"))
                        .await
                        .map(|value: Option<String>| {
                            value
                                .and_then(|value| if value.is_empty() { None } else { Some(value) })
                        })
                        .with_context(|| "Failed to fetch etag".to_string())?,
                )
            } else {
                Response::Miss
            }
        } else {
            redis::pipe()
                .atomic()
                .get(format!("{key}:value"))
                .get(format!("{key}:headers"))
                .query_async(&mut *conn)
                .await
                .map(
                    |response: (Option<String>, Option<String>)| match response {
                        (Some(value), Some(headers)) => Response::Hit(
                            Cache {
                                response: value,
                                headers,
                            },
                            expires_in as usize,
                        ),
                        _ => Response::Miss,
                    },
                )
                .with_context(|| "Failed to fetch value and headers".to_string())?
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
        debug!(
            key = key,
            value = value,
            etag = etag,
            expires_in = expires_in,
            "Saving cache"
        );
        let mut conn = self.pool.get().await.with_context(|| {
            format!("Failed to get connection from Redis pool for key: {}", key)
        })?;
        redis::pipe()
            .atomic()
            .set_ex(format!("{key}:expire"), "", expires_in as u64)
            .ignore()
            .set(format!("{key}:etag"), etag.unwrap_or_default())
            .ignore()
            .set(format!("{key}:headers"), headers)
            .ignore()
            .set(format!("{key}:value"), value)
            .ignore()
            .query_async(&mut *conn)
            .await
            .map(|_: ()| ())
            .with_context(|| format!("Failed to save cache for key: {}", key))?;
        Ok(())
    }

    async fn refresh_cache(&self, key: &str, expires_at: usize) -> Result<Response> {
        let now = chrono::Utc::now().timestamp() as usize;
        let ttl = expires_at.saturating_sub(now);
        let mut conn = self.pool.get().await.with_context(|| {
            format!("Failed to get connection from Redis pool for key: {}", key)
        })?;
        redis::pipe()
            .atomic()
            .cmd("SET")
            .arg(format!("{key}:expire"))
            .arg(0)
            .arg("EX")
            .arg(expires_at - now)
            .arg("NX")
            .ignore()
            .get(format!("{key}:value"))
            .get(format!("{key}:headers"))
            .query_async(&mut *conn)
            .await
            .map(
                |response: (Option<String>, Option<String>)| match response {
                    (Some(value), Some(headers)) => Response::Hit(
                        Cache {
                            response: value,
                            headers,
                        },
                        ttl,
                    ),
                    _ => Response::Miss,
                },
            )
            .with_context(|| "Failed to refresh cache".to_string())
    }
}
