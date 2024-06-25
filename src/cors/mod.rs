mod builder;
pub mod config;
mod context;
mod middleware;

use std::collections::HashMap;

pub use builder::MiddlewareBuilder;
pub(crate) use middleware::Middleware;

use crate::{MiddlewareConfig, MiddlewareCtx};

type Config = MiddlewareConfig<config::AppConfig, ()>;
type Context = MiddlewareCtx<context::AppConfig, ()>;

#[derive(Debug, Default)]
pub struct Builder(HashMap<String, config::AppConfig>);

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_auth(mut self, app: &str, token: String, origins: Vec<String>) -> Self {
        match self.0.get_mut(app) {
            Some(config) => {
                config.rules.push(config::Auth::new(token, origins));
            }
            None => {
                self.0.insert(
                    app.to_string(),
                    config::AppConfig::new(vec![config::Auth::new(token, origins)]),
                );
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
