use std::collections::HashMap;

use essentials::info;
use pingora::apps::HttpServerApp;
use pingora::upstreams::peer::HttpPeer;
use structopt::StructOpt;

use pingora::proxy::{http_proxy_service, HttpProxy};
use pingora::server::configuration::Opt;
use pingora::server::Server;

use crate::gateway::{entrypoint::EntryPoint, middleware::Middleware};

use super::health_check::HealthCheck;
use super::upstream_peer::{GeneratePeerKey, UpstreamPeerConnector};

/// A builder for a server.
pub struct ServerBuilder<H: Send + Sync + 'static>
where
    HttpProxy<EntryPoint>: HttpServerApp,
    HttpProxy<H>: HttpServerApp,
{
    peer_connector: UpstreamPeerConnector,
    middlewares: HashMap<usize, Box<dyn Middleware + Send + Sync + 'static>>,
    health_check: H,
    app_port: u16,
    health_check_port: u16,
}

impl<H: Send + Sync + 'static> ServerBuilder<H>
where
    HttpProxy<EntryPoint>: HttpServerApp,
    HttpProxy<H>: HttpServerApp,
{
    fn new(generate_peer_key: Box<GeneratePeerKey>, health_check: H) -> Self {
        Self {
            peer_connector: UpstreamPeerConnector::new(generate_peer_key),
            middlewares: HashMap::new(),
            health_check,
            app_port: 8080,
            health_check_port: 8081,
        }
    }

    /// Register a peer with the given key.
    pub fn register_peer(mut self, key: String, peer: Box<HttpPeer>) -> Self {
        self.peer_connector.register_peer(key, peer);
        self
    }

    /// Register a middleware with the given priority.
    pub fn register_middleware<M>(mut self, priority: usize, middleware: Box<M>) -> Self
    where
        M: Middleware + Send + Sync + 'static,
    {
        self.middlewares.insert(priority, middleware);
        self
    }

    /// Set the port for the application service.
    pub fn with_app_port(mut self, port: u16) -> Self {
        self.app_port = port;
        self
    }

    /// Set the port for the health check service.
    pub fn with_health_check_port(mut self, port: u16) -> Self {
        self.health_check_port = port;
        self
    }

    /// Build the server with the given configuration.
    /// The server will listen on the specified ports and will use the specified health check.
    /// The server will use the specified middlewares to filter and modify requests and responses.
    /// The server will use the specified peer connector to connect to upstream servers.
    /// The server will use the specified generate peer key function to generate keys for the peer connector.
    /// The server will use the specified health check to check the health of the server.
    pub fn build(self) -> pingora::Result<Server> {
        let opt = Opt::from_args();
        let mut my_server = Server::new(Some(opt))?;
        my_server.bootstrap();

        {
            let gateway_entrypoint = EntryPoint::new(
                self.peer_connector,
                self.middlewares.into_values().collect(),
            );
            let mut service = http_proxy_service(&my_server.configuration, gateway_entrypoint);
            let server = format!("127.0.0.1:{}", self.app_port);
            service.add_tcp(server.as_str());
            info!("Listening on {}", server);
            my_server.add_service(service);
        }

        {
            let mut service = http_proxy_service(&my_server.configuration, self.health_check);
            let server = format!("127.0.0.1:{}", self.health_check_port);
            service.add_tcp(server.as_str());
            info!("Healthcheck listening on {}", server);
            my_server.add_service(service);
        }
        Ok(my_server)
    }
}

/// Create a new server builder with a default health check.
/// The health check will return a 200 OK response for all requests.
pub fn builder(generate_peer_key: Box<GeneratePeerKey>) -> ServerBuilder<HealthCheck>
where
    HttpProxy<EntryPoint>: HttpServerApp,
{
    builder_with_health_check(generate_peer_key, HealthCheck)
}

/// Create a new server builder with a custom health check.
/// The health check must implement the [ProxyHttp](https://docs.rs/pingora/latest/pingora/proxy/trait.ProxyHttp.html) trait.
/// The health check service wrapped in [HttpProxy](https://docs.rs/pingora/latest/pingora/proxy/struct.HttpProxy.html) must implement the [HttpServerApp](https://docs.rs/pingora/latest/pingora/apps/trait.HttpServerApp.html) trait.
pub fn builder_with_health_check<H: Send + Sync + 'static>(
    generate_peer_key: Box<GeneratePeerKey>,
    health_check: H,
) -> ServerBuilder<H>
where
    HttpProxy<EntryPoint>: HttpServerApp,
    HttpProxy<H>: HttpServerApp,
{
    ServerBuilder::new(generate_peer_key, health_check)
}
