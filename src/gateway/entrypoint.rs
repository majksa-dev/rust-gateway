use async_trait::async_trait;
use pingora::{
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
    Error, ErrorType, Result,
};

use crate::server::upstream_peer::UpstreamPeerConnector;

use super::middleware::Middleware;

pub struct EntryPoint {
    peer_connector: UpstreamPeerConnector,
    middlewares: Vec<Box<dyn Middleware + Send + Sync + 'static>>,
}

impl EntryPoint {
    pub fn new(
        peer_connector: UpstreamPeerConnector,
        middlewares: Vec<Box<dyn Middleware + Send + Sync + 'static>>,
    ) -> Self {
        Self {
            peer_connector,
            middlewares,
        }
    }
}

#[derive(Default)]
pub struct Context {
    peer: Option<Box<HttpPeer>>,
}

unsafe impl Send for Context {}
unsafe impl Sync for Context {}

#[async_trait]
impl ProxyHttp for EntryPoint {
    type CTX = Context;

    fn new_ctx(&self) -> Self::CTX {
        Context::default()
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        for controller in self.middlewares.iter() {
            if controller.request_filter(session).await? {
                return Ok(true);
            }
        }
        ctx.peer = self.peer_connector.get_peer(session);
        if ctx.peer.is_none() {
            session.respond_error(502).await;
            return Ok(true);
        }
        Ok(false)
    }

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        ctx.peer
            .take()
            .ok_or_else(|| Error::new(ErrorType::ConnectProxyFailure))
    }
}
