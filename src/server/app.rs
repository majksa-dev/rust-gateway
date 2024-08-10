use anyhow::Result;
use essentials::{debug, error};
use futures::future::join_all;
use tokio::sync::{mpsc, oneshot};
#[cfg(feature = "tls")]
use tokio_rustls::{rustls, TlsAcceptor};

use crate::gateway::entrypoint::{self, EntryPoint};
use crate::gateway::middleware::MiddlewareBuilderService;
use crate::gateway::router::{RouterBuilder, RouterBuilderService};
use crate::http::server::Server as HttpServer;
use crate::http::Request;
use crate::{MiddlewareBuilder, OriginBuilder, OriginServerBuilder};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};

use super::health_check::HealthCheck;

pub(crate) type GenerateKey =
    dyn (Fn(&Request) -> Option<(String, Option<String>)>) + Send + Sync + 'static;

/// A builder for a server.
pub struct ServerBuilder {
    origin: OriginBuilder,
    generate_peer_key: Box<GenerateKey>,
    peers: Vec<(String, RouterBuilderService)>,
    middlewares: HashMap<usize, MiddlewareBuilderService>,
    host: IpAddr,
    app_port: u16,
    #[cfg(feature = "tls")]
    app_tls_port: u16,
    #[cfg(feature = "tls")]
    tls_config: TlsAcceptor,
    health_check_port: u16,
}

impl ServerBuilder {
    fn new(generate_peer_key: Box<GenerateKey>, origin: OriginBuilder) -> Self {
        Self {
            origin,
            generate_peer_key,
            peers: Vec::new(),
            middlewares: HashMap::new(),
            host: IpAddr::from([127, 0, 0, 1]), // Default host (localhost)
            app_port: 80,
            #[cfg(feature = "tls")]
            app_tls_port: 443,
            #[cfg(feature = "tls")]
            tls_config: TlsAcceptor::from(std::sync::Arc::new(
                rustls::ServerConfig::builder()
                    .with_no_client_auth()
                    .with_cert_resolver(std::sync::Arc::new(entrypoint::tls::EmptyResolver::new())),
            )),
            health_check_port: 9000,
        }
    }

    /// Register a peer with the given key.
    pub fn register_peer(mut self, key: String, router: impl RouterBuilder + 'static) -> Self {
        self.peers.push((key, Box::new(router)));
        self
    }

