use crate::http::{response::ResponseBody, stream::WriteHalf};
use async_trait::async_trait;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};

use super::stream::ReadHalf;

#[derive(Debug)]
pub struct OriginResponse {
    pub remains: Box<[u8]>,
    pub reader: ReadHalf,
}

#[async_trait]
impl ResponseBody for OriginResponse {
    async fn read_all(mut self: Box<Self>, len: usize) -> io::Result<String> {
        let mut buf = String::with_capacity(len);
        let remains_len = self.remains.len();
        buf.push_str(
            std::str::from_utf8(&self.remains)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?,
        );
        unsafe {
            self.reader
                .read_exact(&mut buf.as_bytes_mut()[remains_len..])
                .await?
        };
        Ok(buf)
    }

    async fn copy_to<'a>(
        &mut self,
        writer: &'a mut WriteHalf,
        length: Option<usize>,
    ) -> io::Result<()> {
        if let Some(length) = length {
            if length == 0 {
                return Ok(());
            }
        }
        if length.is_some() {
            writer.write_all(&self.remains).await?;
            tokio::io::copy(&mut self.reader, writer).await?;
        }
        Ok(())
    }
}
