#[derive(Debug)]
pub struct Auth {
    pub credentials: Vec<Credential>,
}

#[derive(Debug)]
pub struct Credential {
    pub username: String,
    pub password: String,
}

impl Auth {
    pub fn new(credentials: Vec<Credential>) -> Self {
        Self { credentials }
    }
}
