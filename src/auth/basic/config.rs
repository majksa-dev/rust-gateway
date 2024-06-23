use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Config {
    pub config: HashMap<String, AppConfig>,
}

impl Config {
    pub fn new(config: HashMap<String, AppConfig>) -> Self {
        Self { config }
    }
}

unsafe impl Send for Config {}
unsafe impl Sync for Config {}

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
    pub rules: Vec<Auth>,
}

impl AppConfig {
    pub fn new(rules: Vec<Auth>) -> Self {
        Self { rules }
    }

    pub fn authenticate(&self, username: &String, password: &String, endpoint: &String) -> bool {
        self.rules
            .iter()
            .find_map(|auth| auth.authenticate(username, password))
            .is_some_and(|enable| enable.is_enabled(endpoint))
    }
}

#[derive(Debug)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

#[derive(Debug)]
pub struct Auth {
    pub enable: Enable,
    pub credentials: Vec<Credential>,
}

impl Auth {
    pub fn new(enable: Enable, credentials: Vec<Credential>) -> Self {
        Self {
            enable,
            credentials,
        }
    }

    pub fn authenticate(&self, username: &String, password: &String) -> Option<&Enable> {
        self.credentials
            .iter()
            .find(|credential| credential.username == *username)
            .and_then(|credential| {
                if credential.password == *password {
                    Some(&self.enable)
                } else {
                    None
                }
            })
    }
}
