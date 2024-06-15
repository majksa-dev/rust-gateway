use std::sync::Arc;

use async_trait::async_trait;
use tokio::{io::AsyncRead, net::tcp::OwnedReadHalf};

use crate::{
    http::{Request, Response},
    Context, Result,
};

mod tcp;

pub use tcp::TcpOrigin;

pub type Origin = Box<dyn OriginServer + Send + Sync + 'static>;
pub type OriginResponse = Box<dyn AsyncRead + Unpin + Send + 'static>;

#[async_trait]
pub trait OriginServer {
    async fn connect(
        &self,
        context: Arc<Context>,
        request: Request,
        left_rx: OwnedReadHalf,
        left_remains: Vec<u8>,
    ) -> Result<(Response, OriginResponse, Vec<u8>)>;
}
