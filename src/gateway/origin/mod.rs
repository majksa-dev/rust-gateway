use crate::{
    http::{Request, Response},
    Context, Result,
};
use async_trait::async_trait;
use std::io::Read;
use tokio::net::tcp::OwnedReadHalf;

pub use tcp::TcpOrigin;

mod tcp;

pub type Origin = Box<dyn OriginServer + Send + Sync + 'static>;
pub type OriginResponse = Box<dyn Read + Unpin + Send + 'static>;

#[async_trait]
pub trait OriginServer {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    async fn connect(
        &self,
        context: &Context,
        request: Request,
        left_rx: OwnedReadHalf,
        left_remains: Vec<u8>,
    ) -> Result<Response>;
}
