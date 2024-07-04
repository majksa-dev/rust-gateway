use crate::{ConfigToContext, Result};
use async_trait::async_trait;

#[derive(Debug)]
pub struct Connection {
    pub addr: String,
}

impl Connection {
    pub fn new(addr: String) -> Self {
        Self { addr }
    }
}

#[async_trait]
impl ConfigToContext for Connection {
    type Context = Box<str>;

    async fn into_context(self) -> Result<Self::Context> {
        self.addr.into_context().await
    }
}
