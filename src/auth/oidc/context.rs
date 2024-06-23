use super::{
    jwt::{self, fetch_keys},
    AppConfig, Claim, Config, Enable,
};
use anyhow::Result;
use futures::future::join_all;
use openidconnect::core::CoreJsonWebKey;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context {
    pub apps: HashMap<String, App>,
}

impl Context {
    pub async fn new(config: Config) -> Result<Self> {
        join_all(
            config
                .apps
                .into_iter()
                .map(|(app_id, config)| async { App::new(config).await.map(|app| (app_id, app)) }),
        )
        .await
        .into_iter()
        .collect::<Result<HashMap<_, _>>>()
        .map(|apps| Self { apps })
    }
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

#[derive(Debug)]
pub struct App {
    pub rules: Vec<Auth>,
}

impl App {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let rules = join_all(config.rules.into_iter().map(|config| async {
            fetch_keys(config.issuer_url)
                .await
                .map(|keys| Auth::new(config.enable, keys, config.claims))
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
        Ok(Self { rules })
    }

    pub async fn authenticate(
        &self,
        token: &str,
        endpoint: &String,
    ) -> Option<Vec<(&String, String)>> {
        for auth in self.rules.iter() {
            if auth.enable.is_enabled(endpoint) {
                return auth.authenticate(token).await.and_then(|claims| {
                    claims
                        .into_iter()
                        .map(|(key, value)| {
                            auth.claims
                                .iter()
                                .find(|claim| key == claim.claim)
                                .map(|claim| (&claim.header, value))
                        })
                        .collect::<Option<Vec<_>>>()
                });
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Auth {
    pub enable: Enable,
    pub keys: Vec<CoreJsonWebKey>,
    pub claims: Vec<Claim>,
}

impl Auth {
    pub fn new(enable: Enable, keys: Vec<CoreJsonWebKey>, claims: Vec<Claim>) -> Self {
        Self {
            enable,
            keys,
            claims,
        }
    }

    async fn authenticate(&self, token: &str) -> Option<HashMap<String, String>> {
        jwt::parse_token(token, &self.keys).await
    }
}
