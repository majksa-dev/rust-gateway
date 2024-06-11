use async_trait::async_trait;
use gateway::{
    http::{Request, Response},
    Context, Middleware, Next, Result,
};
use http::header;
use std::{env, net::SocketAddr, sync::Arc};

struct Gateway;

#[async_trait]
impl Middleware for Gateway {
    async fn run(&self, _ctx: Arc<Context>, request: Request, next: Next) -> Result<Response> {
        let mut response = next.run(request).await?;
        response.insert_header("X-Server", "Rust").unwrap();
        response.insert_header(header::CONNECTION, "close").unwrap();
        Ok(response)
    }
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "info");
    essentials::install();
    gateway::builder(Box::new(|request| {
        request
            .headers
            .get("X-App")
            .unwrap_or(&header::HeaderValue::from_static("app"))
            .to_str()
            .unwrap()
            .to_string()
    }))
    .register_peer(
        "app".to_string(),
        Box::new("127.0.0.1:7979".parse::<SocketAddr>().unwrap()),
        Box::new(|request| request.path.clone()),
    )
    .register_middleware(1, Box::new(Gateway))
    .with_app_port(7878)
    .build()
    .run()
    .await;
}
