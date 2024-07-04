use async_trait::async_trait;
use tokio::io::{self, AsyncWriteExt};

use crate::http::{response::ResponseBody, stream::WriteHalf};

#[derive(Debug)]
pub struct CachedResponseBody {
    pub body: String,
}

impl CachedResponseBody {
    pub fn new(body: String) -> Self {
        Self { body }
    }
}

#[async_trait]
impl ResponseBody for CachedResponseBody {
    async fn read_all(self: Box<Self>, _len: usize) -> io::Result<String> {
        Ok(self.body)
    }

    async fn copy_to<'a>(&mut self, writer: &'a mut WriteHalf) -> io::Result<()> {
        writer.write_all(self.body.as_bytes()).await
    }
}
