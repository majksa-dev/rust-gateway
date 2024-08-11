use super::{response::OriginResponse, stream::Stream};
use crate::{
    http::{
        stream::{ReadHalf, Split},
        HeaderMapExt, ReadResponse, Request, Response,
    },
    Ctx, OriginServer, Result, WriteRequest,
};
use anyhow::Context;
use async_trait::async_trait;
use essentials::{debug, warn};
use http::{header, StatusCode};
use std::io;
#[cfg(feature = "tls")]
use tokio::io::AsyncReadExt;
use tokio::{io::AsyncWriteExt, net::TcpStream};
use tokio_rustls::{rustls::pki_types, TlsConnector};

pub struct Origin {
    pub(super) context: super::Context,
    pub(super) connector: TlsConnector,
}

#[async_trait]
impl OriginServer for Origin {
    async fn connect(
        &self,
        context: &Ctx,
        mut request: Request,
        left_rx: ReadHalf,
        left_remains: Vec<u8>,
    ) -> Result<Response> {
        let connection = match self.context.get(context.app_id) {
            Some(addr) => addr.global(),
            None => {
                return Ok(Response::new(StatusCode::NOT_FOUND));
            }
        };
        let right = TcpStream::connect(connection.addr.to_string())
            .await
            .with_context(|| "Failed to connect to origin".to_string())?;
        debug!("Connected to origin");
        if let Some(host) = connection.host.as_deref() {
            request.insert_header(header::HOST, host);
        }

        let right = if connection.tls {
            let domain = pki_types::ServerName::try_from(
                request
                    .header(header::HOST)
                    .and_then(|v| {
                        v.to_str()
                            .map_err(|err| warn!(?err, "Failed to convert header value to string"))
                            .ok()
                    })
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::InvalidInput, "Host header not found")
                    })?,
            )?
            .to_owned();
            Stream::Tls(self.connector.connect(domain, right).await?.into())
        } else {
            Stream::Tcp(right)
        };
        let (mut right_rx, mut right_tx) = right.to_split();
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
        if let Some(size) = request.get_content_length().map(|v| v - left_remains.len()) {
            if size > 0 {
                tokio::io::copy(&mut left_rx.take(size as u64), &mut right_tx).await?;
            }
        };
        debug!("Body sent to origin");
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
