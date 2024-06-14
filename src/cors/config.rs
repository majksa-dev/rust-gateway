use std::collections::HashMap;

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
pub struct AppConfig {
    pub rules: Vec<Auth>,
}

impl AppConfig {
    pub fn new(rules: Vec<Auth>) -> Self {
        Self { rules }
    }

    pub fn find_auth(&self, token: impl AsRef<str>) -> Option<&Auth> {
        self.rules.iter().find(|auth| auth.token == token.as_ref())
    }
}

#[derive(Debug)]
pub struct Auth {
    pub token: String,
    pub origins: Vec<String>,
}

impl Auth {
    pub fn new(token: String, origins: Vec<String>) -> Self {
        Self { token, origins }
    }

    pub fn is_origin_allowed(&self, origin: impl AsRef<str>) -> bool {
        self.origins
            .iter()
            .any(|allowed_origin| allowed_origin == origin.as_ref())
    }
}
