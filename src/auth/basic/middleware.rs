use crate::{
    gateway::{middleware::Middleware as TMiddleware, next::Next, Result},
    http::{headers, HeaderMapExt, Request, Response},
    Ctx,
};
use anyhow::{anyhow, bail, Context};
use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE, Engine};
use essentials::warn;
use http::{header, StatusCode};

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
        let auth = match authorization.strip_prefix("Basic ") {
            Some(auth) => auth,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        let credentials = URL_SAFE
            .decode(auth)
            .with_context(|| "Failed to decode Authorization header")
            .and_then(|decoded| {
                let decoded = String::from_utf8(decoded).map_err(|_| anyhow!("Invalid UTF-8"))?;
                let parts: Vec<_> = decoded.splitn(2, ':').collect();
                if parts.len() != 2 {
                    bail!("Invalid Authorization header");
                }
                Ok((parts[0].to_string(), parts[1].to_string()))
            });
        let (username, password) = match credentials {
            Ok(credentials) => credentials,
            Err(err) => {
                warn!("Failed to decode Authorization header: {}", err);
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        let config = match self.0.get(ctx.app_id) {
            Some(config) => config,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        }
        .global();
        if !config.authenticate(&username, &password) {
            return Ok(Response::new(StatusCode::FORBIDDEN));
        }
        request.remove_header(header::AUTHORIZATION);
        request.insert_header(&headers::USERNAME, username);
        next.run(request).await
    }
}
