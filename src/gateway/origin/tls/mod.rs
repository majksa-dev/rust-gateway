mod builder;
pub mod config;
mod context;
mod origin;
mod response;
mod stream;

use builder::TcpOriginBuilder;
use origin::Origin;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};

use crate::{MiddlewareConfig, MiddlewareCtx};
use std::{
    collections::HashMap,
    fs::File,
    io::{self, BufReader},
    path::Path,
};

type Context = MiddlewareCtx<context::Connection, ()>;
type Config = MiddlewareConfig<config::Connection, ()>;

#[derive(Debug)]
pub struct Builder {
    config: HashMap<String, config::Connection>,
    certs: RootCertStore,
}

impl Builder {
    pub fn new(config: HashMap<String, config::Connection>) -> Self {
        let mut certs = RootCertStore::empty();
        certs.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        Self { config, certs }
    }

    pub fn add_peer(mut self, app: &str, connection: config::Connection) -> Self {
        self.config.insert(app.to_string(), connection);
        self
    }

    pub fn add_ca_root<P: AsRef<Path>>(mut self, path: P) -> io::Result<Self> {
        let mut pem = BufReader::new(File::open(path)?);
        for cert in rustls_pemfile::certs(&mut pem) {
            self.certs
                .add(cert?)
                .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("{:?}", e)))?;
        }
        Ok(self)
    }

    pub fn build(self) -> TcpOriginBuilder {
        let config = self
            .config
            .into_iter()
            .map(|(app, config)| (app, (config, HashMap::new()).into()))
            .collect::<HashMap<_, _>>();
        let tls = ClientConfig::builder()
            .with_root_certificates(self.certs)
            .with_no_client_auth();
        TcpOriginBuilder::new(config, tls)
    }
}

impl From<HashMap<String, config::Connection>> for Builder {
    fn from(connections: HashMap<String, config::Connection>) -> Self {
        Self::new(connections)
    }
}

impl FromIterator<(String, config::Connection)> for Builder {
    fn from_iter<T: IntoIterator<Item = (String, config::Connection)>>(iter: T) -> Self {
        Self::from(iter.into_iter().collect::<HashMap<_, _>>())
    }
}
