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
    pub endpoints: HashMap<String, Endpoint>,
}

impl AppConfig {
    pub fn new(endpoints: HashMap<String, Endpoint>) -> Self {
        Self { endpoints }
    }
}

#[derive(Debug)]
pub struct Endpoint {
    pub expires_in: Time,
    pub vary_headers: Vec<String>,
}

impl Endpoint {
    pub fn new(expires_in: Time, vary_headers: Vec<String>) -> Self {
        Self {
            expires_in,
            vary_headers,
        }
    }
}
