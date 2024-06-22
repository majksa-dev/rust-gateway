use super::{next::Next, Result};
use crate::http::{Request, Response};
use async_trait::async_trait;

pub type Service = Box<dyn Middleware + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct Context {
    pub app_id: String,
    pub endpoint_id: String,
}

#[async_trait]
pub trait Middleware: Sync {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    async fn run<'n>(&self, ctx: &Context, request: Request, next: Next<'n>) -> Result<Response>;
}
