use super::config;
use crate::{
    gateway::{middleware::Middleware as TMiddleware, next::Next, Result},
    http::{HeaderMapExt, Request, Response},
};
use async_trait::async_trait;
use http::{header, HeaderName, StatusCode};

#[derive(Debug)]
pub struct Middleware {
    context: super::context::Context,
}

impl Middleware {
    pub async fn new(config: config::Config) -> Result<Self> {
        super::context::Context::new(config)
            .await
            .map(|context| Self { context })
    }
}

#[async_trait]
impl TMiddleware for Middleware {
    async fn run<'n>(
        &self,
        ctx: &crate::Context,
        mut request: Request,
        next: Next<'n>,
    ) -> Result<Response> {
        let authorization = match request
            .header(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
        {
            Some(origin) => origin,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        let app = match self.context.apps.get(&ctx.app_id) {
            Some(app) => app,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        let token = match authorization.strip_prefix("Bearer ") {
            Some(token) => token,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        let claims = match app.authenticate(token, &ctx.endpoint_id).await {
            Some(claims) => claims,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        request.remove_header(header::AUTHORIZATION);
        for (header, value) in claims {
            request.insert_header(HeaderName::from_bytes(header.as_bytes())?, value);
        }
        next.run(request).await
    }
}
