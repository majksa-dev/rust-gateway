use std::sync::Arc;

use super::config;
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
use http::{header, StatusCode};

#[derive(Debug)]
pub struct Middleware(pub config::Config);

#[async_trait]
impl TMiddleware for Middleware {
    async fn run(&self, ctx: Arc<Context>, request: Request, next: Next) -> Result<Response> {
        let method = request.method.clone();
        let config = match self.0.config.get(&ctx.app_id) {
            Some(config) => config.clone(),
            None => {
                return Err(Error::status(StatusCode::UNAUTHORIZED));
            }
        };
        let origin = match request
            .headers
            .get(header::ORIGIN)
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
        {
            Some(origin) => origin,
            None => {
                return Err(Error::status(StatusCode::BAD_REQUEST));
            }
        };
        let token = match request
            .headers
            .get("X-Api-Token")
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
        {
            Some(token) => token,
            None => {
                return Err(Error::status(StatusCode::BAD_REQUEST));
            }
        };
        use config::AllowedResult::*;
        let rules = match config
            .rules
            .is_allowed(origin.as_str(), token.as_str(), &method)
        {
            Allowed => &config.rules,
            MethodNotAllowed => {
                return Err(Error::status(StatusCode::METHOD_NOT_ALLOWED));
            }
            Forbidden => {
                return Err(Error::status(StatusCode::FORBIDDEN));
            }
            NotFound => {
                let endpoint = match config.endpoints.get(&ctx.endpoint_id) {
                    Some(endpoint) => endpoint,
                    None => {
                        return Err(Error::status(StatusCode::UNAUTHORIZED));
                    }
                };
                match config
                    .rules
                    .is_allowed(origin.as_str(), token.as_str(), &method)
                {
                    Allowed => endpoint,
                    MethodNotAllowed => {
                        return Err(Error::status(StatusCode::METHOD_NOT_ALLOWED));
                    }
                    Forbidden => {
                        return Err(Error::status(StatusCode::FORBIDDEN));
                    }
                    NotFound => {
                        return Err(Error::status(StatusCode::UNAUTHORIZED));
                    }
                }
            }
        };
        let mut response = next.run(request).await?;
        response
            .insert_header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin)
            .unwrap();
        response
            .insert_header(header::ACCESS_CONTROL_ALLOW_METHODS, method.to_string())
            .unwrap();
        response.insert_header(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            format!("Authorization,Content-Type,X-Api-Token,X-Real-IP,X-RateLimit-Limit,X-RateLimit-Remaining,X-RateLimit-Reset{}", rules.headers.join(",")),
        ).unwrap();
        Ok(response)
    }
}
