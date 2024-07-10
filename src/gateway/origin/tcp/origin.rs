use super::response::OriginResponse;
use crate::{
    http::{stream::ReadHalf, HeaderMapExt, ReadResponse, Request, Response},
    Ctx, OriginServer, Result, WriteRequest,
};
use anyhow::Context;
use async_trait::async_trait;
use essentials::{debug, error};
use http::StatusCode;
#[cfg(feature = "tls")]
use tokio::io::AsyncReadExt;
use tokio::{io::AsyncWriteExt, net::TcpStream, spawn};

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
        right_tx
            .flush()
            .await
            .with_context(|| "Failed to flush request to origin".to_string())?;
        debug!("Request sent to origin: {:?}", request);
        right_tx
            .write_all(left_remains.as_slice())
            .await
            .with_context(|| format!("Failed to send remains to origin: {:?}", left_remains))?;
        debug!("Remains sent to origin: {:?}", left_remains);
        match request.get_content_length().map(|v| v - left_remains.len()) {
            Some(size) => {
                #[cfg(not(feature = "tls"))]
                ::io::copy_tcp(&mut left_rx, &mut right_tx, Some(size)).await?;
                #[cfg(feature = "tls")]
                tokio::io::copy(&mut left_rx.take(size as u64), &mut right_tx).await?;
            }
            None => {
                spawn(async move {
                    if let Err(err) = tokio::io::copy(&mut left_rx, &mut right_tx).await {
                        error!(?err, "failed forwarding request body to origin");
                    }
                });
            }
        };
        debug!("Body sent to origin");
        right_rx.readable().await?;
        debug!("Origin response received");
        let (mut response, right_remains) = right_rx
            .read_response()
            .await
            .with_context(|| "Failed to read response from origin:")?;
        debug!("Response received from origin: {:?}", response);
        response.set_body(OriginResponse {
            remains: right_remains,
            reader: right_rx,
        });
        Ok(response)
    }
}
