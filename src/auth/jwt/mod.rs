mod builder;
pub mod config;
mod context;
mod middleware;
mod token;

use std::collections::HashMap;

pub(crate) use middleware::Middleware;

use crate::{MiddlewareConfig, MiddlewareCtx};
use builder::MiddlewareBuilder;

type Config = MiddlewareConfig<config::App, ()>;
type Context = MiddlewareCtx<context::App, ()>;

#[derive(Debug, Default)]
pub struct Builder(HashMap<String, config::App>);

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_app_auth(mut self, app: &str, auth: config::Auth) -> Self {
        match self.0.get_mut(app) {
            Some(config) => {
                config.rules.push(auth);
            }
            None => {
                self.0.insert(app.to_string(), config::App::new(vec![auth]));
            }
        };
        self
    }

    pub fn build(self) -> MiddlewareBuilder {
        let config: Config = self
            .0
            .into_iter()
            .map(|(app, config)| (app, (config, HashMap::new()).into()))
            .collect::<HashMap<_, _>>()
            .into();
        MiddlewareBuilder::new(config)
    }
}
