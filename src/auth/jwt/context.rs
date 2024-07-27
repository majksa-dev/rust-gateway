use crate::{auth::ClaimParser, ConfigToContext};

use super::{
    config,
    token::{self, fetch_keys, Algorithm, Validation},
};
use anyhow::Result;
use async_trait::async_trait;
use essentials::warn;
use futures::future::join_all;
use jsonwebtoken::jwk::JwkSet;
use serde_json::Value;

#[derive(Debug)]
pub struct App {
    pub rules: Box<[Auth]>,
}

#[async_trait]
impl ConfigToContext for config::App {
    type Context = App;

    async fn into_context(self) -> Result<Self::Context> {
        let rules = join_all(self.rules.into_iter().map(move |config| async move {
            fetch_keys(config.keys_url)
                .await
                .map(|keys| Auth::new(keys, config.claims.into_boxed_slice()))
        }))
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?
        .into_boxed_slice();
        Ok(App { rules })
    }
}

impl App {
    pub async fn authenticate(&self, token: &str) -> Option<Vec<(&String, String)>> {
        for auth in self.rules.iter() {
            match auth.authenticate(token).await {
                Some(claims) => {
                    return auth
                        .claims
                        .iter()
                        .map(|claim| {
                            claims
                                .parse(claim.claim.as_str())
                                .map(|value| (&claim.header, value))
                        })
                        .collect::<Result<Vec<_>>>()
                        .map_err(|e| warn!("Failed to parse claim: {}", e))
                        .ok()
                }
                None => {
                    continue;
                }
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct Auth {
    pub keys: JwkSet,
    pub claims: Box<[config::Claim]>,
}

impl Auth {
    pub fn new(keys: JwkSet, claims: Box<[config::Claim]>) -> Self {
        Self { keys, claims }
    }

    async fn authenticate(&self, token: &str) -> Option<Value> {
        token::parse_token(token, &self.keys.keys, &Validation::new(Algorithm::RS256)).await
    }
}
