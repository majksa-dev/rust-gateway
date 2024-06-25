#[derive(Debug)]
pub struct App {
    pub rules: Vec<Auth>,
}

impl App {
    pub fn new(rules: Vec<Auth>) -> Self {
        Self { rules }
    }
}

#[derive(Debug)]
pub struct Auth {
    pub keys_url: reqwest::Url,
    pub claims: Vec<Claim>,
}

impl Auth {
    pub fn new(keys_url: reqwest::Url, claims: Vec<Claim>) -> Self {
        Self { keys_url, claims }
    }
}

#[derive(Debug)]
pub struct Claim {
    pub claim: String,
    pub header: String,
}
