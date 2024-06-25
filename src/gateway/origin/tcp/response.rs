use crate::http::response::ResponseBody;
use async_trait::async_trait;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
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

    async fn copy_to<'a>(&mut self, writer: &'a mut OwnedWriteHalf) -> io::Result<()> {
        writer.write_all(self.remains.as_slice()).await?;
        ::io::copy_tcp(&mut self.reader, writer).await?;
        Ok(())
    }
}
