use essentials::debug;
use gateway::{http::HeaderMapExt, tcp, ParamRouterBuilder};
use http::{header, Method};
use pretty_assertions::assert_eq;
use std::env;
use std::net::SocketAddr;
use testing_utils::{
    macros as utils,
    surf::{self, StatusCode},
};
use tokio::task::JoinHandle;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_succeed(ctx: Context) {
    let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("Host", "app")
        .await;
    debug!("{:?}", response);
    let mut response = response.unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
}

struct Context {
    app: u16,
    _origin_server: MockServer,
    _app_server: JoinHandle<()>,
}

async fn before_each() -> Context {
    if env::var("CI").is_err() {
        env::set_var("RUST_LOG", "debug");
        env::set_var("RUST_BACKTRACE", "0");
        env::set_var("APP_ENV", "d");
        essentials::install();
    }
    let listener = std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
    let mock_addr = listener.local_addr().unwrap().to_string();
    let mock_server = MockServer::builder().listener(listener).start().await;
    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Hello, world!"))
        .mount(&mock_server)
        .await;
    let ports = testing_utils::get_random_ports(2);
    let server = gateway::builder(
        tcp::Builder::new()
            .add_peer("app", tcp::config::Connection::new(mock_addr))
            .build(),
        |request| {
            request
                .header(header::HOST)
                .and_then(|value| value.to_str().ok())
                .map(|x| x.to_string())
        },
    )
    .with_app_port(ports[0])
    .with_health_check_port(ports[1])
    .register_peer(
        "app".to_string(),
        ParamRouterBuilder::new().add_route(Method::GET, "/hello".to_string(), "hello".to_string()),
    )
    .build()
    .await
    .unwrap();
    let server_thread = tokio::spawn(server.run());
    wait_for_server(ports[1]).await;
    Context {
        app: ports[0],
        _app_server: server_thread,
        _origin_server: mock_server,
    }
}

async fn after_each(_ctx: ()) {}

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
