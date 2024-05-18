use std::sync::Arc;

use async_trait::async_trait;
use http::{
    header::{
        ACCESS_CONTROL_ALLOW_HEADERS, ACCESS_CONTROL_ALLOW_METHODS, ACCESS_CONTROL_ALLOW_ORIGIN,
    },
    StatusCode,
};
use pingora::{http::ResponseHeader, proxy::Session, Result};

use crate::{
    gateway::{entrypoint::Context, middleware::AnyContext},
    Middleware,
};

use super::{config::AllowedResult, Config, CorsConfig};

#[derive(Debug)]
pub struct Cors(pub CorsConfig);

unsafe impl Send for Cors {}
unsafe impl Sync for Cors {}

#[derive(Default, Debug)]
pub struct CorsContext {
    config: Option<Arc<Config>>,
    origin: Option<String>,
    token: Option<String>,
    allowed_headers: Option<String>,
}

unsafe impl Send for CorsContext {}
unsafe impl Sync for CorsContext {}

type Ctx = Box<CorsContext>;

#[async_trait]
impl Middleware for Cors {
    fn new_ctx(&self) -> AnyContext {
        Box::<CorsContext>::default()
    }

    async fn filter(
        &self,
        session: &Session,
        ctx: (&Context, &mut AnyContext),
    ) -> Result<Option<ResponseHeader>> {
        let my_ctx = ctx.1.downcast_mut::<Ctx>().unwrap();
        let config = match self.0.config.get(&ctx.0.id) {
            Some(config) => {
                my_ctx.config = Some(config.clone());
                config.clone()
            }
            None => {
                return Ok(None);
            }
        };
        my_ctx.origin = session
            .get_header("Origin")
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string());
        my_ctx.token = session
            .get_header("X-Api-Token")
            .and_then(|header| header.to_str().ok())
            .map(|header| header.to_string());
        if my_ctx.origin.is_none() || my_ctx.origin.is_none() {
            return Ok(Some(ResponseHeader::build(
                StatusCode::BAD_REQUEST,
                Some(2),
            )?));
        }
        let origin = my_ctx.origin.as_ref().unwrap();
        let token = my_ctx.token.as_ref().unwrap();
        let rules = match config
            .rules
            .is_allowed(origin, token, &session.req_header().method)
        {
            AllowedResult::Allowed => &config.rules,
            AllowedResult::MethodNotAllowed => {
                return Ok(Some(ResponseHeader::build(
                    StatusCode::METHOD_NOT_ALLOWED,
                    Some(2),
                )?));
            }
            AllowedResult::Forbidden => {
                return Ok(Some(ResponseHeader::build(StatusCode::FORBIDDEN, Some(2))?));
            }
            AllowedResult::NotFound => {
                let endpoint = match config.endpoints.get(&ctx.0.endpoint) {
                    Some(endpoint) => endpoint,
                    None => {
                        return Ok(Some(ResponseHeader::build(
                            StatusCode::UNAUTHORIZED,
                            Some(2),
                        )?));
                    }
                };
                match config
                    .rules
                    .is_allowed(origin, token, &session.req_header().method)
                {
                    AllowedResult::Allowed => endpoint,
                    AllowedResult::MethodNotAllowed => {
                        return Ok(Some(ResponseHeader::build(
                            StatusCode::METHOD_NOT_ALLOWED,
                            Some(2),
                        )?));
                    }
                    AllowedResult::Forbidden => {
                        return Ok(Some(ResponseHeader::build(StatusCode::FORBIDDEN, Some(2))?));
                    }
                    AllowedResult::NotFound => {
                        return Ok(Some(ResponseHeader::build(
                            StatusCode::UNAUTHORIZED,
                            Some(2),
                        )?));
                    }
                }
            }
        };
        my_ctx.allowed_headers = Some(format!("Authorization,Content-Type,X-Api-Token,X-Real-IP,X-RateLimit-Limit,X-RateLimit-Remaining,X-RateLimit-Reset{}", rules.headers.join(",")));
        Ok(None)
    }

    async fn modify_response(
        &self,
        session: &mut Session,
        response: &mut ResponseHeader,
        ctx: (&Context, &mut AnyContext),
    ) -> Result<()> {
        let my_ctx = ctx.1.downcast_mut::<Ctx>().unwrap();
        if my_ctx.config.is_none() {
            return Ok(());
        };
        response.insert_header(
            ACCESS_CONTROL_ALLOW_ORIGIN,
            my_ctx.origin.as_ref().unwrap().clone(),
        )?;
        response.insert_header(
            ACCESS_CONTROL_ALLOW_METHODS,
            session.req_header().method.to_string(),
        )?;
        response.insert_header(
            ACCESS_CONTROL_ALLOW_HEADERS,
            my_ctx.allowed_headers.as_ref().unwrap().clone(),
        )?;
        Ok(())
    }
}
