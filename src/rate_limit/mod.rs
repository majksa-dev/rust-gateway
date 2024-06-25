mod builder;
pub mod config;
mod context;
pub mod datastore;
mod middleware;

use std::collections::HashMap;

use config::*;
use datastore::Datastore;
pub(crate) use middleware::Middleware;

use crate::{MiddlewareConfig, MiddlewareCtx};
use builder::MiddlewareBuilder;

type Config = MiddlewareConfig<config::Rules, config::Rules>;
type Context = MiddlewareCtx<context::Rules, context::Rules>;

#[derive(Debug, Default)]
pub struct Builder(HashMap<String, (config::Rules, EndpointBuilder)>);

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_app(mut self, app: &str, root: config::Rules, endpoints: EndpointBuilder) -> Self {
        self.0.insert(app.to_string(), (root, endpoints));
        self
    }

    pub fn build(self, datastore: impl Datastore + Send + Sync + 'static) -> MiddlewareBuilder {
        let config: Config = self
            .0
            .into_iter()
            .map(|(app, (root, endpoints))| (app, (root, endpoints.0).into()))
            .collect::<HashMap<_, _>>()
            .into();
        MiddlewareBuilder::new(config, datastore)
    }
}

#[derive(Debug, Default)]
pub struct EndpointBuilder(HashMap<String, config::Rules>);

impl EndpointBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_endpoint(mut self, endpoint: &str, root: config::Rules) -> Self {
        self.0.insert(endpoint.to_string(), root);
        self
    }
}
