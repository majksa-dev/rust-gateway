use crate::io::error::{error, RequestStatusLine};

use super::{headers::HeaderMapExt, ReadHeaders, WriteHeaders};
use async_trait::async_trait;
use http::{HeaderMap, Method};
use tokio::io::{self, AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

#[derive(Debug)]
pub struct Request {
    pub method: Method,
    pub path: String,
    pub version: String,
    headers: HeaderMap,
}

impl Request {
    pub fn new(&self, path: String, method: Method) -> Self {
        Self {
            path,
            method,
            version: "HTTP/1.1".to_string(),
            headers: HeaderMap::new(),
        }
    }
}

impl HeaderMapExt for Request {
    fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    fn headers_mut(&mut self) -> &mut HeaderMap {
        &mut self.headers
    }
}

#[async_trait]
pub trait WriteRequest {
    async fn write_request(&mut self, request: &Request) -> std::io::Result<()>;
}

#[async_trait]
impl<R> WriteRequest for R
where
    R: AsyncWrite + ?Sized + Unpin + Send,
{
    async fn write_request(&mut self, request: &Request) -> io::Result<()> {
        self.write_all(request.method.as_str().as_bytes()).await?;
        self.write_all(b" ").await?;
        self.write_all(request.path.as_bytes()).await?;
        self.write_all(b" ").await?;
        self.write_all(request.version.as_bytes()).await?;
        self.write_all(b"\r\n").await?;
        self.write_headers(&request.headers).await?;
        self.write_all(b"\r\n").await?;
        Ok(())
    }
}

#[async_trait]
pub trait ReadRequest {
    async fn read_request(&mut self) -> io::Result<Request>;
}

#[async_trait]
impl<R> ReadRequest for R
where
    R: AsyncBufRead + ?Sized + Unpin + Send,
{
    async fn read_request(&mut self) -> io::Result<Request> {
        let status_line = self
            .lines()
            .next_line()
            .await?
            .ok_or(error(RequestStatusLine::MissingStatusLine))?;
        let (method, path, version) = {
            let mut parts = status_line.split_whitespace();
            (
                parts
                    .next()
                    .ok_or(error(RequestStatusLine::MissingMethod))?
                    .parse()
                    .map_err(|_| error(RequestStatusLine::InvalidMethod))?,
                parts
                    .next()
                    .ok_or(error(RequestStatusLine::MissingPath))?
                    .to_string(),
                parts
                    .next()
                    .ok_or(error(RequestStatusLine::MissingVersion))?
                    .to_string(),
            )
        };
        Ok(Request {
            method,
            path,
            version,
            headers: self.read_headers().await?,
        })
    }
}