    /// Register a middleware with the given priority.
    pub fn register_middleware<M: MiddlewareBuilder + Send + Sync + 'static>(
        mut self,
        priority: usize,
        middleware: M,
    ) -> Self {
        self.middlewares.insert(priority, Box::new(middleware));
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

    /// Set the TLS configuration for the application service.
    /// The default configuration is None
    /// The configuration is a tuple of the port and the TLS acceptor.
    /// The acceptor is used to create a TLS server.
    /// The server will listen on the specified port.
    #[cfg(feature = "tls")]
    pub fn with_tls(mut self, port: u16, acceptor: TlsAcceptor) -> Self {
        self.app_tls_port = port;
        self.tls_config = acceptor;
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
    pub async fn build(self) -> Result<Server> {
        let ids = self
            .peers
            .iter()
            .map(|(id, _)| id.clone())
            .collect::<Box<[_]>>();
        let (endpoints, peers) = self
            .peers
            .into_iter()
            .map(|(id, router)| (id, router.build()))
            .map(|(id, (ids, router))| ((id.clone(), ids), (id, router)))
            .collect::<(HashMap<_, _>, Vec<_>)>();
        let middlewares = join_all(
            self.middlewares
                .into_values()
                .map(|builder| builder.build(&ids, &endpoints)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;
        let entrypoint = EntryPoint::new(
            self.origin.build(&ids, &endpoints).await?,
            self.generate_peer_key,
            peers,
            middlewares,
        );
        #[cfg(feature = "tls")]
        let handler = entrypoint::tls::build(
            entrypoint,
            self.host,
            self.app_port,
            self.app_tls_port,
            self.tls_config,
        );
        #[cfg(not(feature = "tls"))]
        let handler = entrypoint::tcp::build(entrypoint, self.host, self.app_port);
        let server = Server {
            app: handler,
            health_check: HttpServer::new(
                SocketAddr::new(self.host, self.health_check_port),
                HealthCheck,
            ),
        };
        Ok(server)
    }
}

pub struct Server {
    #[cfg(feature = "tls")]
    pub app: entrypoint::tls::TlsServer,
    #[cfg(not(feature = "tls"))]
    pub app: entrypoint::tcp::TcpServer,
    pub health_check: HttpServer<HealthCheck>,
}

impl Server {
    /// Start the server.
    pub async fn run(self) {
        debug!("Starting server");
        let (tx_app, rx_app) = oneshot::channel();
        let (tx_health, rx_health) = oneshot::channel();
        let (tx, mut rx) = mpsc::channel(2);
        let tx_2 = tx.clone();
        tokio::spawn(async move {
            tokio::select! {
                result = self.app.run() => {
                    debug!("App stopped");
                    tx_health.send(()).unwrap();
                    if let Err(error) = result {
                        error!(?error, "App error");
                    }
                }
                _ = rx_app => {}
            }
            let _ = tx.send(()).await;
        });
        tokio::spawn(async move {
            tokio::select! {
                result = self.health_check.run() => {
                    debug!("health_check stopped");
                    tx_app.send(()).unwrap();
                    if let Err(error) = result {
                        error!(?error, "health_check error");
                    }
                }
                _ = rx_health => {}
            }
            let _ = tx_2.send(()).await;
        });
        rx.recv().await;
        debug!("Server stopped");
    }
}

/// Create a new server builder with a default health check.
pub fn builder<F, O>(origin: O, generate_peer_key: F) -> ServerBuilder
where
    F: Fn(&Request) -> Option<(String, Option<String>)> + Send + Sync + 'static,
    O: OriginServerBuilder + Send + Sync + 'static,
{
    ServerBuilder::new(Box::new(generate_peer_key), Box::new(origin))
}

#[cfg(all(test, not(feature = "tls")))]
mod tests {
    use async_trait::async_trait;
    use http::StatusCode;
    use pretty_assertions::assert_eq;
    use std::io;
    use testing_utils::{get_random_ports, surf};
    use tokio::{io::AsyncWriteExt, spawn};

    use crate::{
        gateway::origin::{OriginServer, OriginServerBuilder},
        http::{response::ResponseBody, HeaderMapExt},
        AnyRouterBuilder, Ctx, Origin, ReadHalf, Response, WriteHalf,
    };

    use super::*;

    #[derive(Debug)]
    pub struct StringBody(pub String);

    #[async_trait]
    impl ResponseBody for StringBody {
        async fn read_all(self: Box<Self>, _: usize) -> io::Result<String> {
            Ok(self.0)
        }

        async fn copy_to<'a>(
            &mut self,
            writer: &'a mut WriteHalf,
            _: Option<usize>,
        ) -> io::Result<()> {
            writer.write_all(self.0.as_bytes()).await?;
            Ok(())
        }
    }

    #[derive(Debug)]
    struct MockOriginServer;

    #[async_trait]
    impl OriginServer for MockOriginServer {
        async fn connect(
            &self,
            ctx: &Ctx,
            _: Request,
            _: ReadHalf,
            _: Vec<u8>,
        ) -> Result<Response> {
            let mut response = Response::new(StatusCode::OK);
            let app_id = ctx.app_id.to_string();
            response.insert_header("content-length", app_id.len().to_string().as_str());
            response.set_body(StringBody(app_id));
            Ok(response)
        }
    }

    #[derive(Debug)]
    struct MockOriginServerBuilder;

    #[async_trait]
    impl OriginServerBuilder for MockOriginServerBuilder {
        async fn build(
            self: Box<Self>,
            _: &[String],
            _: &HashMap<String, Vec<String>>,
        ) -> Result<Origin> {
            Ok(Box::new(MockOriginServer))
        }
    }

    #[tokio::test]
    async fn test_server_builder() {
        let ports = get_random_ports(2);
        let mut builder = ServerBuilder::new(
            Box::new(|req| {
                Some((
                    req.header("host").unwrap().to_str().unwrap().to_string(),
                    None,
                ))
            }),
            Box::new(MockOriginServerBuilder),
        )
        .with_app_port(ports[0])
        .with_health_check_port(ports[1]);
        for i in 0..100 {
            builder = builder.register_peer(format!("test{}", i), AnyRouterBuilder);
        }
        let server = builder.build().await.unwrap();
        spawn(server.run());
        wait_for_server(ports[1]).await;
        println!("Server started");

        for i in 0..100 {
            let mut response = surf::get(format!("http://127.0.0.1:{}", ports[0]))
                .header("host", format!("test{}", i))
                .await
                .expect("Failed to get response");
            assert_eq!(surf::StatusCode::Ok, response.status());
            assert_eq!(i.to_string(), response.body_string().await.unwrap());
        }
    }

    async fn wait_for_server(health_check: u16) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
        loop {
            if let Ok(response) = surf::get(format!("http://127.0.0.1:{}", health_check)).await {
                if response.status() == 200 {
                    break;
                }
            }
            interval.tick().await;
        }
    }
}
