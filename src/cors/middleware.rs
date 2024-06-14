use std::sync::Arc;

use super::config;
use crate::{
    gateway::{
        middleware::{Context, Middleware as TMiddleware},
        next::Next,
        Result,
    },
    http::{HeaderMapExt, Request, Response},
    Error,
};
use async_trait::async_trait;
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
    async fn run(&self, ctx: Arc<Context>, request: Request, next: Next) -> Result<Response> {
        let origin = match request
            .header(header::ORIGIN)
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
        {
            Some(origin) => origin,
            None => {
                return Ok(Response::new(StatusCode::BAD_REQUEST));
            }
        };
        if request.method == http::Method::OPTIONS {
            let mut response = Response::new(StatusCode::NO_CONTENT);
            response
                .insert_header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin)
                .ok_or_else(|| {
                    Error::new("ACCESS_CONTROL_ALLOW_ORIGIN contains an invalid character")
                })?;
            return Ok(response);
        }
        let method = request.method.clone();
        let config = match self.config.config.get(&ctx.app_id) {
            Some(config) => config,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        let token = match request
            .header("X-Api-Token")
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
        {
            Some(token) => token,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        if let Some(auth) = config.find_auth(&token) {
            if !auth.is_origin_allowed(&origin) {
                return Ok(Response::new(StatusCode::FORBIDDEN));
            }
        } else {
            return Ok(Response::new(StatusCode::UNAUTHORIZED));
        }

        let mut response = next.run(request).await?;
        response
            .insert_header(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                response
                    .headers()
                    .keys()
                    .map(|key| key.as_str())
                    .collect::<Vec<&str>>()
                    .join(", "),
            )
            .ok_or_else(|| {
                Error::new("ACCESS_CONTROL_ALLOW_HEADERS contains an invalid character")
            })?;
        response
            .insert_header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin)
            .ok_or_else(|| {
                Error::new("ACCESS_CONTROL_ALLOW_ORIGIN contains an invalid character")
            })?;
        response
            .insert_header(header::ACCESS_CONTROL_ALLOW_METHODS, method.to_string())
            .ok_or_else(|| {
                Error::new("ACCESS_CONTROL_ALLOW_METHODS contains an invalid character")
            })?;
        Ok(response)
    }
}
