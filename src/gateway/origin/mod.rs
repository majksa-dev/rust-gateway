use super::LeftStream;
use crate::{
    http::{Request, Response},
    Context, Result,
};
use async_trait::async_trait;
use std::{io::Read, sync::Arc};

pub use tcp::TcpOrigin;

mod tcp;

pub type Origin = Box<dyn OriginServer + Send + Sync + 'static>;
pub type OriginResponse = Box<dyn Read + Unpin + Send + 'static>;

#[async_trait]
pub trait OriginServer {
    async fn connect(
        &self,
        context: Arc<Context>,
        request: Request,
        left_rx: LeftStream,
        left_remains: Vec<u8>,
    ) -> Result<(Response, OriginResponse, Vec<u8>)>;
}
