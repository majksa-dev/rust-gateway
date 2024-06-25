#[derive(Debug, Default)]
pub struct AppConfig {
    pub rules: Vec<Auth>,
}

impl AppConfig {
    pub fn new(rules: Vec<Auth>) -> Self {
        Self { rules }
    }
}

#[derive(Debug)]
pub struct Auth {
    pub token: String,
    pub origins: Option<Vec<String>>,
}

impl Auth {
    pub fn new(token: String, origins: Option<Vec<String>>) -> Self {
        Self { token, origins }
    }
}
