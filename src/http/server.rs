use essentials::info;
use pingora::apps::HttpServerApp;
use structopt::StructOpt;

use pingora::proxy::{http_proxy_service, HttpProxy};
use pingora::server::configuration::Opt;
use pingora::server::Server;

use super::health_check::HealthCheck;

pub struct ServerBuilder<G, H>
where
    G: Send + Sync + 'static,
    HttpProxy<G>: HttpServerApp,
    H: Send + Sync + 'static,
    HttpProxy<H>: HttpServerApp,
{
    gateway: G,
    health_check: H,
    app_port: u16,
    health_check_port: u16,
}

impl<G, H> ServerBuilder<G, H>
where
    G: Send + Sync + 'static,
    HttpProxy<G>: HttpServerApp,
    H: Send + Sync + 'static,
    HttpProxy<H>: HttpServerApp,
{
    pub fn new(gateway: G, health_check: H) -> Self {
        Self {
            gateway,
            health_check,
            app_port: 8080,
            health_check_port: 8081,
        }
    }

    pub fn with_health_check(mut self, health_check: H) -> Self {
        self.health_check = health_check;
        self
    }

    pub fn with_app_port(mut self, port: u16) -> Self {
        self.app_port = port;
        self
    }

    pub fn with_health_check_port(mut self, port: u16) -> Self {
        self.health_check_port = port;
        self
    }

    pub fn build(self) -> pingora::Result<Server> {
        let opt = Opt::from_args();
        let mut my_server = Server::new(Some(opt))?;
        my_server.bootstrap();

        {
            let mut service = http_proxy_service(&my_server.configuration, self.gateway);
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

pub fn builder<G>(gateway: G) -> ServerBuilder<G, HealthCheck>
where
    G: Send + Sync + 'static,
    HttpProxy<G>: HttpServerApp,
{
    ServerBuilder::<G, HealthCheck>::new(gateway, HealthCheck)
}

pub fn builder_with_health_check<G, H>(gateway: G, health_check: H) -> ServerBuilder<G, H>
where
    G: Send + Sync + 'static,
    HttpProxy<G>: HttpServerApp,
    H: Send + Sync + 'static,
    HttpProxy<H>: HttpServerApp,
{
    ServerBuilder::new(gateway, health_check)
}
