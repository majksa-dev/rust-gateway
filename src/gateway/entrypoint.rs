use async_trait::async_trait;
use essentials::info;
use pingora::{
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
    Result,
};

use super::middleware::Middleware;

pub struct EntryPoint(pub Vec<Box<dyn Middleware + Send + Sync + 'static>>);

#[derive(Debug, Clone, Default)]
pub enum Phase {
    #[default]
    Init,
    Responded,
}

#[derive(Default)]
pub struct Context {
    pub phase: Phase,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

#[async_trait]
impl ProxyHttp for EntryPoint {
    type CTX = Context;

    fn new_ctx(&self) -> Self::CTX {
        info!("Creating new context");
        Context::default()
    }

    async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX) -> Result<bool> {
        for controller in self.0.iter() {
            if !controller.request_filter(session).await? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        todo!()
    }
}
