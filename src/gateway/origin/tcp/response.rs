use crate::http::{response::ResponseBody, stream::WriteHalf};
use async_trait::async_trait;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::tcp::OwnedReadHalf,
};

#[derive(Debug)]
pub struct OriginResponse {
    pub remains: Box<[u8]>,
    pub reader: OwnedReadHalf,
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
        #[cfg(not(feature = "tls"))] length: Option<usize>,
        #[cfg(feature = "tls")] length: Option<usize>,
    ) -> io::Result<()> {
        if let Some(length) = length {
            if length == 0 {
                return Ok(());
            }
        }
        writer.write_all(&self.remains).await?;
        #[cfg(feature = "tls")]
        tokio::io::copy(&mut self.reader, writer).await?;
        #[cfg(not(feature = "tls"))]
        ::io::copy_tcp(&mut self.reader, writer, length).await?;
        Ok(())
    }
}
