use async_trait::async_trait;
use gateway::{
    cors::{Cors, CorsConfig},
    http::{Request, Response},
    Context, Middleware, Next, Result, TcpOrigin,
};
use http::header;
use std::{collections::HashMap, env, net::SocketAddr, sync::Arc};

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
    gateway::builder(
        TcpOrigin::new(HashMap::from([(
            "app".to_string(),
            Box::new("127.0.0.1:7979".parse::<SocketAddr>().unwrap()),
        )])),
        |request| {
            request
                .headers
                .get("X-App")
                .and_then(|value| value.to_str().ok())
                .map(|x| x.to_string())
        },
    )
    .register_peer("app".to_string(), |request| Some(request.path.clone()))
    .register_middleware(1, Gateway)
    .register_middleware(
        2,
        Cors(CorsConfig {
            config: HashMap::new(),
        }),
    )
    .with_app_port(7878)
    .build()
    .run()
    .await;
}
