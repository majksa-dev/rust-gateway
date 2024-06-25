use crate::{
    gateway::{middleware::Middleware as TMiddleware, next::Next, Result},
    http::{HeaderMapExt, Request, Response},
    Ctx,
};
use async_trait::async_trait;
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
    async fn run<'n>(&self, ctx: &Ctx, request: Request, next: Next<'n>) -> Result<Response> {
        let origin = match request
            .header(header::ORIGIN)
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
        {
            Some(origin) => origin,
            None => {
                let config = match self.0.get(ctx.app_id) {
                    Some(config) => config,
                    None => {
                        return Ok(Response::new(StatusCode::UNAUTHORIZED));
                    }
                }
                .global();
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
                    if !auth.is_any_origin_allowed() {
                        return Ok(Response::new(StatusCode::FORBIDDEN));
                    }
                } else {
                    return Ok(Response::new(StatusCode::UNAUTHORIZED));
                }
                return Ok(Response::new(StatusCode::BAD_REQUEST));
            }
        };
        if request.method == http::Method::OPTIONS {
            let mut response = Response::new(StatusCode::NO_CONTENT);
            response.insert_header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
            return Ok(response);
        }
        let method = request.method.clone();
        let config = match self.0.get(ctx.app_id) {
            Some(config) => config,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        }
        .global();
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
        response.insert_header(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            response
                .headers()
                .keys()
                .map(|key| key.as_str())
                .collect::<Vec<&str>>()
                .join(", "),
        );
        response.insert_header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
        response.insert_header(header::ACCESS_CONTROL_ALLOW_METHODS, method.to_string());
        Ok(response)
    }
}
