use crate::{
    http::{Request, Response},
    Ctx, Result,
};
use async_trait::async_trait;
use std::{collections::HashMap, io::Read};
use tokio::net::tcp::OwnedReadHalf;

pub mod tcp;

pub type Origin = Box<dyn OriginServer + Send + Sync + 'static>;
pub type OriginResponse = Box<dyn Read + Unpin + Send + 'static>;

#[async_trait]
pub trait OriginServer {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    async fn connect(
        &self,
        context: &Ctx,
        request: Request,
        left_rx: OwnedReadHalf,
        left_remains: Vec<u8>,
    ) -> Result<Response>;
}

pub type OriginBuilder = Box<dyn OriginServerBuilder + Send + Sync + 'static>;

#[async_trait]
pub trait OriginServerBuilder: Sync {
    async fn build(
        self: Box<Self>,
        ids: &[String],
        routers: &HashMap<String, Vec<String>>,
    ) -> Result<Origin>;
}
