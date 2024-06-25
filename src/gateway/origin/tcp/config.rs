use crate::{ConfigToContext, Result};
use async_trait::async_trait;
use std::net::SocketAddr;

#[derive(Debug)]
pub struct Connection {
    pub addr: Box<SocketAddr>,
}

impl Connection {
    pub fn new(addr: SocketAddr) -> Self {
        Self {
            addr: Box::new(addr),
        }
    }
}

#[async_trait]
impl ConfigToContext for Connection {
    type Context = Box<SocketAddr>;

    async fn into_context(self) -> Result<Self::Context> {
        Ok(self.addr)
    }
}
