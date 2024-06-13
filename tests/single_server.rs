use gateway::cors;
use gateway::TcpOrigin;
use http::header;
use pretty_assertions::assert_eq;
use std::collections::HashMap;
use std::env;
use std::net::SocketAddr;
use testing_utils::macros as utils;
use testing_utils::surf;
use tokio::task::JoinHandle;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

struct Context {
    app: u16,
    _origin_server: MockServer,
    _app_server: JoinHandle<()>,
}

async fn before_each() -> Context {
    if env::var("CI").is_err() {
        env::set_var("RUST_LOG", "trace");
        env::set_var("RUST_BACKTRACE", "0");
        env::set_var("APP_ENV", "d");
        essentials::install();
    }
    let listener = std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
    let origin = listener.local_addr().unwrap().port();
    let mock_server = MockServer::builder().listener(listener).start().await;
    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;
    let ports = testing_utils::get_random_ports(2);
    let server = gateway::builder(
        TcpOrigin::new(HashMap::from([(
            "app".to_string(),
            Box::new(SocketAddr::from(([127, 0, 0, 1], origin))),
        )])),
        |request| {
            request
                .headers
                .get(header::HOST)
                .and_then(|value| value.to_str().ok())
                .map(|x| x.to_string())
        },
    )
    .with_app_port(ports[0])
    .with_health_check_port(ports[1])
    .register_peer("app".to_string(), |request| match request.path.as_str() {
        "/hello" => Some("hello".to_string()),
        _ => None,
    })
    .register_middleware(
        1,
        cors::Middleware(cors::Config::new(HashMap::from([(
            "app".to_string(),
            cors::AppConfig::new(
                cors::ConfigRules::new(
                    vec![],
                    vec![cors::Auth::new("token", vec!["http://localhost:3000"])],
                ),
                HashMap::from([(
                    "hello".to_string(),
                    cors::ConfigRules::new(vec![http::Method::GET], vec![]),
                )]),
            ),
        )]))),
    )
    .build();
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

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_succeed(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("X-Real-IP", "1.2.3.4")
        .header("X-Api-Token", "token")
        .header("Origin", "http://localhost:3000")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status as u16, 200);
}

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_fail_when_calling_without_host(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .await
        .unwrap()
        .status();
    assert_eq!(status as u16, 502, "expected status 502, got {}", status);
}

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_fail_when_calling_valid_endpoint_without_token(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("X-Real-IP", "1.2.3.4")
        .header("Origin", "http://localhost:3000")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status as u16, 400);
}

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_fail_when_calling_valid_endpoint_with_invalid_token(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("X-Real-IP", "1.2.3.4")
        .header("Origin", "http://localhost:3000")
        .header("X-Api-Token", "invalid")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status as u16, 401);
}

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_fail_when_calling_valid_endpoint_without_ip(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("X-Api-Token", "token")
        .header("Origin", "http://localhost:3000")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status as u16, 200);
}

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_fail_when_calling_valid_endpoint_without_origin(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("X-Real-IP", "1.2.3.4")
        .header("X-Api-Token", "token")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status as u16, 400);
}

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_fail_when_calling_valid_endpoint_with_invalid_origin(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("X-Real-IP", "1.2.3.4")
        .header("X-Api-Token", "token")
        .header("Origin", "http://localhost:4000")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status as u16, 403);
}

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_fail_when_calling_invalid_endpoint(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/unknown", &ctx.app))
        .header("X-Real-IP", "1.2.3.4")
        .header("X-Api-Token", "token")
        .header("Origin", "http://localhost:3000")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status as u16, 404);
}
