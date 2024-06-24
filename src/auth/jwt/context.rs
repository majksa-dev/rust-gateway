use crate::auth::Enable;

use super::{
    token::{self, fetch_keys, Algorithm, ClaimParser, Validation},
    AppConfig, Claim, Config,
};
use anyhow::Result;
use essentials::warn;
use futures::future::join_all;
use jsonwebtoken::jwk::JwkSet;
use serde_json::Value;
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
        let rules = join_all(config.rules.into_iter().map(move |config| async move {
            fetch_keys(config.keys_url)
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
                    auth.claims
                        .iter()
                        .map(|claim| {
                            claims
                                .parse(claim.claim.as_str())
                                .map(|value| (&claim.header, value))
                        })
                        .collect::<Result<Vec<_>>>()
                        .map_err(|e| warn!("Failed to parse claim: {}", e))
                        .ok()
                });
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Auth {
    pub enable: Enable,
    pub keys: JwkSet,
    pub claims: Vec<Claim>,
}

impl Auth {
    pub fn new(enable: Enable, keys: JwkSet, claims: Vec<Claim>) -> Self {
        Self {
            enable,
            keys,
            claims,
        }
    }

    async fn authenticate(&self, token: &str) -> Option<Value> {
        token::parse_token(token, &self.keys.keys, &Validation::new(Algorithm::RS256)).await
    }
}
