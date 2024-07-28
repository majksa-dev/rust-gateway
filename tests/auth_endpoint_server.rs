#![cfg(feature = "auth")]
mod tests {
    use essentials::debug;
    use gateway::{http::HeaderMapExt, tcp, ParamRouterBuilder};
    use http::{header, Method};
    use pretty_assertions::assert_eq;
    use serde_json::json;
    use std::env;
    use std::net::SocketAddr;
    use testing_utils::{
        macros as utils,
        surf::{self, StatusCode},
    };
    use tokio::task::JoinHandle;
    use wiremock::{
        matchers::{method, path},
        Mock, MockServer, Request, Respond, ResponseTemplate,
    };

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_email(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .header("Authorization", "Bearer hello_world")
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "john@doe.com");
    }

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_401_when_no_token_is_attached(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .await;
        debug!("{:?}", response);
        let response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Unauthorized);
    }

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_401_when_invalid_token_is_used(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .header("Authorization", "Bearer hello")
            .await;
        debug!("{:?}", response);
        let response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Unauthorized);
    }

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_403_when_user_does_not_have_role(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/private", &ctx.app))
            .header("Host", "app")
            .header("Authorization", "Bearer hello_world")
            .await;
        debug!("{:?}", response);
        let response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Forbidden);
    }

    struct Context {
        app: u16,
        _origin_server: MockServer,
        _user_info_server: MockServer,
        _app_server: JoinHandle<()>,
    }

    struct RespondWithEmailHeader;

    impl Respond for RespondWithEmailHeader {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            let email = request.headers.get("X-Email").unwrap();
            ResponseTemplate::new(200).set_body_string(email.to_str().unwrap())
        }
    }

    struct RespondWithUserInfo;

    impl Respond for RespondWithUserInfo {
        fn respond(&self, request: &Request) -> ResponseTemplate {
            let token = request
                .headers
                .get(header::AUTHORIZATION)
                .unwrap()
                .to_str()
                .unwrap();
            debug!(
                token,
                expected = "Bearer hello_world",
                result = token != "Bearer hello_world",
                ""
            );
            if token != "Bearer hello_world" {
                ResponseTemplate::new(401)
            } else {
                ResponseTemplate::new(200).set_body_string(
                    json!({
                        "security": {
                            "roles": [
                                { "name": "ROLE1" },
                                { "name": "ROLE2" }
                            ]
                        },
                        "extra": {
                            "email": "john@doe.com"
                        }
                    })
                    .to_string(),
                )
            }
        }
    }

    async fn before_each() -> Context {
        if env::var("CI").is_err() {
            env::set_var("RUST_LOG", "debug");
            env::set_var("RUST_BACKTRACE", "0");
            env::set_var("APP_ENV", "d");
            essentials::install();
        }
        let (mock_server, mock_addr) = {
            let listener =
                std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
            let mock_addr = listener.local_addr().unwrap();
            let server = MockServer::builder().listener(listener).start().await;
            Mock::given(method("GET"))
                .and(path("/hello"))
                .respond_with(RespondWithEmailHeader)
                .mount(&server)
                .await;
            (server, mock_addr.to_string())
        };
        let user_info_server = {
            let listener =
                std::net::TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0))).unwrap();
            let server = MockServer::builder().listener(listener).start().await;
            Mock::given(method("GET"))
                .and(path("/userinfo"))
                .respond_with(RespondWithUserInfo)
                .mount(&server)
                .await;
            server
        };
        debug!("UserInfo server started at: {:?}", user_info_server.uri());
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
            ParamRouterBuilder::new()
                .add_route(Method::GET, "/hello".to_string(), "hello".to_string())
                .add_route(Method::GET, "/secret".to_string(), "secret".to_string())
                .add_route(Method::GET, "/private".to_string(), "private".to_string()),
        );
        #[cfg(feature = "auth")]
        {
            use gateway::auth;
            server_builder = server_builder.register_middleware(
                1,
                auth::endpoint::Builder::new()
                    .add_app_auth(
                        "app",
                        auth::endpoint::config::Auth::new(
                            reqwest::Url::parse(
                                format!("{}/userinfo", user_info_server.uri()).as_str(),
                            )
                            .unwrap(),
                            vec![auth::endpoint::config::Claim {
                                claim: "extra.email".to_string(),
                                header: "X-Email".to_string(),
                            }],
                            Some(auth::endpoint::config::RolesClaims {
                                claim: "security.roles".to_string(),
                                inner_mapping: Some("name".to_string()),
                            }),
                        ),
                    )
                    .set_app_endpoints(
                        "app",
                        auth::endpoint::EndpointBuilder::new()
                            .add_endpoint(
                                "secret",
                                auth::endpoint::config::Endpoint::new(vec!["ROLE1".to_string()]),
                            )
                            .add_endpoint(
                                "private",
                                auth::endpoint::config::Endpoint::new(vec!["ROLE3".to_string()]),
                            ),
                    )
                    .build(),
            );
        }
        let server = server_builder.build().await.unwrap();
        let server_thread = tokio::spawn(server.run());
        wait_for_server(ports[1]).await;
        Context {
            app: ports[0],
            _user_info_server: user_info_server,
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
}
