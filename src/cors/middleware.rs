use crate::{
    gateway::{middleware::Middleware as TMiddleware, next::Next, Result},
    http::{headers, HeaderMapExt, Request, Response},
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
    async fn run(&self, ctx: &Ctx, request: Request, next: Next<'_>) -> Result<Response> {
        let config = match self.0.get(ctx.app_id) {
            Some(config) => config,
            None => {
                return next.run(request).await;
            }
        }
        .global();
        let token = match request
            .header(&headers::API_TOKEN)
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
        {
            Some(token) => token,
            None => {
                return Ok(Response::new(StatusCode::UNAUTHORIZED));
            }
        };
        let origin = match request
            .header(header::ORIGIN)
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string())
        {
            Some(origin) => origin,
            None => {
                if let Some(auth) = config.find_auth(&token) {
                    if !auth.is_any_origin_allowed() {
                        return Ok(Response::new(StatusCode::FORBIDDEN));
                    }
                } else {
                    return Ok(Response::new(StatusCode::UNAUTHORIZED));
                }
                return next.run(request).await;
            }
        };
        if request.method == http::Method::OPTIONS {
            let mut response = Response::new(StatusCode::NO_CONTENT);
            response.insert_header(header::ACCESS_CONTROL_ALLOW_ORIGIN, origin);
            return Ok(response);
        }
        let method = request.method.clone();
        if let Some(auth) = config.find_auth(&token) {
            if !auth.is_any_origin_allowed() && !auth.is_origin_allowed(&origin) {
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
