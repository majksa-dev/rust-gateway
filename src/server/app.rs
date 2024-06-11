use tokio::sync::{mpsc, oneshot};

use crate::gateway::entrypoint::{EntryPoint, EntryPointHandler};
use crate::http::server::Server as HttpServer;
use crate::http::Request;
use crate::Service;
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use super::health_check::HealthCheck;

pub(crate) type GenerateKey = dyn (Fn(&Request) -> String) + Send + Sync + 'static;

/// A builder for a server.
pub struct ServerBuilder {
    generate_peer_key: Box<GenerateKey>,
    peers: HashMap<String, (Box<SocketAddr>, Box<GenerateKey>)>,
    middlewares: HashMap<usize, Service>,
    host: IpAddr,
    app_port: u16,
    health_check_port: u16,
}

impl ServerBuilder {
    fn new(generate_peer_key: Box<GenerateKey>) -> Self {
        Self {
            generate_peer_key,
            peers: HashMap::new(),
            middlewares: HashMap::new(),
            host: IpAddr::from([127, 0, 0, 1]), // Default host (localhost)
            app_port: 80,
            health_check_port: 9000,
        }
    }

    /// Register a peer with the given key.
    pub fn register_peer(
        mut self,
        key: String,
        peer: Box<SocketAddr>,
        endpoint_key_generator: Box<GenerateKey>,
    ) -> Self {
        self.peers.insert(key, (peer, endpoint_key_generator));
        self
    }

    /// Register a middleware with the given priority.
    pub fn register_middleware(mut self, priority: usize, middleware: Service) -> Self {
        self.middlewares.insert(priority, middleware);
        self
    }

    /// Set the host for the application service.
    /// The default host is 127.0.0.1
    pub fn with_host(mut self, host: IpAddr) -> Self {
        self.host = host;
        self
    }

    /// Set the port for the application service.
    /// The default port is 80
    pub fn with_app_port(mut self, port: u16) -> Self {
        self.app_port = port;
        self
    }

    /// Set the port for the health check service.
    /// The default port is 9000
    pub fn with_health_check_port(mut self, port: u16) -> Self {
        self.health_check_port = port;
        self
    }

    /// Build the server with the given configuration.
    /// The server will listen on the specified ports and will use the specified health check.
    pub fn build(self) -> Server {
        let handler = EntryPointHandler(Arc::new(EntryPoint::new(
            self.generate_peer_key,
            self.peers,
            self.middlewares.into_values().collect(),
        )));
        Server {
            app: HttpServer::new(SocketAddr::new(self.host, self.app_port), handler),
            health_check: HttpServer::new(
                SocketAddr::new(self.host, self.health_check_port),
                HealthCheck,
            ),
        }
    }
}

pub struct Server {
    pub app: HttpServer<EntryPointHandler>,
    pub health_check: HttpServer<HealthCheck>,
}

impl Server {
    /// Start the server.
    pub async fn run(self) {
        let (tx_app, rx_app) = oneshot::channel();
        let (tx_health, rx_health) = oneshot::channel();
        let (tx, mut rx) = mpsc::channel(2);
        let tx_2 = tx.clone();
        tokio::spawn(async move {
            tokio::select! {
                _ = self.app.run() => {
                    tx_health.send(()).unwrap();
                }
                _ = rx_app => {}
            }
            let _ = tx.send(()).await;
        });
        tokio::spawn(async move {
            tokio::select! {
                _ = self.health_check.run() => {
                    tx_app.send(()).unwrap();
                }
                _ = rx_health => {}
            }
            let _ = tx_2.send(()).await;
        });
        rx.recv().await;
    }
}

/// Create a new server builder with a default health check.
/// The health check will return a 200 OK response for all requests.
pub fn builder(generate_peer_key: Box<GenerateKey>) -> ServerBuilder {
    ServerBuilder::new(generate_peer_key)
}
