use std::collections::HashMap;
use std::net::IpAddr;

use essentials::info;
use pingora::apps::HttpServerApp;
use pingora::upstreams::peer::HttpPeer;
use structopt::StructOpt;

use pingora::proxy::{http_proxy_service, HttpProxy, Session};
use pingora::server::configuration::Opt;
use pingora::server::Server;

use crate::gateway::{entrypoint::EntryPoint, middleware::Middleware};

use super::health_check::HealthCheck;

pub(crate) type GeneratePeerKey = dyn (Fn(&Session) -> String) + Send + Sync + 'static;

/// A builder for a server.
pub struct ServerBuilder<H: Send + Sync + 'static>
where
    HttpProxy<EntryPoint>: HttpServerApp,
    HttpProxy<H>: HttpServerApp,
{
    generate_peer_key: Box<GeneratePeerKey>,
    peers: HashMap<String, Box<HttpPeer>>,
    middlewares: HashMap<usize, Box<dyn Middleware + Send + Sync + 'static>>,
    health_check: H,
    host: IpAddr,
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
            generate_peer_key,
            peers: HashMap::new(),
            middlewares: HashMap::new(),
            health_check,
            host: IpAddr::from([127, 0, 0, 1]), // Default host (localhost)
            app_port: 80,
            health_check_port: 9000,
        }
    }

    /// Register a peer with the given key.
    pub fn register_peer(mut self, key: String, peer: Box<HttpPeer>) -> Self {
        self.peers.insert(key, peer);
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
    pub fn build(self) -> pingora::Result<Server> {
        let opt = Opt::from_args();
        let mut my_server = Server::new(Some(opt))?;
        my_server.bootstrap();

        {
            let gateway_entrypoint = EntryPoint::new(
                self.generate_peer_key,
                self.peers,
                self.middlewares.into_values().collect(),
            );
            let mut service = http_proxy_service(&my_server.configuration, gateway_entrypoint);
            let server = format!("{}:{}", self.host, self.app_port);
            service.add_tcp(server.as_str());
            info!("Listening on {}", server);
            my_server.add_service(service);
        }

        {
            let mut service = http_proxy_service(&my_server.configuration, self.health_check);
            let server = format!("{}:{}", self.host, self.health_check_port);
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
