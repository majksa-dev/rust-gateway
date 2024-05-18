use async_trait::async_trait;
use essentials::error;
use pingora::{
    http::{RequestHeader, ResponseHeader},
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
            match controller.filter(session).await {
                Ok(false) => {
                    return Ok(true);
                }
                Ok(true) => {}
                Err(e) => {
                    error!("filter error: {:?}", e);
                }
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

    async fn upstream_request_filter(
        &self,
        session: &mut Session,
        request: &mut RequestHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        for controller in self.middlewares.iter() {
            if let Err(e) = controller.modify_request(session, request).await {
                error!("modify request error: {:?}", e);
            }
        }
        Ok(())
    }

    async fn response_filter(
        &self,
        session: &mut Session,
        response: &mut ResponseHeader,
        _ctx: &mut Self::CTX,
    ) -> Result<()> {
        for controller in self.middlewares.iter() {
            if let Err(e) = controller.modify_response(session, response).await {
                error!("modify response error: {:?}", e);
            }
        }
        Ok(())
    }
}
