use std::collections::HashMap;

use crate::time::Time;

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
    pub endpoints: HashMap<String, Quota>,
}

impl AppConfig {
    pub fn new(endpoints: HashMap<String, Quota>) -> Self {
        Self { endpoints }
    }
}

#[derive(Debug)]
pub struct Quota {
    pub expires_in: Time,
}

impl Quota {
    pub fn new(expires_in: Time) -> Self {
        Self { expires_in }
    }
}
