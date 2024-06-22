use std::collections::HashMap;

use crate::utils::time::Frequency;

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
    pub root: Rules,
    pub auth: HashMap<String, Rules>,
}

impl AppConfig {
    pub fn new(root: Rules, auth: HashMap<String, Rules>) -> Self {
        Self { root, auth }
    }
}

#[derive(Debug)]
pub struct Rules {
    pub quota: Option<Quota>,
    pub endpoints: HashMap<String, Quota>,
}

impl Rules {
    pub fn new(quota: Option<Quota>, endpoints: HashMap<String, Quota>) -> Self {
        Self { quota, endpoints }
    }
}

#[derive(Debug)]
pub struct Quota {
    pub total: Frequency,
    pub user: Option<Frequency>,
}
