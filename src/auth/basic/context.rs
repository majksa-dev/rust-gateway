use async_trait::async_trait;

use crate::{ConfigToContext, Result};

use super::{config, config::Credential};

#[derive(Debug)]
pub struct Auth {
    pub credentials: Box<[Credential]>,
}

impl Auth {
    fn new(credentials: Box<[Credential]>) -> Self {
        Self { credentials }
    }

    pub fn authenticate(&self, username: &String, password: &String) -> bool {
        self.credentials
            .iter()
            .find(|credential| credential.username == *username)
            .is_some_and(|credential| credential.password == *password)
    }
}

#[async_trait]
impl ConfigToContext for config::Auth {
    type Context = Auth;

    async fn into_context(self) -> Result<Self::Context> {
        Ok(Auth::new(self.credentials.into_boxed_slice()))
    }
}
