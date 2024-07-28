use crate::{
    gateway::{middleware::Middleware as TMiddleware, next::Next, Result},
    http::{HeaderMapExt, Request, Response},
    Ctx,
};
use async_trait::async_trait;
use http::{header, HeaderName, StatusCode};

use super::context::AuthResult;

#[derive(Debug)]
pub struct Middleware(super::Context);

impl Middleware {
    pub(crate) fn new(ctx: super::Context) -> Self {
        Self(ctx)
    }
}

#[async_trait]
impl TMiddleware for Middleware {
    async fn run(&self, ctx: &Ctx, mut request: Request, next: Next<'_>) -> Result<Response> {
        let app_ctx = match self.0.get(ctx.app_id) {
            Some(config) => config,
            None => {
                return next.run(request).await;
            }
        };
        let app = app_ctx.global();
        let roles = app_ctx
            .get(ctx.endpoint_id)
            .map(|endpoint| &endpoint.roles[..]);
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
        let claims = match app.authenticate(authorization.as_str(), roles).await {
            AuthResult::Ok(claims) => claims,
            AuthResult::Unauthorized => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
            AuthResult::Forbidden => {
                return Ok(Response::new(StatusCode::FORBIDDEN));
            }
        };
        request.remove_header(header::AUTHORIZATION);
        for (header, value) in claims {
            request.insert_header(HeaderName::from_bytes(header.as_bytes())?, value);
        }
        next.run(request).await
    }
}
