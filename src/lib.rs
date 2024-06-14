//! Gateway library to create custom gateways.
//!
//! # Example usage
//!
//! ```
//! use async_trait::async_trait;
//! use gateway::{
//!     cors,
//!     http::{HeaderMapExt, Request, Response},
//!     rate_limit, time, Context, Middleware, Next, ParamRouter, Result, TcpOrigin,
//! };
//! use http::{header, Method};
//! use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};
//!
//! struct Gateway;
//!
//! #[async_trait]
//! impl Middleware for Gateway {
//!     async fn run(&self, _ctx: Arc<Context>, request: Request, next: Next) -> Result<Response> {
//!         let mut response = next.run(request).await?;
//!         response.insert_header("X-Server", "Rust").unwrap();
//!         response.insert_header(header::CONNECTION, "close").unwrap();
//!         Ok(response)
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     env::set_var("RUST_LOG", "info");
//!     essentials::install();
//!     gateway::builder(
//!         TcpOrigin::new(HashMap::from([(
//!             "app".to_string(),
//!             Box::new("127.0.0.1:7979".parse::<SocketAddr>().unwrap()),
//!         )])),
//!         |request| {
//!             request
//!                 .header(header::HOST)
//!                 .and_then(|value| value.to_str().ok())
//!                 .map(|x| x.to_string())
//!         },
//!     )
//!     .register_peer(
//!         "app".to_string(),
//!         ParamRouter::new().add_route(Method::GET, "/:hello".to_string(), "hello".to_string()),
//!     )
//!     .register_middleware(1, Gateway)
//!     .register_middleware(
//!         2,
//!         cors::Middleware::new(cors::Config {
//!             config: HashMap::new(),
//!         }),
//!     )
//!     .register_middleware(
//!         3,
//!         rate_limit::Middleware::new(
//!             rate_limit::Config {
//!                 config: HashMap::from([(
//!                     "app".to_string(),
//!                     rate_limit::AppConfig::new(
//!                         Some(rate_limit::Quota {
//!                             total: time::Frequency {
//!                                 amount: 5,
//!                                 interval: time::Time {
//!                                     amount: 1,
//!                                     unit: time::TimeUnit::Minutes,
//!                                 },
//!                             },
//!                             user: Some(time::Frequency {
//!                                 amount: 2,
//!                                 interval: time::Time {
//!                                     amount: 1,
//!                                     unit: time::TimeUnit::Minutes,
//!                                 },
//!                             }),
//!                         }),
//!                         HashMap::new(),
//!                     ),
//!                 )]),
//!             },
//!             rate_limit::InMemoryDatastore::new(),
//!         ),
//!     )
//!     .with_app_port(7878)
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
pub(crate) mod utils;

pub use gateway::{
    entrypoint::EntryPoint,
    middleware::{Context, Middleware, Service},
    origin::{Origin, OriginResponse, OriginServer, TcpOrigin},
    router::{ParamRouter, RegexRouter, Router, RouterService},
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
