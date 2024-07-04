use async_trait::async_trait;
use essentials::{error, info, warn};
use http::header;
use std::{io, net::SocketAddr};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

use crate::{
    http::{stream::Split, HeaderMapExt},
    Handler, ReadRequest,
};

#[derive(Default)]
pub struct RedirectHandler;

impl RedirectHandler {
    pub fn new() -> Self {
        Self
    }

    async fn safe_handle(
        &self,
        ip: Option<SocketAddr>,
        left_rx: OwnedReadHalf,
        mut left_tx: OwnedWriteHalf,
    ) {
        match self.handle(left_rx, &mut left_tx).await {
            Ok(_) => {
                info!(ip = ?ip, "Connection closed");
            }
            Err(error) => {
                error!("{}", error);
                if let Err(err) = left_tx
                    .write_all(
                        b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .await
                {
                    warn!(ip = ?ip, "Failed to write response: {}", err);
                };
            }
        }
    }

    async fn handle(
        &self,
        mut left_rx: OwnedReadHalf,
        left_tx: &mut OwnedWriteHalf,
    ) -> io::Result<()> {
        let mut request_reader = BufReader::new(&mut left_rx);
        let request = request_reader.read_request().await?;
        let host = match request.header(header::HOST).and_then(|h| h.to_str().ok()) {
            Some(host) => host,
            None => {
                return left_tx
                    .write_all(
                        b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .await;
            }
        };
        left_tx
            .write_all(
                format!(
                    "HTTP/1.1 301 Moved Permanently\r\nLocation: https://{}{}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    host,
                    request.path
                )
                .as_bytes(),
            )
            .await?;
        Ok(())
    }
}

#[async_trait]
impl Handler for RedirectHandler {
    async fn handle(&self, left: TcpStream) {
        let ip = left.peer_addr().ok();
        info!(ip = ?ip, "Connection received");
        let (left_rx, left_tx) = left.to_split();
        self.safe_handle(ip, left_rx, left_tx).await
    }
}
