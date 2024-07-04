mod builder;
pub mod config;
mod origin;
mod response;

use builder::TcpOriginBuilder;
use origin::Origin;

use crate::{MiddlewareConfig, MiddlewareCtx};
use std::collections::HashMap;

type Context = MiddlewareCtx<Box<str>, ()>;
type Config = MiddlewareConfig<config::Connection, ()>;

#[derive(Debug, Default)]
pub struct Builder(HashMap<String, config::Connection>);

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_peer(mut self, app: &str, connection: config::Connection) -> Self {
        self.0.insert(app.to_string(), connection);
        self
    }

    pub fn build(self) -> TcpOriginBuilder {
        let config: Config = self
            .0
            .into_iter()
            .map(|(app, config)| (app, (config, HashMap::new()).into()))
            .collect::<HashMap<_, _>>()
            .into();
        TcpOriginBuilder::new(config)
    }
}

impl From<HashMap<String, config::Connection>> for Builder {
    fn from(connections: HashMap<String, config::Connection>) -> Self {
        Self(connections)
    }
}

impl FromIterator<(String, config::Connection)> for Builder {
    fn from_iter<T: IntoIterator<Item = (String, config::Connection)>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}
