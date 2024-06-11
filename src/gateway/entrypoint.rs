use super::{
    middleware::Context,
    next::Next,
    origin::{Origin, OriginResponse},
};
use crate::{
    gateway::Error,
    http::{server::Handler, ReadRequest, Request, Response, WriteResponse},
    io::WriteReader,
    server::app::GenerateKey,
    utils::AsyncAndThen,
    Service,
};
use async_trait::async_trait;
use http::StatusCode;
use std::result::Result as StdResult;
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
};
use tokio::{
    io::{self, AsyncWriteExt, BufReader, ReadHalf},
    net::TcpStream,
};

pub type Middlewares = VecDeque<Arc<Service>>;

pub struct EntryPointHandler(pub Arc<EntryPoint>);

#[async_trait]
impl Handler for EntryPointHandler {
    async fn handle(&self, left: TcpStream) {
        let (left_rx, mut left_tx) = io::split(left);
        match EntryPoint::handle(&self.0, left_rx).await {
            Ok((response, right_rx, right_remains)) => {
                if let Err(err) = left_tx
                    .write_response(&response)
                    .await
                    .async_and_then(|_| async {
                        if response.forward_body {
                            left_tx.write_all(right_remains.as_slice()).await
                        } else {
                            Ok(())
                        }
                    })
                    .await
                    .map(|_| left_tx.write_reader(right_rx))
                {
                    essentials::warn!("{}", err);
                }
            }
            Err(err) => {
                essentials::error!("{}", err);
                if let Err(err) = left_tx
                    .write_all("HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n".as_bytes())
                    .await
                {
                    essentials::warn!("{}", err);
                }
            }
        };
    }
}

pub struct EntryPoint {
    origin: Origin,
    generate_peer_key: Box<GenerateKey>,
    peers: HashMap<String, Box<GenerateKey>>,
    middlewares: Middlewares,
}

unsafe impl Sync for EntryPoint {}
unsafe impl Send for EntryPoint {}

type Result<T> = StdResult<T, Error>;

impl EntryPoint {
    pub fn new(
        origin: Origin,
        generate_peer_key: Box<GenerateKey>,
        peers: HashMap<String, Box<GenerateKey>>,
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
        entrypoint: Arc<Self>,
        context: Arc<Context>,
        request: Request,
        left_rx: ReadHalf<TcpStream>,
        left_remains: Vec<u8>,
        mut it: Middlewares,
    ) -> Result<(Response, OriginResponse, Vec<u8>)> {
        match it.pop_back() {
            Some(middleware) => {
                let right = Arc::new(Mutex::new(Option::<(OriginResponse, Vec<u8>)>::None));
                let next = Next {
                    entrypoint: entrypoint.clone(),
                    context: context.clone(),
                    left: left_rx,
                    left_remains,
                    right: right.clone(),
                    it,
                };
                let response = middleware.run(context, request, next).await?;
                let right = right
                    .lock()
                    .map_err(|_| Error::new("Mutex poisoned when returning right stream"))?
                    .take()
                    .ok_or(Error::new("Mutex poisoned when returning right stream"))?;
                Ok((response, Box::new(right.0), right.1))
            }
            None => {
                entrypoint
                    .origin
                    .connect(context, request, left_rx, left_remains)
                    .await
            }
        }
    }

    pub async fn handle(
        entrypoint: &Arc<Self>,
        mut left_rx: ReadHalf<TcpStream>,
    ) -> Result<(Response, OriginResponse, Vec<u8>)> {
        let mut request_reader = BufReader::new(&mut left_rx);
        let request = request_reader.read_request().await.map_err(Error::io)?;
        let app_id = match (entrypoint.generate_peer_key)(&request) {
            Some(app) => app,
            None => Err(Error::status(StatusCode::BAD_REQUEST))?,
        };
        let endpoint_id = match (entrypoint
            .peers
            .get(&app_id)
            .ok_or(Error::status(StatusCode::NOT_FOUND))?)(&request)
        {
            Some(endpoint_id) => endpoint_id,
            None => Err(Error::status(StatusCode::BAD_REQUEST))?,
        };
        let context = Context {
            app_id,
            endpoint_id,
        };
        let left_remains = request_reader.buffer().to_vec();
        let it = entrypoint.middlewares.clone();
        Self::next(
            entrypoint.clone(),
            Arc::new(context),
            request,
            left_rx,
            left_remains,
            it,
        )
        .await
    }
}
