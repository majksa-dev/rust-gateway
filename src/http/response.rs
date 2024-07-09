use super::{headers::HeaderMapExt, stream::WriteHalf, WriteHeaders};
use crate::io::error::{error, ResponseStatusLine};
use async_trait::async_trait;
use essentials::debug;
use http::{header, HeaderMap, HeaderValue, StatusCode};
use std::{fmt::Debug, io::ErrorKind, time::Duration};
use tokio::{
    io::{self, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    time::sleep,
};

#[derive(Debug)]
pub struct Response {
    pub version: String,
    pub status: StatusCode,
    headers: HeaderMap,
    body: Option<Box<dyn ResponseBody + Send + Sync + 'static>>,
}

#[async_trait]
pub trait ResponseBody: Debug {
    async fn read_all(self: Box<Self>, len: usize) -> io::Result<String>;

    async fn copy_to<'a>(
        &mut self,
        writer: &'a mut WriteHalf,
        length: Option<usize>,
    ) -> io::Result<()>;
}

impl Response {
    pub fn new(status: StatusCode) -> Self {
        Self {
            version: "HTTP/1.1".to_string(),
            status,
            headers: vec![(header::CONTENT_LENGTH, HeaderValue::from_static("0"))]
                .into_iter()
                .collect(),
            body: None,
        }
    }

    pub fn error() -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR)
    }

    pub fn set_body<B>(&mut self, body: B)
    where
        B: ResponseBody + Send + Sync + 'static,
    {
        self.body = Some(Box::new(body));
    }

    pub fn body(self) -> Option<Box<dyn ResponseBody + Send + Sync + 'static>> {
        self.body
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
    async fn read_response(&mut self) -> io::Result<(Response, Box<[u8]>)>;
}

#[async_trait]
impl<R> ReadResponse for R
where
    R: AsyncRead + ?Sized + Unpin + Send,
{
    async fn read_response(&mut self) -> io::Result<(Response, Box<[u8]>)> {
        let mut buf = [0_u8; 256];
        let mut timeout = Duration::from_micros(100);
        let mut line = String::new();
        let mut response = Option::<Response>::None;
        loop {
            let read = match self.read(&mut buf).await {
                Ok(n) => Ok(n),
                Err(e) if e.kind() == ErrorKind::WouldBlock => Ok(0),
                Err(e) => Err(e),
            }?;
            if read == 0 {
                debug!(?response, ?timeout, line, "sleeping");
                sleep(timeout).await;
                timeout *= 2;
                continue;
            }
            let mut it = buf.iter().copied().peekable();
            while let Some(c) = it.next() {
                if c == b'\r' && it.peek().is_some_and(|c| *c == b'\n') {
                    it.next();
                    if line.is_empty() {
                        if let Some(response) = response {
                            return Ok((response, it.collect()));
                        } else {
                            return Err(io::Error::new(
                                ErrorKind::InvalidData,
                                "response line is empty",
                            ));
                        }
                    }
                    if let Some(response) = response.as_mut() {
                        response.parse_header(line)?;
                        line = String::new();
                    } else {
                        let mut parts = line.split_whitespace();
                        response = Some(Response {
                            version: parts
                                .next()
                                .ok_or(error(ResponseStatusLine::MissingVersion))?
                                .to_string(),
                            status: parts
                                .next()
                                .ok_or(error(ResponseStatusLine::MissingStatus))?
                                .to_string()
                                .parse()
                                .map_err(|_| error(ResponseStatusLine::InvalidStatus))?,
                            headers: HeaderMap::new(),
                            body: None,
                        });
                        line = String::new();
                    }
                } else {
                    line.push(c as char);
                }
            }
        }
    }
}
