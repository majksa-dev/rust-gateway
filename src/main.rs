use anyhow::Context;
use async_trait::async_trait;
use essentials::info;
use gateway::{
    http::{response::ResponseBody, HeaderMapExt, Request, Response},
    tcp, Ctx, Middleware, MiddlewareBuilder, Next, Origin, OriginServer, OriginServerBuilder,
    ParamRouterBuilder, ReadHalf, Result, Service, WriteHalf,
};
use http::{header, Method, StatusCode};
use std::{collections::HashMap, env, path::Path};
use tokio::{
    fs::File,
    io::{self, AsyncReadExt},
};

struct Gateway;

#[async_trait]
impl Middleware for Gateway {
    async fn run(&self, _ctx: &Ctx, request: Request, next: Next<'_>) -> Result<Response> {
        println!("[gateway] before");
        let mut response = next.run(request).await?;
        println!("[gateway] after");
        response.insert_header("X-Server", "Rust");
        Ok(response)
    }
}

struct GatewayBuilder;

impl GatewayBuilder {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl MiddlewareBuilder for GatewayBuilder {
    async fn build(
        self: Box<Self>,
        _: &[String],
        _: &HashMap<String, Vec<String>>,
    ) -> Result<Service> {
        Ok(Box::new(Gateway))
    }
}

#[derive(Default)]
struct FileServer;

#[derive(Default)]
pub struct FileServerBuilder;

impl FileServerBuilder {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl OriginServerBuilder for FileServerBuilder {
    async fn build(
        self: Box<Self>,
        _: &[String],
        _: &HashMap<String, Vec<String>>,
    ) -> Result<Origin> {
        Ok(Box::new(FileServer))
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
        _ctx: &Ctx,
        request: Request,
        _left_rx: ReadHalf,
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

    async fn copy_to<'a>(
        &mut self,
        writer: &'a mut WriteHalf,
        _length: Option<usize>,
    ) -> io::Result<()> {
        println!("copying response to client");
        tokio::io::copy(&mut self.file, writer).await?;
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
        gateway::builder(FileServerBuilder::new(), |_| Some((String::new(), None)))
            .register_peer(
                String::new(),
                ParamRouterBuilder::new().add_route(
                    Method::GET,
                    "/:file".to_string(),
                    "file".to_string(),
                ),
            )
            .with_app_port(81)
            .with_health_check_port(9001)
            .build()
            .await
            .unwrap()
            .run(),
    );
    let mut server_builder = gateway::builder(
        tcp::Builder::new()
            .add_peer(
                "",
                tcp::config::Connection::new("127.0.0.1:81".parse().unwrap()),
            )
            .build(),
        |_| Some((String::new(), None)),
    );
    server_builder = server_builder.register_peer(
        String::new(),
        ParamRouterBuilder::new().add_route(
            Method::GET,
            "/:hello".to_string(),
            "hello".to_string(),
        ),
    );
    #[cfg(feature = "cors")]
    {
        use gateway::cors;
        server_builder = server_builder.register_middleware(
            1,
            cors::Builder::new()
                .add_auth(
                    "app",
                    "token".to_string(),
                    vec!["http://localhost:3000".to_string()],
                )
                .build(),
        );
    }
    #[cfg(feature = "rate-limit")]
    {
        use gateway::{rate_limit, time};
        server_builder = server_builder.register_middleware(
            2,
            rate_limit::Builder::new()
                .add_app(
                    "app",
                    rate_limit::config::Rules::new(
                        Some(rate_limit::config::Quota {
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
                        HashMap::new(),
                    ),
                    rate_limit::EndpointBuilder::new(),
                )
                .build(rate_limit::datastore::InMemoryDatastore::new()),
        );
    }
    server_builder = server_builder.register_middleware(usize::MAX, GatewayBuilder::new());
    server_builder
        .with_host("0.0.0.0".parse().unwrap())
        .with_app_port(80)
        .build()
        .await
        .unwrap()
        .run()
        .await;
    info!("Gateway stopped");
}
