use async_trait::async_trait;

use crate::{ConfigToContext, Result};

use super::{config, Quota};

#[derive(Debug)]
pub struct Rules {
    pub root: Option<Quota>,
    pub tokens: Box<[(Box<str>, Quota)]>,
}

impl Rules {
    fn new(root: Option<Quota>, tokens: Box<[(Box<str>, Quota)]>) -> Self {
        Self { root, tokens }
    }

    pub fn find_quota(&self, token: &str) -> Option<&Quota> {
        self.tokens
            .iter()
            .find(|(t, _)| t.as_ref() == token)
            .map(|(_, quota)| quota)
            .or(self.root.as_ref())
    }
}

#[async_trait]
impl ConfigToContext for config::Rules {
    type Context = Rules;

    async fn into_context(self) -> Result<Self::Context> {
        Ok(Rules::new(
            self.root,
            self.tokens
                .into_iter()
                .map(|(token, quota)| (token.into(), quota))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ))
    }
}
