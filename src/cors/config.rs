use std::{collections::HashMap, sync::Arc};

use http::Method;

#[derive(Debug)]
pub struct CorsConfig {
    pub config: HashMap<String, Arc<Config>>,
}

unsafe impl Send for CorsConfig {}
unsafe impl Sync for CorsConfig {}

#[derive(Debug)]
pub struct Config {
    pub rules: ConfigRules,
    pub endpoints: HashMap<String, ConfigRules>,
}

#[derive(Debug)]
pub struct ConfigRules {
    pub methods: Vec<Method>,
    pub headers: Vec<String>,
    pub auth: Vec<Auth>,
}

pub enum AllowedResult {
    Allowed,
    Forbidden,
    NotFound,
    MethodNotAllowed,
}

impl ConfigRules {
    pub fn is_allowed(&self, origin: &str, token: &str, method: &Method) -> AllowedResult {
        if method != Method::OPTIONS && self.methods.contains(method) {
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
