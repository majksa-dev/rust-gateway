use async_trait::async_trait;
use std::{net::SocketAddr, sync::Arc};
use tokio::{
    io::Result,
    net::{TcpListener, TcpStream},
};

#[async_trait]
pub trait Handler {
    async fn handle(&self, stream: TcpStream);
}

pub struct Server<H: Handler + Send + Sync + 'static> {
    addr: SocketAddr,
    handler: Arc<H>,
}

impl<H: Handler + Send + Sync + 'static> Server<H> {
    pub fn new(addr: SocketAddr, handler: H) -> Self {
        Self {
            addr,
            handler: Arc::new(handler),
        }
    }

    pub async fn run(self) -> Result<()> {
        let listener = TcpListener::bind(self.addr).await?;
        loop {
            let (stream, _) = listener.accept().await?;
            let handler = self.handler.clone();
            tokio::spawn(async move {
                handler.handle(stream).await;
            });
        }
    }
}
