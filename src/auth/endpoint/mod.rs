mod builder;
pub mod config;
mod context;
mod middleware;

use std::collections::HashMap;

pub(crate) use middleware::Middleware;

use crate::{MiddlewareConfig, MiddlewareCtx};
use builder::MiddlewareBuilder;

type Config = MiddlewareConfig<config::App, config::Endpoint>;
type Context = MiddlewareCtx<context::App, context::Endpoint>;

#[derive(Debug, Default)]
pub struct Builder(HashMap<String, (config::App, EndpointBuilder)>);

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_app_auth(mut self, app: &str, auth: config::Auth) -> Self {
        match self.0.get_mut(app) {
            Some((config, _)) => {
                config.rules.push(auth);
            }
            None => {
                self.0.insert(
                    app.to_string(),
                    (config::App::new(vec![auth], None), EndpointBuilder::new()),
                );
            }
        };
        self
    }

    pub fn require_app_roles(mut self, app: &str, roles: Vec<String>) -> Self {
        match self.0.get_mut(app) {
            Some((config, _)) => {
                config.roles = Some(roles);
            }
            None => {
                self.0.insert(
                    app.to_string(),
                    (
                        config::App::new(vec![], Some(roles)),
                        EndpointBuilder::new(),
                    ),
                );
            }
        };
        self
    }

    pub fn set_app_endpoints(mut self, app: &str, endpoints: EndpointBuilder) -> Self {
        match self.0.get_mut(app) {
            Some((_, config)) => {
                *config = endpoints;
            }
            None => {
                self.0
                    .insert(app.to_string(), (config::App::new(vec![], None), endpoints));
            }
        };
        self
    }

    pub fn build(self) -> MiddlewareBuilder {
        let config: Config = self
            .0
            .into_iter()
            .map(|(app_name, (app, endpoints))| (app_name, (app, endpoints.0).into()))
            .collect::<HashMap<_, _>>()
            .into();
        MiddlewareBuilder::new(config)
    }
}

impl<EB: Into<EndpointBuilder>> From<HashMap<String, (config::App, EB)>> for Builder {
    fn from(auth: HashMap<String, (config::App, EB)>) -> Self {
        Self(
            auth.into_iter()
                .map(|(app, (root, endpoints))| (app, (root, endpoints.into())))
                .collect::<HashMap<_, _>>(),
        )
    }
}

#[derive(Debug, Default)]
pub struct EndpointBuilder(HashMap<String, config::Endpoint>);

impl EndpointBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_endpoint(mut self, endpoint: &str, root: config::Endpoint) -> Self {
        self.0.insert(endpoint.to_string(), root);
        self
    }
}

impl From<HashMap<String, config::Endpoint>> for EndpointBuilder {
    fn from(rules: HashMap<String, config::Endpoint>) -> Self {
        Self(rules)
    }
}

impl<I> FromIterator<(String, (config::App, I))> for Builder
where
    I: IntoIterator<Item = (String, config::Endpoint)>,
{
    fn from_iter<T: IntoIterator<Item = (String, (config::App, I))>>(iter: T) -> Self {
        Self(
            iter.into_iter()
                .map(|(key, value)| {
                    (
                        key,
                        (
                            value.0,
                            value.1.into_iter().collect::<HashMap<_, _>>().into(),
                        ),
                    )
                })
                .collect(),
        )
    }
}
