use gateway::http::HeaderMapExt;
use gateway::tcp;
use gateway::ParamRouterBuilder;
use http::header;
use http::Method;
use pretty_assertions::assert_eq;
use std::env;
use std::net::SocketAddr;
use testing_utils::macros as utils;
use testing_utils::surf;
use testing_utils::surf::StatusCode;
use tokio::task::JoinHandle;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[utils::test(setup = before_each, teardown = after_each)]
#[cfg_attr(not(feature = "cors"), ignore)]
async fn should_succeed(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("X-Real-IP", "1.2.3.4")
        .header("X-Api-Token", "token")
        .header("Origin", "http://localhost:3000")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::Ok);
}

#[utils::test(setup = before_each, teardown = after_each)]
#[cfg_attr(not(feature = "cors"), ignore)]
async fn should_fail_when_calling_without_host(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::BadGateway);
}

#[utils::test(setup = before_each, teardown = after_each)]
#[cfg_attr(not(feature = "cors"), ignore)]
async fn should_fail_when_calling_valid_endpoint_without_token(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("Origin", "http://localhost:3000")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::Unauthorized);
}

#[utils::test(setup = before_each, teardown = after_each)]
#[cfg_attr(not(feature = "cors"), ignore)]
async fn should_fail_when_calling_valid_endpoint_with_invalid_token(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("Origin", "http://localhost:3000")
        .header("X-Api-Token", "invalid")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::Unauthorized);
}

#[utils::test(setup = before_each, teardown = after_each)]
#[cfg_attr(not(feature = "cors"), ignore)]
async fn should_fail_when_calling_valid_endpoint_without_origin(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("X-Api-Token", "token")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::Forbidden);
}

#[utils::test(setup = before_each, teardown = after_each)]
#[cfg_attr(not(feature = "cors"), ignore)]
async fn should_fail_when_calling_valid_endpoint_with_invalid_origin(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("X-Api-Token", "token")
        .header("Origin", "http://localhost:4000")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::Forbidden);
}

#[utils::test(setup = before_each, teardown = after_each)]
#[cfg_attr(not(feature = "cors"), ignore)]
async fn should_fail_when_calling_invalid_endpoint(ctx: Context) {
    let status = surf::get(format!("http://127.0.0.1:{}/unknown", &ctx.app))
        .header("X-Api-Token", "token")
        .header("Origin", "http://localhost:3000")
        .header("Host", "app")
        .await
        .unwrap()
        .status();
    assert_eq!(status, StatusCode::Forbidden);
}

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
    let mock_addr = listener.local_addr().unwrap();
    let mock_server = MockServer::builder().listener(listener).start().await;
    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&mock_server)
        .await;
    let ports = testing_utils::get_random_ports(2);
    let mut server_builder = gateway::builder(
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
    .with_health_check_port(ports[1]);
    server_builder = server_builder.register_peer(
        "app".to_string(),
        ParamRouterBuilder::new().add_route(Method::GET, "/hello".to_string(), "hello".to_string()),
    );
    #[cfg(feature = "cors")]
    {
        use gateway::cors;
        server_builder = server_builder.register_middleware(
            1,
            cors::Builder::new()
                .add_auth(
                    "app",
                    "token".to_string(),
                    vec!["http://localhost:3000".to_string()],
                )
                .build(),
        );
    }
    let server = server_builder.build().await.unwrap();
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
