use std::collections::HashMap;

use async_trait::async_trait;

use crate::{MiddlewareConfig, Result, Service};

use super::{config, Config};

pub struct MiddlewareBuilder(MiddlewareConfig<config::AppConfig, ()>);

impl MiddlewareBuilder {
    pub fn new(config: impl Into<Config>) -> Self {
        Self(config.into())
    }
}

#[async_trait]
impl crate::MiddlewareBuilder for MiddlewareBuilder {
    async fn build(
        self: Box<Self>,
        ids: &[String],
        routers: &HashMap<String, Vec<String>>,
    ) -> Result<Service> {
        Ok(Box::new(super::Middleware::new(
            self.0.into_context(ids, routers).await?,
        )))
    }
}
