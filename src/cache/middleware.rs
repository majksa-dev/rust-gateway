use super::{config, datastore, response::CachedResponseBody, Datastore};
use crate::{
    gateway::{
        middleware::{Context, Middleware as TMiddleware},
        next::Next,
        Result,
    },
    http::{HeaderMapExt, Request, Response},
    time::TimeUnit,
};
use anyhow::anyhow;
use async_trait::async_trait;
use essentials::{debug, warn};
use http::{header, StatusCode};
use pingora_cache::{
    key::{hash_key, CacheHashKey, CompactCacheKey},
    VarianceBuilder,
};

pub struct Middleware {
    config: config::Config,
    datastore: Box<dyn Datastore + Sync + 'static>,
}

unsafe impl Send for Middleware {}
unsafe impl Sync for Middleware {}

impl Middleware {
    pub fn new(config: config::Config, datastore: impl Datastore + Sync + 'static) -> Self {
        Self {
            config,
            datastore: Box::new(datastore),
        }
    }
}

#[async_trait]
impl TMiddleware for Middleware {
    async fn run<'n>(
        &self,
        ctx: &Context,
        mut request: Request,
        next: Next<'n>,
    ) -> Result<Response> {
        let config = match self.config.config.get(&ctx.app_id) {
            Some(config) => config,
            None => {
                warn!("No config found for app: {}", ctx.app_id);
                return Ok(Response::new(StatusCode::BAD_GATEWAY));
            }
        };
        let endpoint = match config.endpoints.get(&ctx.endpoint_id) {
            Some(quota) => quota,
            None => {
                warn!("Cache not configured for endpoint: {}", ctx.endpoint_id);
                return next.run(request).await;
            }
        };
        let ip = request
            .header("X-Real-IP")
            .and_then(|header| header.to_str().ok())
            .unwrap_or("")
            .to_string()
            .into_boxed_str();
        let mut variance = VarianceBuilder::new();
        for header in endpoint.vary_headers.iter() {
            let value = request
                .header(header)
                .and_then(|value| value.to_str().ok())
                .unwrap_or_default();
            variance.add_value(header, value);
        }
        let key = CompactCacheKey {
            primary: hash_key(format!("{}:{}", ctx.app_id, ctx.endpoint_id).as_str()),
            user_tag: ip,
            variance: variance.finalize().map(Box::new),
        };
        let key = key.combined();
        let ttl = endpoint.expires_in.convert(TimeUnit::Seconds).amount;
        let etag = {
            use datastore::Response::*;
            match self.datastore.fetch_cache(key.as_str()).await? {
                Hit(cached, ttl) => {
                    let mut response = Response::new(StatusCode::OK);
                    response.set_body(CachedResponseBody::new(cached.response));
                    for header_raw in cached.headers.lines() {
                        let mut parts = header_raw.splitn(2, ": ");
                        if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                            response.insert_header(key, value);
                        }
                    }
                    response
                        .insert_header(header::CACHE_CONTROL, format!("max-age={}", ttl))
                        .ok_or_else(|| anyhow!("CACHE_CONTROL contains an invalid character"))?;
                    return Ok(response);
                }
                Expired(etag) => etag,
                Miss => None,
            }
        };
        debug!(etag = etag, key = key, "Fetching response from origin");
        if let Some(etag) = &etag {
            request
                .insert_header("If-None-Match", etag.clone())
                .ok_or_else(|| anyhow!("If-None-Match contains an invalid character"))?;
        }
        debug!("Fetching response from origin for key: {}", key);
        let origin_response = next.run(request).await?;
        debug!("Origin response: {:?}", origin_response);
        if origin_response.status == StatusCode::NOT_MODIFIED {
            use datastore::Response::*;
            if let Hit(data, ttl) = self.datastore.refresh_cache(key.as_str(), ttl).await? {
                let mut response = Response::new(StatusCode::OK);
                response
                    .insert_header(header::CACHE_CONTROL, format!("max-age={}", ttl))
                    .ok_or_else(|| anyhow!("CACHE_CONTROL contains an invalid character"))?;
                response.set_body(CachedResponseBody::new(data.response));
                return Ok(response);
            };
        }
        let mut response = Response::new(origin_response.status);
        for (key, value) in origin_response.headers().iter() {
            response.insert_header(key, value);
        }
        response.remove_header(header::ETAG);
        debug!("Caching response for key: {}", key);
        let origin_headers = origin_response
            .headers()
            .iter()
            .map(|(key, value)| key.to_string() + ": " + value.to_str().unwrap_or_default())
            .reduce(|mut a, b| {
                a.push('\n');
                a.push_str(b.as_str());
                a
            })
            .unwrap_or_default();
        let body = if let Some(len) = origin_response.get_content_length() {
            origin_response.body().unwrap().read_all(len).await?
        } else {
            String::new()
        };

        debug!("Response body: {}", body);
        self.datastore
            .save_cache(key.as_str(), body.clone(), origin_headers, etag, ttl)
            .await?;
        response.set_body(CachedResponseBody::new(body));
        response
            .insert_header(header::CACHE_CONTROL, format!("max-age={}", ttl))
            .ok_or_else(|| anyhow!("CACHE_CONTROL contains an invalid character"))?;
        Ok(response)
    }
}