use std::net::{IpAddr, SocketAddr};

use tokio::sync::{mpsc, oneshot};
use tokio_rustls::TlsAcceptor;

use crate::{EntryPoint, HttpServer};

use super::{handler::EntryPointHandler, redirect::RedirectHandler};

pub struct TlsServer {
    pub tls: HttpServer<EntryPointHandler>,
    pub tcp: HttpServer<RedirectHandler>,
}

impl TlsServer {
    /// Start the server.
    pub async fn run(self) -> std::io::Result<()> {
        let (tx_tls, rx_tls) = oneshot::channel();
        let (tx_tcp, rx_tcp) = oneshot::channel();
        let (tx, mut rx) = mpsc::channel(2);
        let tx_2 = tx.clone();
        tokio::spawn(async move {
            tokio::select! {
                result = self.tls.run() => {
                    tx_tcp.send(()).unwrap();
                    result.unwrap();
                }
                _ = rx_tls => {}
            }
            let _ = tx.send(()).await;
        });
        tokio::spawn(async move {
            tokio::select! {
                result = self.tcp.run() => {
                    tx_tls.send(()).unwrap();
                    result.unwrap();
                }
                _ = rx_tcp => {}
            }
            let _ = tx_2.send(()).await;
        });
        rx.recv().await;
        Ok(())
    }
}

pub fn build(
    entrypoint: EntryPoint,
    host: IpAddr,
    http_port: u16,
    https_port: u16,
    acceptor: TlsAcceptor,
) -> TlsServer {
    TlsServer {
        tls: HttpServer::new(
            SocketAddr::new(host, https_port),
            EntryPointHandler::new(entrypoint, acceptor),
        ),
        tcp: HttpServer::new(SocketAddr::new(host, http_port), RedirectHandler::new()),
    }
}
