use super::{config, datastore, Datastore};
use crate::{
    gateway::{
        middleware::{Context, Middleware as TMiddleware},
        next::Next,
        Result,
    },
    http::{HeaderMapExt, Request, Response},
};
use anyhow::anyhow;
use async_trait::async_trait;
use essentials::warn;
use http::{header, StatusCode};

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

    fn too_many_requests(reset: usize) -> Result<Response> {
        let mut response = Response::new(StatusCode::TOO_MANY_REQUESTS);
        response
            .insert_header(
                header::RETRY_AFTER,
                reset.saturating_sub(chrono::Utc::now().timestamp() as usize),
            )
            .ok_or_else(|| anyhow!("RETRY_AFTER contains an invalid character"))?;
        Ok(response)
    }
}

#[async_trait]
impl TMiddleware for Middleware {
    async fn run<'n>(&self, ctx: &Context, request: Request, next: Next<'n>) -> Result<Response> {
        let ip = match request
            .header("X-Real-IP")
            .and_then(|header| header.to_str().ok())
        {
            Some(ip) => ip.to_string(),
            None => {
                return Ok(Response::new(StatusCode::BAD_REQUEST));
            }
        };
        let token = match request
            .header("X-Api-Token")
            .and_then(|header| header.to_str().ok())
        {
            Some(token) => token.to_string(),
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        let config = match self.config.config.get(&ctx.app_id) {
            Some(config) => config,
            None => {
                warn!("No config found for app: {}", ctx.app_id);
                return Ok(Response::new(StatusCode::BAD_GATEWAY));
            }
        };
        let rules = config.auth.get(&token).unwrap_or(&config.root);
        let quota = match rules
            .endpoints
            .get(&ctx.endpoint_id)
            .or(rules.quota.as_ref())
        {
            Some(quota) => quota,
            None => {
                warn!("No quota found for endpoint: {}", ctx.endpoint_id);
                return next.run(request).await;
            }
        };
        let total_key = format!("{}--{}--{}", ctx.app_id, ctx.endpoint_id, token);
        let user_key = format!("{}--{}--{}", ctx.app_id, ctx.endpoint_id, ip);

        let rate_limit = {
            use datastore::Response::*;
            match quota.user.as_ref() {
                Some(frequency) => match self.datastore.get_rate_limit(&user_key, frequency).await?
                {
                    Ok(rate_limit) => Some(rate_limit),
                    Limited(reset) => {
                        return Self::too_many_requests(reset);
                    }
                },
                None => None,
            }
        };
        {
            use datastore::Response::*;
            if let Limited(reset) = self
                .datastore
                .get_rate_limit(&total_key, &quota.total)
                .await?
            {
                return Self::too_many_requests(reset);
            };
        }
        let mut response = next.run(request).await?;
        if let Some(rate_limit) = rate_limit {
            response
                .insert_header("RateLimit-Limit", rate_limit.limit)
                .ok_or_else(|| anyhow!("RateLimit-Limit contains an invalid character"))?;
            response
                .insert_header("RateLimit-Remaining", rate_limit.remaining)
                .ok_or_else(|| anyhow!("RateLimit-Remaining contains an invalid character"))?;
            response
                .insert_header(
                    "RateLimit-Reset",
                    rate_limit
                        .reset
                        .saturating_sub(chrono::Utc::now().timestamp() as usize),
                )
                .ok_or_else(|| anyhow!("RateLimit-Reset contains an invalid character"))?;
        }
        Ok(response)
    }
}
