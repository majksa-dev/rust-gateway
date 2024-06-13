use std::{collections::HashMap, sync::Arc};

use http::Method;

#[derive(Debug)]
pub struct Config {
    pub config: HashMap<String, Arc<AppConfig>>,
}

impl Config {
    pub fn new(config: HashMap<String, Arc<AppConfig>>) -> Self {
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
    pub headers: Vec<String>,
    pub auth: Vec<Auth>,
}

impl ConfigRules {
    pub fn new(methods: Vec<Method>, headers: Vec<String>, auth: Vec<Auth>) -> Self {
        Self {
            methods,
            headers,
            auth,
        }
    }
}

pub enum AllowedResult {
    Allowed,
    Forbidden,
    NotFound,
    MethodNotAllowed,
}

impl ConfigRules {
    pub fn is_allowed(&self, origin: &str, token: &str, method: &Method) -> AllowedResult {
        if method != Method::OPTIONS && !self.methods.contains(method) {
            return AllowedResult::MethodNotAllowed;
        }
        self.auth
            .iter()
            .find(|auth| auth.token == token)
            .map(|auth| {
                auth.origins
                    .iter()
                    .any(|allowed_origin| origin == allowed_origin)
            })
            .map_or(AllowedResult::NotFound, |allowed| {
                if allowed {
                    AllowedResult::Allowed
                } else {
                    AllowedResult::Forbidden
                }
            })
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
}
