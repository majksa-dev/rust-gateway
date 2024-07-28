#[derive(Debug)]
pub struct App {
    pub rules: Vec<Auth>,
    pub roles: Option<Vec<String>>,
}

impl App {
    pub fn new(rules: Vec<Auth>, roles: Option<Vec<String>>) -> Self {
        Self { rules, roles }
    }
}

#[derive(Debug)]
pub struct Auth {
    pub url: reqwest::Url,
    pub roles_claim: Option<RolesClaims>,
    pub claims: Vec<Claim>,
}

impl Auth {
    pub fn new(url: reqwest::Url, claims: Vec<Claim>, roles_claim: Option<RolesClaims>) -> Self {
        Self {
            url,
            claims,
            roles_claim,
        }
    }
}

#[derive(Debug)]
pub struct RolesClaims {
    pub claim: String,
    pub inner_mapping: Option<String>,
}

#[derive(Debug)]
pub struct Claim {
    pub claim: String,
    pub header: String,
}

#[derive(Debug)]
pub struct Endpoint {
    pub roles: Vec<String>,
}

impl Endpoint {
    pub fn new(roles: Vec<String>) -> Self {
        Self { roles }
    }
}
