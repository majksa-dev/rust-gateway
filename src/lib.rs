//! Gateway library to create custom gateways.
//!
//! # Example usage
//!
//! ```
//! use async_trait::async_trait;
//! use gateway::{
//!     cors,
//!     http::{Request, Response},
//!     Context, Middleware, Next, Result, TcpOrigin,
//! };
//! use http::header;
//! use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};

//! struct Gateway;

//! #[async_trait]
//! impl Middleware for Gateway {
//!     async fn run(&self, _ctx: Arc<Context>, request: Request, next: Next) -> Result<Response> {
//!         let mut response = next.run(request).await?;
//!         response.insert_header("X-Server", "Rust").unwrap();
//!         response.insert_header(header::CONNECTION, "close").unwrap();
//!         Ok(response)
//!     }
//! }

//! #[tokio::main]
//! async fn main() {
//!     env::set_var("RUST_LOG", "info");
//!     essentials::install();
//!     gateway::builder(
//!         TcpOrigin::new(HashMap::from([(
//!             "app".to_string(),
//!             Box::new("127.0.0.1:8081".parse::<SocketAddr>().unwrap()),
//!         )])),
//!         |request| {
//!             request
//!                 .headers
//!                 .get("X-App")
//!                 .and_then(|value| value.to_str().ok())
//!                 .map(|x| x.to_string())
//!         },
//!     )
//!     .register_peer("app".to_string(), |request| Some(request.path.clone()))
//!     .register_middleware(1, Gateway)
//!     .register_middleware(
//!         2,
//!         cors::Middleware(cors::Config {
//!             config: HashMap::new(),
//!         }),
//!     )
//!     .with_app_port(8080)
//!     .build()
//!     .run()
//!     .await;
//! }
//! ```
#[cfg(feature = "cors")]
pub mod cors;
pub(crate) mod gateway;
pub mod http;
pub mod io;
#[cfg(feature = "rate-limit")]
pub mod rate_limit;
pub(crate) mod server;
pub mod thread;
pub(crate) mod utils;

pub use gateway::{
    entrypoint::EntryPoint,
    middleware::{Context, Middleware, Service},
    origin::{Origin, OriginResponse, OriginServer, TcpOrigin},
    Error, Next, Result,
};
pub use http::{
    server::{Handler, Server as HttpServer},
    ReadHeaders, ReadRequest, ReadResponse, Request, Response, WriteHeaders, WriteRequest,
    WriteResponse,
};
pub use server::{
    app::{builder, Server, ServerBuilder},
    health_check::HealthCheck,
};
pub use utils::time;
