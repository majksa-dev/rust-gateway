use async_trait::async_trait;
use essentials::{error, info};
use std::sync::Arc;
use tokio::net::TcpStream;
use tokio_rustls::TlsAcceptor;

use crate::{http::stream::Split, EntryPoint, Handler};

pub struct EntryPointHandler {
    entrypoint: Arc<EntryPoint>,
    acceptor: TlsAcceptor,
}

impl EntryPointHandler {
    pub fn new(entrypoint: EntryPoint, acceptor: TlsAcceptor) -> Self {
        Self {
            entrypoint: Arc::new(entrypoint),
            acceptor,
        }
    }
}

#[async_trait]
impl Handler for EntryPointHandler {
    async fn handle(&self, left: TcpStream) {
        let ip = left.peer_addr().ok();
        info!(ip = ?ip, "Connection received");
        let (left_rx, left_tx) = match self.acceptor.accept(left).await {
            Ok(stream) => stream.to_split(),
            Err(err) => {
                error!(ip = ?ip, "Failed to accept TLS connection: {}", err);
                return;
            }
        };
        self.entrypoint.safe_handle(ip, left_rx, left_tx).await
    }
}
