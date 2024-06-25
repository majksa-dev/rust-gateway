use async_trait::async_trait;

use crate::{time::Time, ConfigToContext, Result};

use super::config;

#[derive(Debug)]
pub struct Endpoint {
    pub expires_in: Time,
    pub vary_headers: Box<[String]>,
}

impl Endpoint {
    fn new(expires_in: Time, vary_headers: Box<[String]>) -> Self {
        Self {
            expires_in,
            vary_headers,
        }
    }
}

#[async_trait]
impl ConfigToContext for config::Endpoint {
    type Context = Endpoint;

    async fn into_context(self) -> Result<Self::Context> {
        Ok(Endpoint::new(
            self.expires_in,
            self.vary_headers.into_boxed_slice(),
        ))
    }
}
