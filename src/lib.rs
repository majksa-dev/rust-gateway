//! Gateway library to create custom gateways.
//!
//! # Example usage
//!
//! ```
//! use std::{
//!     env,
//!     net::{IpAddr, SocketAddr},
//! };
//!
//! use async_trait::async_trait;
//! use gateway::Middleware;
//! use pingora::{proxy::Session, upstreams::peer::HttpPeer, Result};
//!
//! struct Gateway;
//!
//! #[async_trait]
//! impl Middleware for Gateway {
//!     async fn filter(&self, session: &mut Session) -> Result<bool> {
//!         session.respond_error(404).await;
//!         Ok(false)
//!     }
//! }
//!
//! fn generate_peer_key(session: &Session) -> String {
//!     let mut host = session
//!         .get_header("Host")
//!         .and_then(|host| host.to_str().ok())
//!         .unwrap_or("")
//!         .to_string();
//!     host.truncate(host.find('.').unwrap_or(host.len()));
//!     host
//! }
//! essentials::install();
//! gateway::builder(Box::new(generate_peer_key))
//!     .with_app_port(
//!         env::var("PORT")
//!             .ok()
//!             .unwrap_or("80".to_string())
//!             .parse()
//!             .unwrap_or(80),
//!     )
//!     .register_middleware(1, Box::new(Gateway))
//!     .register_peer(
//!         "hello".to_string(),
//!         Box::new(HttpPeer::new(
//!             SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8083),
//!             false,
//!             "localhost".to_string(),
//!         )),
//!     )
//!     .build()
//!     .unwrap()
//!     .run_forever();
//! ```
pub(crate) mod gateway;
pub(crate) mod server;

pub use gateway::middleware::Middleware;
pub use server::app::{builder, builder_with_health_check, ServerBuilder};
