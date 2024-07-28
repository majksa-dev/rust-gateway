use std::collections::HashSet;

use crate::{auth::ClaimParser, ConfigToContext};

use super::config;
use anyhow::Result;
use async_trait::async_trait;
use essentials::{debug, warn};
use http::header;
use reqwest::Url;
use serde_json::Value;

#[derive(Debug)]
pub struct App {
    pub rules: Box<[Auth]>,
    pub roles: Option<Box<[Box<str>]>>,
}

#[async_trait]
impl ConfigToContext for config::App {
    type Context = App;

    async fn into_context(self) -> Result<Self::Context> {
        let rules = self
            .rules
            .into_iter()
            .map(move |config| {
                Auth::new(
                    config.url,
                    config.claims.into_boxed_slice(),
                    config.roles_claim.map(|claim| {
                        (
                            claim.claim.into_boxed_str(),
                            claim.inner_mapping.map(String::into_boxed_str),
                        )
                    }),
                )
            })
            .collect();
        let roles = self
            .roles
            .map(|o| o.into_iter().map(String::into_boxed_str).collect());
        Ok(App { rules, roles })
    }
}

#[derive(Debug)]
pub struct Endpoint {
    pub roles: Box<[Box<str>]>,
}

impl Endpoint {
    pub fn new(roles: Box<[Box<str>]>) -> Self {
        Self { roles }
    }
}

#[async_trait]
impl ConfigToContext for config::Endpoint {
    type Context = Endpoint;

    async fn into_context(self) -> Result<Self::Context> {
        let roles = self
            .roles
            .into_iter()
            .map(String::into_boxed_str)
            .collect::<Box<_>>();
        Ok(Self::Context { roles })
    }
}

pub enum AuthResult<'a> {
    Ok(Vec<(&'a String, String)>),
    Unauthorized,
    Forbidden,
}

fn has_roles(
    get_roles: impl FnOnce() -> Option<HashSet<Box<str>>>,
    mut app_roles: Option<&[Box<str>]>,
    mut endpoint_roles: Option<&[Box<str>]>,
) -> bool {
    if app_roles.is_some_and(|slice| slice.is_empty()) {
        app_roles = None;
    }
    if endpoint_roles.is_some_and(|slice| slice.is_empty()) {
        endpoint_roles = None;
    }
    let (roles, additional_roles) = match (app_roles, endpoint_roles) {
        (Some(app_roles), Some(endpoint_roles)) => (app_roles, Some(endpoint_roles)),
        (Some(app_roles), None) => (app_roles, None),
        (None, Some(endpoint_roles)) => (endpoint_roles, None),
        (None, None) => return true,
    };
    let user_roles = match get_roles() {
        Some(roles) => roles,
        None => {
            return false;
        }
    };
    roles.iter().all(|role| user_roles.contains(role))
        && additional_roles.map_or(true, |roles| {
            roles.iter().all(|role| user_roles.contains(role))
        })
}

fn parse_roles(auth: &Auth, claims: Value) -> Option<HashSet<Box<str>>> {
    let (roles_claim, roles_claim_inner) = match auth.roles_claim.as_ref() {
        Some(roles_claim) => roles_claim,
        None => return Some(HashSet::new()),
    };
    let roles_claim = claims.parse_value(roles_claim).ok()?;
    let roles = roles_claim.as_array()?;
    Some(
        roles
            .iter()
            .filter_map(|value| match roles_claim_inner {
                Some(roles_claim_inner) => value
                    .parse(roles_claim_inner)
                    .map(String::into_boxed_str)
                    .ok(),
                None => value.as_str().map(|s| s.to_string().into_boxed_str()),
            })
            .collect(),
    )
}

impl App {
    pub async fn authenticate(
        &self,
        token: &str,
        required_roles: Option<&[Box<str>]>,
    ) -> AuthResult {
        for auth in self.rules.iter() {
            match auth.authenticate(token).await {
                Some(claims) => {
                    if !has_roles(
                        || parse_roles(auth, claims.clone()),
                        self.roles.as_deref(),
                        required_roles,
                    ) {
                        return AuthResult::Forbidden;
                    }
                    return match auth
                        .claims
                        .iter()
                        .map(|claim| {
                            claims
                                .parse(claim.claim.as_str())
                                .map(|value| (&claim.header, value))
                        })
                        .collect::<Result<Vec<_>>>()
                        .map_err(|e| warn!("Failed to parse claim: {}", e))
                    {
                        Ok(claims) => AuthResult::Ok(claims),
                        Err(_) => AuthResult::Unauthorized,
                    };
                }
                None => {
                    continue;
                }
            }
        }
        debug!("no token");
        AuthResult::Unauthorized
    }
}

#[derive(Debug)]
pub struct Auth {
    pub url: Url,
    pub roles_claim: Option<(Box<str>, Option<Box<str>>)>,
    pub claims: Box<[config::Claim]>,
}

impl Auth {
    pub fn new(
        url: Url,
        claims: Box<[config::Claim]>,
        roles_claim: Option<(Box<str>, Option<Box<str>>)>,
    ) -> Self {
        Self {
            url,
            claims,
            roles_claim,
        }
    }

    async fn authenticate(&self, authorization: &str) -> Option<Value> {
        let bytes = reqwest::Client::builder()
            .build()
            .ok()?
            .get(self.url.clone())
            .header(header::AUTHORIZATION, authorization)
            .send()
            .await
            .ok()?
            .bytes()
            .await
            .ok()?;
        serde_json::from_slice(&bytes).ok()
    }
}
