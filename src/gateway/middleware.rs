use async_trait::async_trait;
use pingora::{proxy::Session, Result};

#[async_trait]
pub trait Middleware {
    async fn request_filter(&self, session: &mut Session) -> Result<bool>;
}
