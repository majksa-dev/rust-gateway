use super::{
    entrypoint::{EntryPoint, Middlewares},
    Result,
};
use crate::{
    http::{Request, Response},
    Ctx,
};
use tokio::net::tcp::OwnedReadHalf;

pub struct Next<'a> {
    pub entrypoint: &'a EntryPoint,
    pub context: &'a Ctx,
    pub left_rx: OwnedReadHalf,
    pub left_remains: Vec<u8>,
    pub it: Middlewares,
}

unsafe impl<'a> Send for Next<'a> {}
unsafe impl<'a> Sync for Next<'a> {}

impl<'a> Next<'a> {
    pub async fn run(self, request: Request) -> Result<Response> {
        self.entrypoint
            .next(
                self.context,
                request,
                self.left_rx,
                self.left_remains,
                self.it,
            )
            .await
    }
}
