use crate::io::error::{error, ResponseStatusLine};

use super::{headers::HeaderMapExt, ReadHeaders, WriteHeaders};
use async_trait::async_trait;
use http::{header, HeaderMap, HeaderValue, StatusCode};
use tokio::io::{self, AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug)]
pub struct Response {
    pub version: String,
    pub status: StatusCode,
    pub forward_body: bool,
    headers: HeaderMap,
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status,
            headers: vec![(header::CONTENT_LENGTH, HeaderValue::from_static("0"))]
                .into_iter()
                .collect(),
            forward_body: false,
        }
    }

    pub fn error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl HeaderMapExt for Response {
    fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }
}

#[async_trait]
pub trait WriteResponse {
    async fn write_response(&mut self, response: &Response) -> io::Result<()>;
}

#[async_trait]
impl<R> WriteResponse for R
where
    R: AsyncWrite + ?Sized + Unpin + Send,
{
    async fn write_response(&mut self, response: &Response) -> io::Result<()> {
        self.write_all(response.version.as_bytes()).await?;
        self.write_all(b" ").await?;
        self.write_all(response.status.as_str().as_bytes()).await?;
        self.write_all(b" ").await?;
        if let Some(reason) = response.status.canonical_reason() {
            self.write_all(reason.as_bytes()).await?;
        }
        self.write_all(b"\r\n").await?;
        self.write_headers(&response.headers).await?;
        self.write_all(b"\r\n").await?;
        Ok(())
    }
}

#[async_trait]
pub trait ReadResponse {
    async fn read_response(&mut self) -> io::Result<Response>;
}

#[async_trait]
impl<R> ReadResponse for R
where
    R: AsyncBufRead + ?Sized + Unpin + Send,
{
    async fn read_response(&mut self) -> io::Result<Response> {
        let status_line = self
            .lines()
            .next_line()
            .await?
            .ok_or(error(ResponseStatusLine::MissingStatusLine))?;
        let (version, status) = {
            let mut parts = status_line.split_whitespace();
            (
                parts
                    .next()
                    .ok_or(error(ResponseStatusLine::MissingVersion))?
                    .to_string(),
                parts
                    .next()
                    .ok_or(error(ResponseStatusLine::MissingStatus))?
                    .to_string(),
            )
        };
        Ok(Response {
            version,
            status: status
                .parse()
                .map_err(|_| error(ResponseStatusLine::InvalidStatus))?,
            headers: self.read_headers().await?,
            forward_body: true,
        })
    }
}
