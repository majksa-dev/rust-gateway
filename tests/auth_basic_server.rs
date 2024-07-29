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
    async fn should_succeed(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .header("Authorization", "Basic dXNlcm5hbWU6cGFzc3dvcmQ=") // username:password
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
    }

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_401_when_no_auth_header_is_attached(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .await;
        debug!("{:?}", response);
        let response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Unauthorized);
    }

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_403_when_password_does_not_match(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("Host", "app")
            .header("Authorization", "Basic dXNlcm5hbWU6aW52YWxpZA==") // username:invalid
            .await;
        debug!("{:?}", response);
        let response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Forbidden);
    }

    mod helper {
        pub use crate::helper::Context;
        use gateway::auth;

        pub async fn before_each() -> Context {
            crate::helper::setup(|server_builder| {
                server_builder.register_middleware(
                    1,
                    auth::basic::Builder::new()
                        .add_app_credential(
                            "app",
                            auth::basic::config::Credential {
                                username: "username".to_string(),
                                password: "password".to_string(),
                            },
                        )
                        .build(),
                )
            })
            .await
        }

        pub async fn after_each(_ctx: ()) {}
    }
}
