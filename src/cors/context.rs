use async_trait::async_trait;

use crate::{ConfigToContext, Result};

use super::config;

#[derive(Debug)]
pub struct AppConfig {
    pub rules: Box<[Auth]>,
}

impl AppConfig {
    fn new(rules: Box<[Auth]>) -> Self {
        Self { rules }
    }

    pub fn find_auth(&self, token: impl AsRef<str>) -> Option<&Auth> {
        self.rules.iter().find(|auth| auth.token == token.as_ref())
    }
}

#[derive(Debug)]
pub struct Auth {
    pub token: String,
    pub origins: Box<[String]>,
}

impl Auth {
    fn new(token: String, origins: Box<[String]>) -> Self {
        Self { token, origins }
    }

    pub fn is_origin_allowed(&self, origin: impl AsRef<str>) -> bool {
        self.origins.len() == 0
            || self
                .origins
                .iter()
                .any(|allowed_origin| allowed_origin == origin.as_ref())
    }
}

#[async_trait]
impl ConfigToContext for config::AppConfig {
    type Context = AppConfig;

    async fn into_context(self) -> Result<Self::Context> {
        Ok(AppConfig::new(
            self.rules
                .into_iter()
                .map(|config| Auth::new(config.token, config.origins.into_boxed_slice()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ))
    }
}
