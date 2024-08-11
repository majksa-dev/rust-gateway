use crate::{ConfigToContext, Result};
use async_trait::async_trait;

use super::config;

#[derive(Debug)]
pub struct Connection {
    pub addr: Box<str>,
    pub host: Option<Box<str>>,
    pub tls: bool,
}

impl Connection {
    pub fn new(addr: Box<str>, host: Option<Box<str>>, tls: bool) -> Self {
        Self { addr, host, tls }
    }
}

#[async_trait]
impl ConfigToContext for config::Connection {
    type Context = Connection;

    async fn into_context(self) -> Result<Self::Context> {
        Ok(Self::Context::new(
            self.addr.into_context().await?,
            self.host.into_context().await?,
            self.tls.into_context().await?,
        ))
    }
}
