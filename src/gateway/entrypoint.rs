use super::{middleware::Context, next::Next, origin::Origin, router::RouterService};
use crate::{
    http::{server::Handler, HeaderMapExt, Request, Response, WriteResponse},
    server::app::GenerateKey,
    utils::{Also, AsyncAndThen},
    ReadRequest, Service,
};
use anyhow::Result;
use async_trait::async_trait;
use essentials::{debug, error, info, warn};
use http::StatusCode;
use std::{
    collections::{HashMap, VecDeque},
    sync::Arc,
};
use tokio::{
    io::{self, AsyncWriteExt, BufReader},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
};

pub type Middlewares = VecDeque<Arc<Service>>;

pub struct EntryPointHandler(pub Arc<EntryPoint>);

#[async_trait]
impl Handler for EntryPointHandler {
    async fn handle(&self, left: TcpStream) {
        let ip = left.peer_addr().ok();
        info!(ip = ?ip, "Connection received");
        let (left_rx, mut left_tx) = left.into_split();
        match self.0.handle(left_rx, &mut left_tx).await {
            Ok(_) => {
                info!(ip = ?ip, "Connection closed");
            }
            Err(error) => {
                error!("{}", error);
                if let Err(err) = left_tx
                    .write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                    .await
                {
                    warn!(ip = ?ip, "Failed to write response: {}", err);
                };
            }
        }
    }
}

pub struct EntryPoint {
    origin: Origin,
    generate_peer_key: Box<GenerateKey>,
    peers: HashMap<String, RouterService>,
    middlewares: Middlewares,
}

unsafe impl Sync for EntryPoint {}
unsafe impl Send for EntryPoint {}

impl EntryPoint {
    pub fn new(
        origin: Origin,
        generate_peer_key: Box<GenerateKey>,
        peers: HashMap<String, RouterService>,
        middlewares: VecDeque<Service>,
    ) -> Self {
        Self {
            origin,
            generate_peer_key,
            peers,
            middlewares: middlewares.into_iter().map(Arc::new).collect(),
        }
    }

    pub async fn next(
        &self,
        context: &Context,
        request: Request,
        left_rx: OwnedReadHalf,
        left_remains: Vec<u8>,
        mut it: Middlewares,
    ) -> Result<Response> {
        match it.pop_front() {
            Some(middleware) => {
                debug!(middleware = middleware.name(), request = ?request, "-->");
                let next = Next {
                    entrypoint: self,
                    context,
                    left_rx,
                    left_remains,
                    it,
                };
                middleware
                    .run(context, request, next)
                    .await
                    .also(|r| debug!(middleware = middleware.name(), response = ?r, "<--"))
            }
            None => {
                debug!(origin = self.origin.name(), request = ?request, "-->");
                self.origin
                    .connect(context, request, left_rx, left_remains)
                    .await
                    .also(|r| debug!(origin = self.origin.name(), response = ?r, "<--"))
            }
        }
    }

    pub async fn handle(
        &self,
        mut left_rx: OwnedReadHalf,
        left_tx: &mut OwnedWriteHalf,
    ) -> io::Result<()> {
        debug!(target: "entrypoint", stage = "request", "0 - init");
        let mut request_reader = BufReader::new(&mut left_rx);
        let request = request_reader.read_request().await?;
        debug!(target: "entrypoint", stage = "request", data = ?request, "1 - parsed request header");
        let left_remains = request_reader.buffer().to_vec();
        debug!(target: "entrypoint", stage = "request", data = ?left_remains, "2 - collected request body (remains from buffer)");
        match self.handle_request(request, left_rx, left_remains).await {
            Ok(mut response) => {
                if response.insert_header("Connection", "close").is_none() {
                    warn!("Failed to insert header Connection: close");
                }
                left_tx
                    .write_response(&response)
                    .await
                    .also(|_| debug!(target: "entrypoint", stage = "response", data = ?response, "3 - wrote response"))
                    .async_and_then(move |_| async move {
                        if let Some(mut body) = response.body() {
                            body.copy_to(left_tx)
                                .await
                        } else {
                            Ok(())
                        }
                    })
                    .await
                    .also(|r| debug!(target: "entrypoint", stage = "response", data = ?r, "4 - wrote response body"))
            }
            Err(error) => {
                error!("{}", error);
                left_tx
                    .write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                    .await
            }
        }
    }

    async fn handle_request(
        &self,
        request: Request,
        left_rx: OwnedReadHalf,
        left_remains: Vec<u8>,
    ) -> Result<Response> {
        let app_id = match (self.generate_peer_key)(&request) {
            Some(app) => app,
            None => {
                warn!("Request could not be matched to an app ID");
                return Ok(Response::new(StatusCode::BAD_GATEWAY));
            }
        };
        debug!("App ID: {}", app_id);
        let app = match self.peers.get(&app_id) {
            Some(app) => app,
            None => {
                warn!("App ID could not be matched to an app");
                return Ok(Response::new(StatusCode::BAD_GATEWAY));
            }
        };
        let endpoint_id = match app.matches(&request) {
            Some(endpoint_id) => endpoint_id.clone(),
            None => {
                warn!("Request could not be matched to an endpoint ID");
                return Ok(Response::new(StatusCode::FORBIDDEN));
            }
        };
        debug!("Endpoint ID: {}", endpoint_id);
        let context = Context {
            app_id,
            endpoint_id,
        };
        debug!("Context: {:?}", context);
        let it = self.middlewares.clone();
        self.next(&context, request, left_rx, left_remains, it)
            .await
    }
}
