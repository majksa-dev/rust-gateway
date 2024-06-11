use async_trait::async_trait;
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::http::server::Handler;
pub struct HealthCheck;

#[async_trait]
impl Handler for HealthCheck {
    async fn handle(&self, mut stream: TcpStream) {
        if let Err(e) = stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .await
        {
            essentials::error!("Failed to write to stream: {:?}", e);
        }
    }
}
