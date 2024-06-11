use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    task::JoinHandle,
};

pub trait WriteReader {
    fn write_reader<R>(self, reader: R) -> JoinHandle<()>
    where
        R: AsyncRead + Unpin + Send + 'static;
}

impl<W> WriteReader for W
where
    W: AsyncWrite + Unpin + Send + 'static,
{
    fn write_reader<R>(mut self, mut reader: R) -> JoinHandle<()>
    where
        R: AsyncRead + Unpin + Send + 'static,
    {
        tokio::spawn(async move {
            if let Err(err) = io::copy(&mut reader, &mut self).await {
                essentials::warn!("Failed to copy stream: {:?}", err);
            }
        })
    }
}
