use crate::http::{response::ResponseBody, stream::WriteHalf};
use async_trait::async_trait;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::tcp::OwnedReadHalf,
};

#[derive(Debug)]
pub struct OriginResponse {
    pub remains: Vec<u8>,
    pub reader: OwnedReadHalf,
}

#[async_trait]
impl ResponseBody for OriginResponse {
    async fn read_all(mut self: Box<Self>, len: usize) -> io::Result<String> {
        let mut buf = String::with_capacity(len);
        let remains_len = self.remains.len();
        buf.push_str(String::from_utf8(self.remains).unwrap().as_str());
        unsafe {
            self.reader
                .read_exact(&mut buf.as_bytes_mut()[remains_len..])
                .await?
        };
        Ok(buf)
    }

    async fn copy_to<'a>(&mut self, writer: &'a mut WriteHalf) -> io::Result<()> {
        writer.write_all(self.remains.as_slice()).await?;
        #[cfg(feature = "tls")]
        tokio::io::copy(&mut self.reader, writer).await?;
        #[cfg(not(feature = "tls"))]
        ::io::copy_tcp(&mut self.reader, writer, None).await?;
        Ok(())
    }
}
