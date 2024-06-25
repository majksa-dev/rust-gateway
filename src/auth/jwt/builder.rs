use std::collections::HashMap;

use async_trait::async_trait;

use crate::{Result, Service};

pub struct MiddlewareBuilder {
    config: super::Config,
}

impl MiddlewareBuilder {
    pub fn new(config: impl Into<super::Config>) -> Self {
        Self {
            config: config.into(),
        }
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
            self.config.into_context(ids, routers).await?,
        )))
    }
}
