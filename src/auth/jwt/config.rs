use std::collections::{HashMap, HashSet};

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
pub enum Enable {
    All,
    Endpoints(HashSet<String>),
}

impl Enable {
    pub fn is_enabled(&self, endpoint: &String) -> bool {
        match self {
            Self::All => true,
            Self::Endpoints(endpoints) => endpoints.contains(endpoint),
        }
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