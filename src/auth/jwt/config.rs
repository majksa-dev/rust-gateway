use std::collections::HashMap;

use crate::auth::Enable;

#[derive(Debug)]
pub struct Config {
    pub apps: HashMap<String, AppConfig>,
}

impl Config {
    pub fn new(apps: HashMap<String, AppConfig>) -> Self {
        Self { apps }
    }
}

#[derive(Debug)]
pub struct AppConfig {
    pub rules: Vec<AuthConfig>,
}

impl AppConfig {
    pub fn new(rules: Vec<AuthConfig>) -> Self {
        Self { rules }
    }
}

#[derive(Debug)]
pub struct AuthConfig {
    pub enable: Enable,
    pub keys_url: reqwest::Url,
    pub claims: Vec<Claim>,
}

impl AuthConfig {
    pub fn new(enable: Enable, keys_url: reqwest::Url, claims: Vec<Claim>) -> Self {
        Self {
            enable,
            keys_url,
            claims,
        }
    }
}

#[derive(Debug)]
pub struct Claim {
    pub claim: String,
    pub header: String,
}
