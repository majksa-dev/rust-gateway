use std::io::{Read, Write};

use tokio::task::JoinHandle;

pub trait WriteReader {
    fn write_reader<R>(self, reader: R) -> JoinHandle<()>
    where
        R: Read + Unpin + Send + 'static;
}

impl<W> WriteReader for W
where
    W: Write + Unpin + Send + 'static,
{
    fn write_reader<R>(mut self, mut reader: R) -> JoinHandle<()>
    where
        R: Read + Unpin + Send + 'static,
    {
        tokio::spawn(async move {
            if let Err(err) = std::io::copy(&mut reader, &mut self) {
                essentials::warn!("Failed to copy stream: {:?}", err);
            }
        })
    }
}
