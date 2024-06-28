use crate::{
    gateway::{middleware::Middleware as TMiddleware, next::Next, Result},
    http::{HeaderMapExt, Request, Response},
    Ctx,
};
use async_trait::async_trait;
use http::{header, HeaderName, StatusCode};

#[derive(Debug)]
pub struct Middleware(super::Context);

impl Middleware {
    pub(crate) fn new(ctx: super::Context) -> Self {
        Self(ctx)
    }
}

#[async_trait]
impl TMiddleware for Middleware {
    async fn run<'n>(&self, ctx: &Ctx, mut request: Request, next: Next<'n>) -> Result<Response> {
        let app = match self.0.get(ctx.app_id) {
            Some(config) => config,
            None => {
                return next.run(request).await;
            }
        }
        .global();
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
        let token = match authorization.strip_prefix("Bearer ") {
            Some(token) => token,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        let claims = match app.authenticate(token).await {
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
