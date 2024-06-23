use super::config;
use crate::{
    gateway::{middleware::Middleware as TMiddleware, next::Next, Result},
    http::{HeaderMapExt, Request, Response},
};
use anyhow::{anyhow, bail, Context};
use async_trait::async_trait;
use base64::{engine::general_purpose::URL_SAFE, Engine};
use essentials::warn;
use http::{header, StatusCode};

#[derive(Debug)]
pub struct Middleware {
    config: config::Config,
}

impl Middleware {
    pub fn new(config: config::Config) -> Self {
        Self { config }
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
        let credentials = URL_SAFE
            .decode(authorization)
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
        let config = match self.config.config.get(&ctx.app_id) {
            Some(config) => config,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        if !config.authenticate(&username, &password, &ctx.endpoint_id) {
            return Ok(Response::new(StatusCode::FORBIDDEN));
        }
        request.remove_header(header::AUTHORIZATION);
        request.insert_header("X-Username", username);
        next.run(request).await
    }
}
