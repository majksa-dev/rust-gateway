use bb8_redis::{bb8, RedisConnectionManager};
use essentials::{debug, info};
use gateway::{http::HeaderMapExt, tcp, ParamRouterBuilder};
use http::{header, Method};
use pretty_assertions::assert_eq;
use std::env;
use std::net::SocketAddr;
use testing_utils::{
    macros as utils,
    surf::{self, StatusCode},
    testcontainers::{
        core::{ContainerPort, WaitFor},
        runners::AsyncRunner,
        ContainerAsync, GenericImage,
    },
};
use tokio::task::JoinHandle;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[utils::test(setup = before_each, teardown = after_each)]
#[cfg_attr(not(feature = "rate-limit"), ignore)]
async fn should_succeed(ctx: Context) {
    let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("X-Real-IP", "1.2.3.4")
        .header("X-Api-Token", "token")
        .header("Host", "app")
        .await;
    debug!("{:?}", response);
    let mut response = response.unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response.header("RateLimit-Limit").unwrap(), "2");
    assert_eq!(response.header("RateLimit-Remaining").unwrap(), "1");
    assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
}

#[utils::test(setup = before_each, teardown = after_each)]
#[cfg_attr(not(feature = "rate-limit"), ignore)]
async fn should_fail_after_2_requests(ctx: Context) {
    {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("X-Real-IP", "1.2.3.4")
            .header("X-Api-Token", "token")
            .header("Host", "app")
            .await;
        info!("{:?}", response);
        let mut response = response.unwrap();
        assert_eq!(response.header("RateLimit-Limit").unwrap(), "2");
        assert_eq!(response.header("RateLimit-Remaining").unwrap(), "1");
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
    }
    {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("X-Real-IP", "1.2.3.4")
            .header("X-Api-Token", "token")
            .header("Host", "app")
            .await;
        info!("{:?}", response);
        let mut response = response.unwrap();
        assert_eq!(response.header("RateLimit-Limit").unwrap(), "2");
        assert_eq!(response.header("RateLimit-Remaining").unwrap(), "0");
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
    }
    {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("X-Real-IP", "1.2.3.4")
            .header("X-Api-Token", "token")
            .header("Host", "app")
            .await;
        info!("{:?}", response);
        let response = response.unwrap();
        assert_eq!(
            response.header(header::RETRY_AFTER.as_str()).is_some(),
            true
        );
        let status = response.status();
        assert_eq!(status, StatusCode::TooManyRequests);
    }
}

#[utils::test(setup = before_each, teardown = after_each)]
#[cfg_attr(not(feature = "rate-limit"), ignore)]
async fn should_fail_after_6_requests_from_different_ips(ctx: Context) {
    for i in 0..5 {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("X-Real-IP", format!("1.2.3.{}", i))
            .header("X-Api-Token", "token")
            .header("Host", "app")
            .await;
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
    }
    {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("X-Real-IP", "1.2.3.6")
            .header("X-Api-Token", "token")
            .header("Host", "app")
            .await;
        let response = response.unwrap();
        assert_eq!(
            response.header(header::RETRY_AFTER.as_str()).is_some(),
            true
        );
        let status = response.status();
        assert_eq!(status, StatusCode::TooManyRequests);
    }
}

struct Context {
    app: u16,
    _origin_server: MockServer,
    _app_server: JoinHandle<()>,
    _redis_server: ContainerAsync<GenericImage>,
}

async fn before_each() -> Context {
    if env::var("CI").is_err() {
        env::set_var("RUST_LOG", "info");
        env::set_var("RUST_BACKTRACE", "0");
        env::set_var("APP_ENV", "d");
        essentials::install();
    }
    let listener = std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
    let mock_addr = listener.local_addr().unwrap();
    let mock_server = MockServer::builder().listener(listener).start().await;
    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Hello, world!"))
        .mount(&mock_server)
        .await;
    let redis = GenericImage::new("redis", "7.2.4")
        .with_exposed_port(ContainerPort::Tcp(6379))
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .start()
        .await
        .expect("Redis could not be started");
    let redis_port = redis.get_host_port_ipv4(6379).await.unwrap();
    let redis_manager =
        RedisConnectionManager::new(format!("redis://127.0.0.1:{redis_port}")).unwrap();
    let redis_pool = bb8::Pool::builder().build(redis_manager).await.unwrap();
    debug!("{:?}", redis_pool);
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
    #[cfg(feature = "rate-limit")]
    {
        use gateway::{rate_limit, time};
        use std::collections::HashMap;
        server_builder = server_builder.register_middleware(
            1,
            rate_limit::Builder::new()
                .add_app(
                    "app",
                    rate_limit::config::Rules {
                        root: Some(rate_limit::config::Quota {
                            total: time::Frequency {
                                amount: 5,
                                interval: time::Time {
                                    amount: 1,
                                    unit: time::TimeUnit::Minutes,
                                },
                            },
                            user: Some(time::Frequency {
                                amount: 2,
                                interval: time::Time {
                                    amount: 1,
                                    unit: time::TimeUnit::Minutes,
                                },
                            }),
                        }),
                        tokens: HashMap::new(),
                    },
                    rate_limit::EndpointBuilder::new(),
                )
                .build(rate_limit::datastore::RedisDatastore::new(redis_pool)),
        );
    }
    let server = server_builder.build().await.unwrap();
    let server_thread = tokio::spawn(server.run());
    wait_for_server(ports[1]).await;
    Context {
        app: ports[0],
        _app_server: server_thread,
        _origin_server: mock_server,
        _redis_server: redis,
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
