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
        let config = self
            .0
            .config
            .get(&ctx.app_id)
            .ok_or(Error::status(StatusCode::UNAUTHORIZED))?;
        let origin = request
            .headers
            .get(header::ORIGIN)
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
            .ok_or(Error::status(StatusCode::BAD_REQUEST))?;
        let token = request
            .headers
            .get("X-Api-Token")
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
            .ok_or(Error::status(StatusCode::BAD_REQUEST))?;
        let endpoint = config
            .endpoints
            .get(&ctx.endpoint_id)
            .ok_or(Error::status(StatusCode::UNAUTHORIZED))?;
        if !config.rules.is_method_allowed(&method) && !endpoint.is_method_allowed(&method) {
            return Err(Error::status(StatusCode::METHOD_NOT_ALLOWED));
        }
        if let Some(auth) = config
            .rules
            .find_auth(&token)
            .or(endpoint.find_auth(&token))
        {
            if !auth.is_origin_allowed(&origin) {
                return Err(Error::status(StatusCode::FORBIDDEN));
            }
        } else {
            return Err(Error::status(StatusCode::UNAUTHORIZED));
        }

        let mut response = next.run(request).await?;
        response
            .insert_header(
                header::ACCESS_CONTROL_ALLOW_HEADERS,
                response
                    .headers
                    .keys()
                    .map(|key| key.as_str())
                    .collect::<Vec<&str>>()
                    .join(", "),
            )
            .unwrap();
        response
            .insert_header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin)
            .unwrap();
        response
            .insert_header(header::ACCESS_CONTROL_ALLOW_METHODS, method.to_string())
            .unwrap();
        Ok(response)
    }
}
