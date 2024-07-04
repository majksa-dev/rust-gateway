use async_trait::async_trait;
use essentials::info;
use std::sync::Arc;
use tokio::net::TcpStream;

use crate::{http::stream::Split, EntryPoint, Handler};

pub struct EntryPointHandler {
    entrypoint: Arc<EntryPoint>,
}

impl EntryPointHandler {
    pub fn new(entrypoint: EntryPoint) -> Self {
        Self {
            entrypoint: Arc::new(entrypoint),
        }
    }
}

#[async_trait]
impl Handler for EntryPointHandler {
    async fn handle(&self, left: TcpStream) {
        let ip = left.peer_addr().ok();
        info!(ip = ?ip, "Connection received");
        let (left_rx, left_tx) = left.to_split();
        self.entrypoint.safe_handle(ip, left_rx, left_tx).await
    }
}
