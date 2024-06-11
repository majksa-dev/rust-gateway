use async_trait::async_trait;
use http::{HeaderMap, HeaderName, HeaderValue};
use tokio::io::{self, AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

use crate::io::error::{error, Headers};

#[async_trait]
pub trait WriteHeaders {
    async fn write_headers(&mut self, headers: &HeaderMap) -> io::Result<()>;
}

#[async_trait]
impl<R> WriteHeaders for R
where
    R: AsyncWrite + ?Sized + Unpin + Send,
{
    async fn write_headers(&mut self, headers: &HeaderMap) -> io::Result<()> {
        for (key, value) in headers {
            self.write_all(key.as_str().as_bytes()).await?;
            self.write_all(b": ").await?;
            self.write_all(value.as_bytes()).await?;
            self.write_all(b"\r\n").await?;
        }
        Ok(())
    }
}

#[async_trait]
pub trait ReadHeaders {
    async fn read_headers(&mut self) -> std::io::Result<HeaderMap>;
}

#[async_trait]
impl<R> ReadHeaders for R
where
    R: AsyncBufRead + ?Sized + Unpin + Send,
{
    async fn read_headers(&mut self) -> std::io::Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        loop {
            let mut line = String::new();
            self.read_line(&mut line).await?;
            if line == "\r\n" {
                break;
            }
            if let Some(i) = line.find(':') {
                let (key, value) = line.split_at(i);
                headers.insert(
                    HeaderName::from_bytes(key.trim().as_bytes())
                        .map_err(Headers::InvalidName)
                        .map_err(error)?,
                    HeaderValue::from_str(value[1..].trim())
                        .map_err(Headers::InvalidValue)
                        .map_err(error)?,
                );
            }
        }
        Ok(headers)
    }
}
