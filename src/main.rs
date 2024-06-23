use anyhow::Context;
use async_trait::async_trait;
use essentials::info;
use gateway::{
    cors,
    http::{response::ResponseBody, HeaderMapExt, Request, Response},
    rate_limit, time, Middleware, Next, OriginServer, ParamRouter, Result, TcpOrigin,
};
use http::{header, Method, StatusCode};
use std::{collections::HashMap, env, net::SocketAddr, path::Path};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt},
    net::tcp::{OwnedReadHalf, OwnedWriteHalf},
};

struct Gateway;

#[async_trait]
impl Middleware for Gateway {
    async fn run<'n>(
        &self,
        _ctx: &gateway::Context,
        request: Request,
        next: Next<'n>,
    ) -> Result<Response> {
        println!("[gateway] before");
        let mut response = next.run(request).await?;
        println!("[gateway] after");
        response.insert_header("X-Server", "Rust");
        Ok(response)
    }
}

#[derive(Default)]
pub struct FileServer;

impl FileServer {
    pub fn new() -> Self {
        Self
    }
}

#[derive(Debug)]
pub struct FileResponse {
    file: File,
}

#[async_trait]
impl OriginServer for FileServer {
    async fn connect(
        &self,
        _context: &gateway::Context,
        request: Request,
        _left_rx: OwnedReadHalf,
        _left_remains: Vec<u8>,
    ) -> Result<Response> {
        println!("[origin] Request received: {:?}", request);
        let path = Path::new("static").join(&request.path.as_str()[1..]);
        if !path.exists() {
            return Ok(Response::new(StatusCode::NOT_FOUND));
        }
        let file = File::open(path)
            .await
            .with_context(|| format!("Failed to open file: {:?}", request.path))?;
        let length = file
            .metadata()
            .await
            .with_context(|| "Failed to read file metadata")?
            .len();
        let mut response = Response::new(StatusCode::OK);
        response.insert_header(header::CONTENT_LENGTH, length.to_string());
        response.set_body(FileResponse { file });
        println!("[origin] Returning file response");
        Ok(response)
    }
}

#[async_trait]
impl ResponseBody for FileResponse {
    async fn read_all(mut self: Box<Self>, len: usize) -> io::Result<String> {
        println!("reading all");
        let mut buf = vec![0; len];
        self.file.read_exact(&mut buf).await?;
        Ok(String::from_utf8(buf).unwrap())
    }

    async fn copy_to<'a>(&mut self, writer: &'a mut OwnedWriteHalf) -> io::Result<()> {
        println!("copying response to client");
        ::io::copy_file(&mut self.file, writer).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    println!("Starting gateway");
    env::set_var("APP_ENV", "d");
    env::set_var("RUST_LOG", "debug");
    env::set_var("RUST_BACKTRACE", "full");
    essentials::install();
    info!("Starting gateway");
    tokio::spawn(
        gateway::builder(FileServer::new(), |_| Some(String::new()))
            .register_peer(
                String::new(),
                ParamRouter::new().add_route(Method::GET, "/:file".to_string(), "file".to_string()),
            )
            .with_app_port(81)
            .with_health_check_port(9001)
            .build()
            .run(),
    );
    gateway::builder(
        TcpOrigin::new(HashMap::from([(
            String::new(),
            Box::new("127.0.0.1:81".parse::<SocketAddr>().unwrap()),
        )])),
        |_| Some(String::new()),
    )
    .register_peer(
        String::new(),
        ParamRouter::new().add_route(Method::GET, "/:hello".to_string(), "hello".to_string()),
    )
    .register_middleware(
        1,
        cors::Middleware::new(cors::Config {
            config: HashMap::from([(
                "app".to_string(),
                cors::AppConfig::new(vec![cors::Auth::new(
                    "token".to_string(),
                    vec!["http://localhost:3000".to_string()],
                )]),
            )]),
        }),
    )
    .register_middleware(
        2,
        rate_limit::Middleware::new(
            rate_limit::Config {
                config: HashMap::from([(
                    "app".to_string(),
                    rate_limit::AppConfig::new(
                        rate_limit::Rules {
                            quota: Some(rate_limit::Quota {
                                total: time::Frequency {
                                    amount: 5,
                                    interval: time::Time {
                                        amount: 1,
                                        unit: time::TimeUnit::Minutes,
                                    },
                                },
                                user: Some(time::Frequency {
                                    amount: 2,
                                    interval: time::Time {
                                        amount: 1,
                                        unit: time::TimeUnit::Minutes,
                                    },
                                }),
                            }),
                            endpoints: HashMap::new(),
                        },
                        HashMap::new(),
                    ),
                )]),
            },
            rate_limit::InMemoryDatastore::new(),
        ),
    )
    .register_middleware(usize::MAX, Gateway)
    .with_host("0.0.0.0".parse().unwrap())
    .with_app_port(80)
    .build()
    .run()
    .await;
    info!("Gateway stopped");
}
