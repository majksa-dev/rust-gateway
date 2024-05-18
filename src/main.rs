use std::{
    env,
    net::{IpAddr, SocketAddr},
};

use async_trait::async_trait;
use gateway::{Context, Middleware};
use http::header;
use pingora::{
    http::{ResponseHeader, StatusCode},
    proxy::Session,
    upstreams::peer::HttpPeer,
    Result,
};

struct Gateway;

struct Ctx;

#[async_trait]
impl Middleware<Ctx> for Gateway {
    fn new_ctx(&self) -> Ctx {
        Ctx
    }

    async fn filter(
        &self,
        _session: &Session,
        _context: (&Context, &mut Ctx),
    ) -> Result<Option<ResponseHeader>> {
        let mut response = ResponseHeader::build(StatusCode::OK, Some(2))?;
        response.insert_header(header::SERVER, "Example")?;
        Ok(Some(response))
    }
}

fn main() {
    essentials::install();
    gateway::builder(Box::new(|session| {
        let mut host = session
            .get_header("Host")
            .and_then(|host| host.to_str().ok())
            .unwrap_or("")
            .to_string();
        host.truncate(host.find('.').unwrap_or(host.len()));
        host
    }))
    .with_app_port(
        env::var("PORT")
            .ok()
            .unwrap_or("80".to_string())
            .parse()
            .unwrap_or(80),
    )
    // .register_middleware(1, Box::new(Gateway))
    .register_peer(
        "hello".to_string(),
        Box::new(HttpPeer::new(
            SocketAddr::new(IpAddr::from([127, 0, 0, 1]), 8083),
            false,
            "localhost".to_string(),
        )),
        Box::new(|session| session.req_header().uri.path().to_string()),
    )
    .build()
    .unwrap()
    .run_forever();
}
