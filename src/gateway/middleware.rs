use std::collections::HashMap;

use super::{ctx::Ctx, next::Next, Result};
use crate::http::{Request, Response};
use async_trait::async_trait;

pub type Service = Box<dyn Middleware + Send + Sync + 'static>;

#[async_trait]
pub trait Middleware: Sync {
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    async fn run<'n>(&self, ctx: &Ctx, request: Request, next: Next<'n>) -> Result<Response>;
}

pub type MiddlewareBuilderService = Box<dyn MiddlewareBuilder + Send + Sync + 'static>;

#[async_trait]
pub trait MiddlewareBuilder: Sync {
    async fn build(
        self: Box<Self>,
        ids: &[String],
        routers: &HashMap<String, Vec<String>>,
    ) -> Result<Service>;
}
