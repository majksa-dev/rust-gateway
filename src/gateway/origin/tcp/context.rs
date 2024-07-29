use crate::{ConfigToContext, Result};
use async_trait::async_trait;

use super::config;

#[derive(Debug)]
pub struct Connection {
    pub addr: Box<str>,
    pub host: Box<str>,
}

impl Connection {
    pub fn new(addr: Box<str>, host: Box<str>) -> Self {
        Self { addr, host }
    }
}

#[async_trait]
impl ConfigToContext for config::Connection {
    type Context = Connection;

    async fn into_context(self) -> Result<Self::Context> {
        Ok(Self::Context::new(
            self.addr.into_context().await?,
            self.host.into_context().await?,
        ))
    }
}
