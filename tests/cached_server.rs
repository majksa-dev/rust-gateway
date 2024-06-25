use bb8_redis::{bb8, RedisConnectionManager};
use essentials::debug;
use gateway::{cache, http::HeaderMapExt, tcp, time, ParamRouterBuilder};
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

struct Context {
    app: u16,
    origin_server: MockServer,
    _app_server: JoinHandle<()>,
    _redis_server: ContainerAsync<GenericImage>,
}

async fn before_each() -> Context {
    if env::var("CI").is_err() {
        env::set_var("RUST_LOG", "debug");
        env::set_var("RUST_BACKTRACE", "0");
        env::set_var("APP_ENV", "d");
        essentials::install();
    }
    let listener = std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
    let mock_addr = listener.local_addr().unwrap();
    let mock_server = MockServer::builder().listener(listener).start().await;
    Mock::given(method("GET"))
        .and(path("/hello"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string("Hello, world!")
                .insert_header("X-Custom", "unique"),
        )
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
    .register_middleware(
        1,
        cache::Builder::new()
            .add_endpoint(
                "app",
                "hello",
                cache::config::Endpoint::new(
                    time::Time {
                        amount: 1,
                        unit: time::TimeUnit::Seconds,
                    },
                    vec!["X-Username".to_string()],
                ),
            )
            .build(cache::datastore::RedisDatastore::new(redis_pool)),
    )
    .build()
    .await
    .unwrap();
    let server_thread = tokio::spawn(server.run());
    wait_for_server(ports[1]).await;
    Context {
        app: ports[0],
        _app_server: server_thread,
        origin_server: mock_server,
        _redis_server: redis,
    }
}

async fn after_each(_ctx: ()) {}

// Defined custom macro
macro_rules! assert_req_count {
    ($ctx:expr,$count:expr) => {
        assert_eq!(
            $ctx.origin_server
                .received_requests()
                .await
                .unwrap_or_default()
                .len(),
            $count
        );
    };
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

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_return_uncached_response(ctx: Context) {
    let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
        .header("Host", "app")
        .await;
    debug!("{:?}", response);
    let mut response = response.unwrap();
    let status = response.status();
    assert_eq!(status, StatusCode::Ok);
    assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
    assert_eq!(
        response.header("X-Custom").unwrap().get(0).unwrap(),
        "unique"
    );
    assert_req_count!(ctx, 1);
}

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_return_forward_varying_requests(ctx: Context) {
    {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .header("X-Username", "username1")
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
        assert_eq!(
            response.header("X-Custom").unwrap().get(0).unwrap(),
            "unique"
        );
        assert_req_count!(ctx, 1);
    }
    {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .header("X-Username", "username1")
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
        assert_eq!(
            response.header("X-Custom").unwrap().get(0).unwrap(),
            "unique"
        );
        assert_req_count!(ctx, 1);
    }
    {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .header("X-Username", "username2")
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
        assert_eq!(
            response.header("X-Custom").unwrap().get(0).unwrap(),
            "unique"
        );
        assert_req_count!(ctx, 2);
    }
    {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .header("X-Username", "username2")
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
        assert_eq!(
            response.header("X-Custom").unwrap().get(0).unwrap(),
            "unique"
        );
        assert_req_count!(ctx, 2);
    }
}

#[utils::test(setup = before_each, teardown = after_each)]
async fn should_return_cached_response_after_initial(ctx: Context) {
    {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
        assert_eq!(
            response.header("X-Custom").unwrap().get(0).unwrap(),
            "unique"
        );
        assert_req_count!(ctx, 1);
    }
    {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
        assert_eq!(
            response.header("X-Custom").unwrap().get(0).unwrap(),
            "unique"
        );
        assert_req_count!(ctx, 1);
    }
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
        assert_eq!(
            response.header("X-Custom").unwrap().get(0).unwrap(),
            "unique"
        );
        assert_eq!(response.header("ETag").is_none(), true);
        assert_req_count!(ctx, 2);
    }
}
