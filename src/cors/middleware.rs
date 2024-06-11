use std::sync::Arc;

use super::{config::AllowedResult, CorsConfig};
use crate::{
    gateway::{
        middleware::{Context, Middleware},
        next::Next,
        Result,
    },
    http::{Request, Response},
};
use async_trait::async_trait;
use http::{header, StatusCode};

#[derive(Debug)]
pub struct Cors(pub CorsConfig);

#[async_trait]
impl Middleware for Cors {
    async fn run(&self, ctx: Arc<Context>, request: Request, next: Next) -> Result<Response> {
        let method = request.method.clone();
        let config = match self.0.config.get(&ctx.app_id) {
            Some(config) => config.clone(),
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
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
                return Ok(Response::new(StatusCode::BAD_REQUEST));
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
                return Ok(Response::new(StatusCode::BAD_REQUEST));
            }
        };
        let rules = match config
            .rules
            .is_allowed(origin.as_str(), token.as_str(), &method)
        {
            AllowedResult::Allowed => &config.rules,
            AllowedResult::MethodNotAllowed => {
                return Ok(Response::new(StatusCode::METHOD_NOT_ALLOWED));
            }
            AllowedResult::Forbidden => {
                return Ok(Response::new(StatusCode::FORBIDDEN));
            }
            AllowedResult::NotFound => {
                let endpoint = match config.endpoints.get(&ctx.endpoint_id) {
                    Some(endpoint) => endpoint,
                    None => {
                        return Ok(Response::new(StatusCode::UNAUTHORIZED));
                    }
                };
                match config
                    .rules
                    .is_allowed(origin.as_str(), token.as_str(), &method)
                {
                    AllowedResult::Allowed => endpoint,
                    AllowedResult::MethodNotAllowed => {
                        return Ok(Response::new(StatusCode::METHOD_NOT_ALLOWED));
                    }
                    AllowedResult::Forbidden => {
                        return Ok(Response::new(StatusCode::FORBIDDEN));
                    }
                    AllowedResult::NotFound => {
                        return Ok(Response::new(StatusCode::UNAUTHORIZED));
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
