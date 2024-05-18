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

use super::middleware::{AnyContext, AnyMiddleware};

pub struct EntryPoint {
    generate_peer_key: Box<GeneratePeerKey>,
    peers: HashMap<String, Box<HttpPeer>>,
    middlewares: Vec<AnyMiddleware>,
}

impl EntryPoint {
    pub fn new(
        generate_peer_key: Box<GeneratePeerKey>,
        peers: HashMap<String, Box<HttpPeer>>,
        middlewares: Vec<AnyMiddleware>,
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
    middleware_contexts: Vec<AnyContext>,
    peer: Option<Box<HttpPeer>>,
}

impl Ctx {
    pub fn new(contexts: Vec<AnyContext>) -> Self {
        Ctx {
            middleware_contexts: contexts,
            ..Default::default()
        }
    }
}

unsafe impl Send for Ctx {}
unsafe impl Sync for Ctx {}

pub struct Context {
    pub id: String,
}

#[async_trait]
impl ProxyHttp for EntryPoint {
    type CTX = Ctx;

    fn new_ctx(&self) -> Self::CTX {
        Ctx::new(self.middlewares.iter().map(|m| m.new_ctx()).collect())
    }

    async fn request_filter(&self, session: &mut Session, ctx: &mut Self::CTX) -> Result<bool> {
        let context = Context {
            id: (self.generate_peer_key)(session),
        };
        for (controller, ctx) in self
            .middlewares
            .iter()
            .zip(ctx.middleware_contexts.iter_mut())
        {
            match controller.filter(session, (&context, ctx)).await {
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
        for (controller, ctx) in self
            .middlewares
            .iter()
            .zip(ctx.middleware_contexts.iter_mut())
        {
            if let Err(e) = controller
                .modify_request(session, request, (context, ctx))
                .await
            {
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
        for (controller, ctx) in self
            .middlewares
            .iter()
            .zip(ctx.middleware_contexts.iter_mut())
        {
            if let Err(e) = controller
                .modify_response(session, response, (context, ctx))
                .await
            {
                error!("modify response error: {:?}", e);
            }
        }
        Ok(())
    }
}
