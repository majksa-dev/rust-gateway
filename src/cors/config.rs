use std::collections::HashMap;

use http::Method;

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
    pub rules: ConfigRules,
    pub endpoints: HashMap<String, ConfigRules>,
}

impl AppConfig {
    pub fn new(rules: ConfigRules, endpoints: HashMap<String, ConfigRules>) -> Self {
        Self { rules, endpoints }
    }
}

#[derive(Debug)]
pub struct ConfigRules {
    pub methods: Vec<Method>,
    pub auth: Vec<Auth>,
}

impl ConfigRules {
    pub fn new(methods: Vec<Method>, auth: Vec<Auth>) -> Self {
        Self { methods, auth }
    }
}

impl ConfigRules {
    pub fn is_method_allowed(&self, method: &Method) -> bool {
        method == Method::OPTIONS || self.methods.contains(method)
    }

    pub fn find_auth(&self, token: impl AsRef<str>) -> Option<&Auth> {
        self.auth.iter().find(|auth| auth.token == token.as_ref())
    }
}

#[derive(Debug)]
pub struct Auth {
    pub token: String,
    pub origins: Vec<String>,
}

impl Auth {
    pub fn new(token: impl AsRef<str>, origins: Vec<impl AsRef<str>>) -> Self {
        Self {
            token: token.as_ref().to_string(),
            origins: origins
                .into_iter()
                .map(|origin| origin.as_ref().to_string())
                .collect(),
        }
    }

    pub fn is_origin_allowed(&self, origin: impl AsRef<str>) -> bool {
        self.origins
            .iter()
            .any(|allowed_origin| allowed_origin == origin.as_ref())
    }
}
