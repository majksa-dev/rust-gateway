use super::{OriginResponse, OriginServer};
use crate::{
    gateway::LeftStream,
    http::{ReadResponse, Request, Response, WriteRequest},
    io::WriteReader,
    Context, Error, Result,
};
use async_trait::async_trait;
use http::StatusCode;
use std::{
    collections::HashMap,
    io,
    net::SocketAddr,
    os::fd::{AsRawFd, FromRawFd},
    sync::Arc,
};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

pub struct TcpOrigin(HashMap<String, Box<SocketAddr>>);

impl TcpOrigin {
    pub fn new(config: HashMap<String, Box<SocketAddr>>) -> Self {
        Self(config)
    }
}

#[async_trait]
impl OriginServer for TcpOrigin {
    async fn connect(
        &self,
        context: Arc<Context>,
        request: Request,
        left_rx: LeftStream,
        left_remains: Vec<u8>,
    ) -> Result<(Response, OriginResponse, Vec<u8>)> {
        let addr = match self.0.get(&context.app_id) {
            Some(addr) => addr.to_string(),
            None => {
                return Ok((
                    Response::new(StatusCode::NOT_FOUND),
                    Box::new(io::empty()),
                    Vec::new(),
                ));
            }
        };
        let right = TcpStream::connect(addr).await.map_err(Error::io)?;
        let right_std = unsafe { std::net::TcpStream::from_raw_fd(right.as_raw_fd()) };
        let (mut right_rx, mut right_tx) = right.into_split();
        right_tx.write_request(&request).await.map_err(Error::io)?;
        right_tx
            .write_all(left_remains.as_slice())
            .await
            .map_err(Error::io)?;
        right_std.try_clone().unwrap().write_reader(left_rx.0);
        let mut response_reader = BufReader::new(&mut right_rx);
        let response = response_reader.read_response().await.map_err(Error::io)?;
        let right_remains = response_reader.buffer().to_vec();
        Ok((response, Box::new(right_std), right_remains))
    }
}
