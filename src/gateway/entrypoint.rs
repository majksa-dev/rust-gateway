use std::collections::HashMap;

use async_trait::async_trait;
use essentials::error;
use pingora::{
    http::{RequestHeader, ResponseHeader},
    proxy::{ProxyHttp, Session},
    upstreams::peer::HttpPeer,
    Error, ErrorType, Result,
};

use crate::server::app::GeneratePeerKey;

use super::middleware::Middleware;

pub struct EntryPoint {
    generate_peer_key: Box<GeneratePeerKey>,
    peers: HashMap<String, Box<HttpPeer>>,
    middlewares: Vec<Box<dyn Middleware + Send + Sync + 'static>>,
}

impl EntryPoint {
    pub fn new(
        generate_peer_key: Box<GeneratePeerKey>,
        peers: HashMap<String, Box<HttpPeer>>,
        middlewares: Vec<Box<dyn Middleware + Send + Sync + 'static>>,
    ) -> Self {
        Self {
            generate_peer_key,
            peers,
            middlewares,
        }
    }
}

#[derive(Default)]
pub struct Ctx {
    context: Option<Context>,
    peer: Option<Box<HttpPeer>>,
}

unsafe impl Send for Ctx {}
unsafe impl Sync for Ctx {}

pub struct Context {
    id: String,
}

#[async_trait]
impl ProxyHttp for EntryPoint {
    type CTX = Ctx;

    fn new_ctx(&self) -> Self::CTX {
        Ctx::default()
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        let context = Context {
            id: (self.generate_peer_key)(session),
        };
        for controller in self.middlewares.iter() {
            match controller.filter(session, &context).await {
                Ok(Some(response)) => {
                    session.write_response_header(Box::new(response)).await?;
                    return Ok(true);
                }
                Ok(None) => {}
                Err(e) => {
                    error!("filter error: {:?}", e);
                }
            }
        }
        ctx.peer = self.peers.get(&context.id).cloned();
        ctx.context = Some(context);
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
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        let context = ctx.context.as_ref().unwrap();
        for controller in self.middlewares.iter() {
            if let Err(e) = controller.modify_request(session, request, context).await {
                error!("modify request error: {:?}", e);
            }
        }
        Ok(())
    }

    async fn response_filter(
        &self,
        session: &mut Session,
        response: &mut ResponseHeader,
        ctx: &mut Self::CTX,
    ) -> Result<()> {
        let context = ctx.context.as_ref().unwrap();
        for controller in self.middlewares.iter() {
            if let Err(e) = controller.modify_response(session, response, context).await {
                error!("modify response error: {:?}", e);
            }
        }
        Ok(())
    }
}
