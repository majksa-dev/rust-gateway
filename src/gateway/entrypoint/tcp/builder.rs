use std::net::{IpAddr, SocketAddr};

use anyhow::Result;

use crate::{EntryPoint, HttpServer};

use super::handler::EntryPointHandler;

pub struct TcpServer {
    pub app: HttpServer<EntryPointHandler>,
}

impl TcpServer {
    pub async fn run(self) -> Result<()> {
        self.app.run().await
    }
}

pub fn build(entrypoint: EntryPoint, host: IpAddr, port: u16) -> TcpServer {
    let handler = EntryPointHandler::new(entrypoint);
    TcpServer {
        app: HttpServer::new(SocketAddr::new(host, port), handler),
    }
}
