use gateway::{http::HeaderMapExt, tcp, ParamRouterBuilder};
use http::{header, Method};
use std::env;
use std::net::SocketAddr;
use testing_utils::surf;
use tokio::task::JoinHandle;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, Request, Respond, ResponseTemplate,
};

#[allow(dead_code)]
pub async fn setup(
    modify: impl FnOnce(gateway::ServerBuilder) -> gateway::ServerBuilder,
) -> Context {
    setup_with_ports(0, |builder, _| modify(builder)).await.0
}

#[allow(dead_code)]
pub async fn setup_with_ports(
    ports: u16,
    modify: impl FnOnce(gateway::ServerBuilder, &[u16]) -> gateway::ServerBuilder,
) -> (Context, Vec<u16>) {
    setup_system();
    let (mock_server, mock_addr) = create_origin_server().await;
    let (server, server_ports, custom_ports) = create_server(mock_addr, ports, modify).await;
    let server_thread = tokio::spawn(server.run());
    wait_for_server(server_ports.1).await;
    (
        Context {
            app: server_ports.0,
            _app_server: server_thread,
            origin_server: mock_server,
        },
        custom_ports,
    )
}

#[macro_export]
macro_rules! assert_req_count {
    ($ctx:expr,$count:expr) => {
        assert_eq!(
            $ctx.context
                .origin_server
                .received_requests()
                .await
                .unwrap_or_default()
                .len(),
            $count
        );
    };
}

pub struct Context {
    #[allow(dead_code)]
    pub app: u16,
    #[allow(dead_code)]
    pub origin_server: MockServer,
    _app_server: JoinHandle<()>,
}

struct RespondWithEmailHeader;

impl Respond for RespondWithEmailHeader {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let email = request.headers.get("X-Email").unwrap();
        ResponseTemplate::new(200).set_body_string(email.to_str().unwrap())
    }
}

fn setup_system() {
    if env::var("CI").is_err() {
        env::set_var("APP_ENV", "d");
        env::set_var("RUST_LOG", "debug");
        env::set_var("RUST_BACKTRACE", "0");
        essentials::install();
    }
}

async fn create_origin_server() -> (MockServer, String) {
    let listener = std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
    let mock_addr = listener.local_addr().unwrap();
    let server = MockServer::builder().listener(listener).start().await;
    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("Hello, world!")
                .append_header("X-Custom", "unique"),
        )
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/email"))
        .respond_with(RespondWithEmailHeader)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/secret"))
        .respond_with(RespondWithEmailHeader)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/private"))
        .respond_with(RespondWithEmailHeader)
        .mount(&server)
        .await;
    (server, mock_addr.to_string())
}

async fn create_server(
    origin_addr: String,
    ports: u16,
    modify: impl FnOnce(gateway::ServerBuilder, &[u16]) -> gateway::ServerBuilder,
) -> (gateway::Server, (u16, u16), Vec<u16>) {
    let ports = testing_utils::get_random_ports(2 + ports);
    let custom_ports = ports[2..].to_vec();
    let server_builder = gateway::builder(
        tcp::Builder::new()
            .add_peer("app", tcp::config::Connection::new(origin_addr))
            .build(),
        |request| {
            request
                .header(header::HOST)
                .and_then(|value| value.to_str().ok())
                .map(|x| (x.to_string(), None))
        },
    )
    .with_app_port(ports[0])
    .with_health_check_port(ports[1])
    .register_peer(
        "app".to_string(),
        ParamRouterBuilder::new()
            .add_route(Method::GET, "/hello".to_string(), "hello".to_string())
            .add_route(Method::GET, "/email".to_string(), "email".to_string())
            .add_route(Method::GET, "/secret".to_string(), "secret".to_string())
            .add_route(Method::GET, "/private".to_string(), "private".to_string()),
    );
    (
        modify(server_builder, &custom_ports).build().await.unwrap(),
        (ports[0], ports[1]),
        custom_ports,
    )
}

async fn wait_for_server(health_check: u16) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_millis(100));
    loop {
        if let Ok(response) = surf::get(format!("http://127.0.0.1:{}", health_check)).await {
            if response.status() == 200 {
                break;
            }
        }
        interval.tick().await;
    }
}
