use super::response::OriginResponse;
use crate::{
    http::{stream::ReadHalf, HeaderMapExt, ReadResponse, Request, Response},
    Ctx, OriginServer, Result, WriteRequest,
};
use anyhow::Context;
use async_trait::async_trait;
use essentials::debug;
use http::StatusCode;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

pub struct Origin(pub super::Context);

#[async_trait]
impl OriginServer for Origin {
    async fn connect(
        &self,
        context: &Ctx,
        request: Request,
        mut left_rx: ReadHalf,
        left_remains: Vec<u8>,
    ) -> Result<Response> {
        let addr = match self.0.get(context.app_id) {
            Some(addr) => addr.global().clone(),
            None => {
                return Ok(Response::new(StatusCode::NOT_FOUND));
            }
        };
        let right = TcpStream::connect(addr.to_string())
            .await
            .with_context(|| "Failed to connect to origin".to_string())?;
        let (mut right_rx, mut right_tx) = right.into_split();
        debug!("Connected to origin");
        right_tx
            .write_request(&request)
            .await
            .with_context(|| format!("Failed to send request to origin: {:?}", request))?;
        debug!("Request sent to origin: {:?}", request);
        let request_body_size = request.get_content_length().map(|v| v - left_remains.len());
        right_tx
            .write_all(left_remains.as_slice())
            .await
            .with_context(|| format!("Failed to send remains to origin: {:?}", left_remains))?;
        debug!("Remains sent to origin: {:?}", left_remains);
        #[cfg(feature = "tls")]
        tokio::spawn(async move {
            if let Some(size) = request_body_size {
                tokio::io::copy(&mut left_rx.take(size as u64), &mut right_tx)
                    .await
                    .unwrap();
            } else {
                tokio::io::copy(&mut left_rx, &mut right_tx).await.unwrap();
            }
        });
        #[cfg(not(feature = "tls"))]
        tokio::spawn(async move {
            ::io::copy_tcp(&mut left_rx, &mut right_tx, request_size)
                .await
                .unwrap();
        });
        debug!("Body sent to origin");
        let mut response_reader = BufReader::new(&mut right_rx);
        let mut response = response_reader.read_response().await.with_context(|| {
            format!(
                "Failed to read response from origin: {:?}",
                response_reader.buffer()
            )
        })?;
        let right_remains = response_reader.buffer().to_vec();
        response.set_body(OriginResponse {
            remains: right_remains,
            reader: right_rx,
        });
        Ok(response)
    }
}
