mod builder;
pub mod config;
mod context;
pub mod datastore;
mod middleware;
mod response;

use std::collections::HashMap;

use builder::MiddlewareBuilder;
use datastore::Datastore;
pub(crate) use middleware::Middleware;

use crate::{MiddlewareConfig, MiddlewareCtx};

type Config = MiddlewareConfig<(), config::Endpoint>;
type Context = MiddlewareCtx<(), context::Endpoint>;

#[derive(Debug, Default)]
pub struct Builder(HashMap<String, HashMap<String, config::Endpoint>>);

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_endpoint(
        mut self,
        app: &str,
        endpoint_id: &str,
        endpoint: config::Endpoint,
    ) -> Self {
        match self.0.get_mut(app) {
            Some(config) => {
                config.insert(endpoint_id.to_string(), endpoint);
            }
            None => {
                self.0.insert(
                    app.to_string(),
                    HashMap::from([(endpoint_id.to_string(), endpoint)]),
                );
            }
        };
        self
    }

    pub fn build(self, datastore: impl Datastore + Send + Sync + 'static) -> MiddlewareBuilder {
        let config: Config = self
            .0
            .into_iter()
            .map(|(app, config)| (app, ((), config).into()))
            .collect::<HashMap<_, _>>()
            .into();
        MiddlewareBuilder::new(config, datastore)
    }
}

impl From<HashMap<String, HashMap<String, config::Endpoint>>> for Builder {
    fn from(auth: HashMap<String, HashMap<String, config::Endpoint>>) -> Self {
        Self(auth)
    }
}
