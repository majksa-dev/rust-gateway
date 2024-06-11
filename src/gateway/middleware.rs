use std::sync::Arc;

use async_trait::async_trait;

use crate::http::{Request, Response};

use super::{next::Next, Result};

pub type Service = Box<dyn Middleware + Send + Sync + 'static>;

#[derive(Debug, Clone)]
pub struct Context {
    pub app_id: String,
    pub endpoint_id: String,
}

#[async_trait]
pub trait Middleware: Sync {
    async fn run(&self, ctx: Arc<Context>, request: Request, next: Next) -> Result<Response>;
}
