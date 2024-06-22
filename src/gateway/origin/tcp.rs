use super::OriginServer;
use crate::{
    http::{response::ResponseBody, ReadResponse, Request, Response, WriteRequest},
    Context, Error, Result,
};
use async_trait::async_trait;
use essentials::debug;
use http::StatusCode;
use std::{collections::HashMap, net::SocketAddr};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

pub struct TcpOrigin(HashMap<String, Box<SocketAddr>>);

impl TcpOrigin {
    pub fn new(config: HashMap<String, Box<SocketAddr>>) -> Self {
        Self(config)
    }
}

#[derive(Debug)]
pub struct OriginResponse {
    remains: Vec<u8>,
    reader: OwnedReadHalf,
}

#[async_trait]
impl ResponseBody for OriginResponse {
    async fn read_all(mut self: Box<Self>) -> io::Result<String> {
        let mut buf = String::from_utf8(self.remains)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        self.reader.read_to_string(&mut buf).await?;
        Ok(buf)
    }

    async fn copy_to<'a>(&mut self, writer: &'a mut OwnedWriteHalf) -> io::Result<()> {
        writer.write_all(self.remains.as_slice()).await?;
        ::io::copy_tcp(&mut self.reader, writer).await?;
        Ok(())
    }
}

#[async_trait]
impl OriginServer for TcpOrigin {
    async fn connect(
        &self,
        context: &Context,
        request: Request,
        mut left_rx: OwnedReadHalf,
        left_remains: Vec<u8>,
    ) -> Result<Response> {
        let addr = match self.0.get(&context.app_id) {
            Some(addr) => addr.to_string(),
            None => {
                return Ok(Response::new(StatusCode::NOT_FOUND));
            }
        };
        let right = TcpStream::connect(addr).await.map_err(Error::io)?;
        let (mut right_rx, mut right_tx) = right.into_split();
        debug!("Connected to origin");
        right_tx.write_request(&request).await.map_err(Error::io)?;
        debug!("Request sent to origin: {:?}", request);
        right_tx
            .write_all(left_remains.as_slice())
            .await
            .map_err(Error::io)?;
        debug!("Remains sent to origin: {:?}", left_remains);
        tokio::spawn(async move {
            ::io::copy_tcp(&mut left_rx, &mut right_tx).await.unwrap();
        });
        debug!("Body sent to origin");
        let mut response_reader = BufReader::new(&mut right_rx);
        let mut response = response_reader.read_response().await.map_err(Error::io)?;
        let right_remains = response_reader.buffer().to_vec();
        response.set_body(OriginResponse {
            remains: right_remains,
            reader: right_rx,
        });
        Ok(response)
    }
}
