use std::env;

use async_trait::async_trait;
use gateway::gateway::middleware::Middleware;
use pingora::{proxy::Session, Result};

struct Gateway;

#[async_trait]
impl Middleware for Gateway {
    async fn request_filter(&self, session: &mut Session) -> Result<bool> {
        session.respond_error(405).await;
        Ok(true)
    }
}

fn main() {
    essentials::install();
    gateway::server::app::builder()
        .with_app_port(
            env::var("PORT")
                .ok()
                .unwrap_or("80".to_string())
                .parse()
                .unwrap_or(80),
        )
        .register_middleware(1, Box::new(Gateway))
        .build()
        .unwrap()
        .run_forever();
}
