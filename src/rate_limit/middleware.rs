use super::{datastore, Datastore};
use crate::{
    gateway::{middleware::Middleware as TMiddleware, next::Next, Result},
    http::{headers, HeaderMapExt, Request, Response},
    Ctx,
};
use async_trait::async_trait;
use essentials::warn;
use http::{header, StatusCode};

pub struct Middleware {
    ctx: super::Context,
    datastore: Box<dyn Datastore + Sync + 'static>,
}

unsafe impl Send for Middleware {}
unsafe impl Sync for Middleware {}

impl Middleware {
    pub(crate) fn new(ctx: super::Context, datastore: Box<dyn Datastore + Sync + 'static>) -> Self {
        Self { ctx, datastore }
    }

    fn too_many_requests(reset: usize) -> Response {
        let mut response = Response::new(StatusCode::TOO_MANY_REQUESTS);
        response.insert_header(
            header::RETRY_AFTER,
            reset.saturating_sub(chrono::Utc::now().timestamp() as usize),
        );
        response
    }
}

#[async_trait]
impl TMiddleware for Middleware {
    async fn run<'n>(&self, ctx: &Ctx, request: Request, next: Next<'n>) -> Result<Response> {
        let config = match self.ctx.get(ctx.app_id) {
            Some(config) => config,
            None => {
                return next.run(request).await;
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
        let quota = config
            .get(ctx.endpoint_id)
            .and_then(|rules| rules.find_quota(&token))
            .or_else(|| config.global().find_quota(&token));
        let quota = match quota {
            Some(quota) => quota,
            None => {
                warn!("No quota found for endpoint: {}", ctx.endpoint_id);
                return next.run(request).await;
            }
        };
        let ip = match request
            .header(&headers::REAL_IP)
            .and_then(|header| header.to_str().ok())
        {
            Some(ip) => ip.to_string(),
            None => {
                return Ok(Response::new(StatusCode::BAD_REQUEST));
            }
        };
        let total_key = format!("{}--{}--{}", ctx.app_id, ctx.endpoint_id, token);
        let user_key = format!("{}--{}--{}", ctx.app_id, ctx.endpoint_id, ip);

        let rate_limit = {
            use datastore::Response;
            match quota.user.as_ref() {
                Some(frequency) => match self.datastore.get_rate_limit(&user_key, frequency).await?
                {
                    Response::Ok(rate_limit) => Some(rate_limit),
                    Response::Limited(reset) => {
                        return Ok(Self::too_many_requests(reset));
                    }
                },
                None => None,
            }
        };
        {
            use datastore::Response;
            if let Response::Limited(reset) = self
                .datastore
                .get_rate_limit(&total_key, &quota.total)
                .await?
            {
                return Ok(Self::too_many_requests(reset));
            };
        }
        let mut response = next.run(request).await?;
        if let Some(rate_limit) = rate_limit {
            response.insert_header(&headers::RATE_LIMIT_LIMIT, rate_limit.limit);
            response.insert_header(&headers::RATE_LIMIT_REMAINING, rate_limit.remaining);
            response.insert_header(
                &headers::RATE_LIMIT_RESET,
                rate_limit
                    .reset
                    .saturating_sub(chrono::Utc::now().timestamp() as usize),
            );
        }
        Ok(response)
    }
}
