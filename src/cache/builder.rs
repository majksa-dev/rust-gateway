use std::collections::HashMap;

use async_trait::async_trait;

use crate::{Result, Service};

use super::{Config, Datastore};

pub struct MiddlewareBuilder {
    config: Config,
    datastore: Box<dyn Datastore + Send + Sync + 'static>,
}

impl MiddlewareBuilder {
    pub fn new(
        config: impl Into<Config>,
        datastore: impl Datastore + Send + Sync + 'static,
    ) -> Self {
        Self {
            config: config.into(),
            datastore: Box::new(datastore),
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
            self.datastore,
        )))
    }
}
