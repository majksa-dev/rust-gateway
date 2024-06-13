use std::sync::Arc;

use super::{config, Datastore};
use crate::{
    gateway::{
        middleware::{Context, Middleware as TMiddleware},
        next::Next,
        Result,
    },
    http::{Request, Response},
    Error,
};
use async_trait::async_trait;
use http::StatusCode;

pub struct Middleware {
    config: config::Config,
    datastore: Box<dyn Datastore + Sync + 'static>,
}

unsafe impl Send for Middleware {}
unsafe impl Sync for Middleware {}

impl Middleware {
    pub fn new(config: config::Config, datastore: impl Datastore + Sync + 'static) -> Self {
        Self {
            config,
            datastore: Box::new(datastore),
        }
    }
}

#[async_trait]
impl TMiddleware for Middleware {
    async fn run(&self, ctx: Arc<Context>, request: Request, next: Next) -> Result<Response> {
        let ip = request
            .headers
            .get("X-Real-IP")
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
            .ok_or(Error::status(StatusCode::BAD_REQUEST))?;
        let config = self
            .config
            .config
            .get(&ctx.app_id)
            .ok_or(Error::status(StatusCode::UNAUTHORIZED))?;
        let quota = match config
            .endpoints
            .get(&ctx.endpoint_id)
            .or(config.quota.as_ref())
        {
            Some(quota) => quota,
            None => {
                return next.run(request).await;
            }
        };
        let total_key = format!("{}--{}", ctx.app_id, ctx.endpoint_id);
        let user_key = format!("{}--{}", total_key, ip);
        let rate_limit = if let Some(frequency) = quota.user.as_ref() {
            self.datastore.get_rate_limit(&user_key, frequency).await
        } else if let Some(frequency) = quota.total.as_ref() {
            self.datastore.get_rate_limit(&total_key, frequency).await
        } else {
            return next.run(request).await;
        };
        if rate_limit.remaining == 0 {
            return Err(Error::status(StatusCode::TOO_MANY_REQUESTS));
        }
        let mut response = next.run(request).await?;
        response
            .insert_header("X-RateLimit-Limit", rate_limit.limit)
            .unwrap();
        response
            .insert_header("X-RateLimit-Remaining", rate_limit.remaining)
            .unwrap();
        response
            .insert_header("X-RateLimit-Reset", rate_limit.reset)
            .unwrap();
        Ok(response)
    }
}
