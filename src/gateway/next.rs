use super::{
    entrypoint::{EntryPoint, Middlewares},
    origin::OriginResponse,
    LeftStream, Result,
};
use crate::{
    http::{Request, Response},
    Context, Error,
};
use std::sync::{Arc, Mutex};

type ResponseWriter = (OriginResponse, Vec<u8>);

pub struct Next {
    pub entrypoint: Arc<EntryPoint>,
    pub context: Arc<Context>,
    pub left: LeftStream,
    pub left_remains: Vec<u8>,
    pub right: Arc<Mutex<Option<ResponseWriter>>>,
    pub it: Middlewares,
}

unsafe impl Send for Next {}
unsafe impl Sync for Next {}

impl Next {
    pub async fn run(self, request: Request) -> Result<Response> {
        let (response, right_stream, right_remains) = EntryPoint::next(
            self.entrypoint.clone(),
            self.context,
            request,
            self.left,
            self.left_remains,
            self.it,
        )
        .await?;
        *self
            .right
            .lock()
            .map_err(|_| Error::new("Mutex poisoned when returning right stream"))? =
            Some((right_stream, right_remains));
        Ok(response)
    }
}
