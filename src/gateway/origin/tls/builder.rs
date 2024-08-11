use crate::{Origin, OriginServerBuilder, Result};
use async_trait::async_trait;
use std::{collections::HashMap, sync::Arc};
use tokio_rustls::{rustls::ClientConfig, TlsConnector};

pub struct TcpOriginBuilder(super::Config, ClientConfig);

impl TcpOriginBuilder {
    pub fn new(config: impl Into<super::Config>, tls: ClientConfig) -> Self {
        Self(config.into(), tls)
    }
}

#[async_trait]
impl OriginServerBuilder for TcpOriginBuilder {
    async fn build(
        self: Box<Self>,
        ids: &[String],
        routers: &HashMap<String, Vec<String>>,
    ) -> Result<Origin> {
        Ok(Box::new(super::Origin {
            context: self.0.into_context(ids, routers).await?,
            connector: TlsConnector::from(Arc::new(self.1)),
        }))
    }
}
