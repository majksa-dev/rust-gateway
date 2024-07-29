mod helper;

#[cfg(feature = "auth")]
mod tests {
    use essentials::debug;
    use helper::*;
    use pretty_assertions::assert_eq;
    use testing_utils::{
        macros as utils,
        surf::{self, StatusCode},
    };

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_email(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/email", &ctx.context.app))
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
    async fn should_return_email_when_calling_secret_endpoint_with_permissions(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/secret", &ctx.context.app))
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
        let response = surf::get(format!("http://127.0.0.1:{}/email", &ctx.context.app))
            .header("Host", "app")
            .await;
        debug!("{:?}", response);
        let response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Unauthorized);
    }

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_401_when_invalid_token_is_used(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/email", &ctx.context.app))
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
        let response = surf::get(format!("http://127.0.0.1:{}/private", &ctx.context.app))
            .header("Host", "app")
            .header("Authorization", "Bearer hello_world")
            .await;
        debug!("{:?}", response);
        let response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Forbidden);
    }

    mod helper {
        use essentials::debug;
        use gateway::auth;
        use http::header;
        use serde_json::json;
        use std::net::SocketAddr;
        use wiremock::{
            matchers::{method, path},
            Mock, MockServer, Request, Respond, ResponseTemplate,
        };

        pub struct Context {
            pub context: crate::helper::Context,
            _user_info_server: MockServer,
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

        pub async fn before_each() -> Context {
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
            let context = crate::helper::setup(|server_builder| {
                server_builder.register_middleware(
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
                        .require_app_roles("app", vec!["ROLE1".to_string()])
                        .set_app_endpoints(
                            "app",
                            auth::endpoint::EndpointBuilder::new()
                                .add_endpoint(
                                    "secret",
                                    auth::endpoint::config::Endpoint::new(
                                        vec!["ROLE2".to_string()],
                                    ),
                                )
                                .add_endpoint(
                                    "private",
                                    auth::endpoint::config::Endpoint::new(
                                        vec!["ROLE3".to_string()],
                                    ),
                                ),
                        )
                        .build(),
                )
            })
            .await;
            Context {
                context,
                _user_info_server: user_info_server,
            }
        }

        pub async fn after_each(_ctx: ()) {}
    }
}
