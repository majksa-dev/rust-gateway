use crate::{
    http::{HeaderMapExt, Request, Response, WriteResponse},
    server::app::GenerateKey,
    utils::{Also, AsyncAndThen},
    Ctx, Id, Next, Origin, ReadRequest, RouterService, Service,
};
use anyhow::Result;
use essentials::{debug, error, info, warn};
use http::{header, StatusCode};
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::io::{self, AsyncWriteExt, BufReader};

use super::{Middlewares, MiddlewaresItem, ReadHalf, WriteHalf};

pub struct EntryPoint {
    origin: Origin,
    generate_peer_key: Box<GenerateKey>,
    peers: HashMap<String, (Id, RouterService)>,
    middlewares: Vec<MiddlewaresItem>,
}

unsafe impl Sync for EntryPoint {}
unsafe impl Send for EntryPoint {}

impl EntryPoint {
    pub fn new<M: IntoIterator<Item = Service>>(
        origin: Origin,
        generate_peer_key: Box<GenerateKey>,
        peers: HashMap<String, RouterService>,
        middlewares: M,
    ) -> Self {
        Self {
            origin,
            generate_peer_key,
            peers: peers
                .into_iter()
                .enumerate()
                .map(|(id, (k, v))| (k, (id as Id, v)))
                .collect(),
            middlewares: middlewares.into_iter().map(Arc::from).collect(),
        }
    }

    pub async fn next(
        &self,
        context: &Ctx,
        request: Request,
        left_rx: ReadHalf,
        left_remains: Vec<u8>,
        mut it: Middlewares<'_>,
    ) -> Result<Response> {
        match it.next() {
            Some(middleware) => {
                debug!(middleware = middleware.name(), request = ?request, "-->");
                let next = Next {
                    entrypoint: self,
                    context,
                    left_rx,
                    left_remains,
                    it: Box::new(it),
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

    pub(crate) async fn safe_handle(
        self: &Arc<EntryPoint>,
        ip: Option<SocketAddr>,
        rx: ReadHalf,
        mut tx: WriteHalf,
    ) {
        match self.handle(rx, &mut tx).await {
            Ok(_) => {
                info!(ip = ?ip, "Connection closed");
            }
            Err(error) => {
                error!("{}", error);
                if let Err(err) = tx
                    .write_all(
                        b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    )
                    .await
                {
                    warn!(ip = ?ip, "Failed to write response: {}", err);
                };
            }
        }
    }

    async fn handle(&self, mut left_rx: ReadHalf, left_tx: &mut WriteHalf) -> io::Result<()> {
        debug!(target: "entrypoint", stage = "request", "0 - init");
        let mut request_reader = BufReader::new(&mut left_rx);
        let request = request_reader.read_request().await?;
        debug!(target: "entrypoint", stage = "request", data = ?request, "1 - parsed request header");
        let left_remains = request_reader.buffer().to_vec();
        debug!(target: "entrypoint", stage = "request", data = ?left_remains, "2 - collected request body (remains from buffer)");
        match self.handle_request(request, left_rx, left_remains).await {
            Ok(mut response) => {
                response.insert_header(header::CONNECTION, "close");
                left_tx
                    .write_response(&response)
                    .await
                    .also(|_| debug!(target: "entrypoint", stage = "response", data = ?response, "3 - wrote response"))
                    .async_and_then(move |_| async move {
                        let length = response.get_content_length();
                        if let Some(mut body) = response.body() {
                            body.copy_to(left_tx, length)
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
        left_rx: ReadHalf,
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
        let (app_id, app) = match self.peers.get(&app_id) {
            Some(app) => app,
            None => {
                warn!("App ID could not be matched to an app");
                return Ok(Response::new(StatusCode::BAD_GATEWAY));
            }
        };
        let endpoint_id = match app.matches(&request) {
            Some(endpoint_id) => endpoint_id,
            None => {
                warn!("Request could not be matched to an endpoint ID");
                return Ok(Response::new(StatusCode::FORBIDDEN));
            }
        };
        debug!("Endpoint ID: {}", endpoint_id);
        let context = Ctx {
            app_id: *app_id,
            endpoint_id,
        };
        debug!("Context: {:?}", context);
        let it = Box::new(self.middlewares.iter().cloned());
        self.next(&context, request, left_rx, left_remains, it)
            .await
    }
}
