use crate::{Origin, OriginServerBuilder, Result};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct TcpOriginBuilder(super::Config);

impl TcpOriginBuilder {
    pub fn new(config: impl Into<super::Config>) -> Self {
        Self(config.into())
    }
}

#[async_trait]
impl OriginServerBuilder for TcpOriginBuilder {
    async fn build(
        self: Box<Self>,
        ids: &[String],
        routers: &HashMap<String, Vec<String>>,
    ) -> Result<Origin> {
        Ok(Box::new(super::Origin(
            self.0.into_context(ids, routers).await?,
        )))
    }
}
