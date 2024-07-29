mod helper;

#[cfg(feature = "auth")]
mod tests {
    use helper::*;
    use pretty_assertions::assert_eq;
    use testing_utils::{
        macros as utils,
        surf::{self, StatusCode},
    };

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
        assert_eq!(status, StatusCode::Ok);
    }

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_fail_when_calling_without_host(ctx: Context) {
        let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .await
            .unwrap()
            .status();
        assert_eq!(status, StatusCode::BadGateway);
    }

    #[utils::test(setup = before_each, teardown = after_each)]
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
    async fn should_succeed_when_calling_valid_endpoint_without_origin(ctx: Context) {
        let status = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
            .header("X-Api-Token", "token2")
            .header("Host", "app")
            .await
            .unwrap()
            .status();
        assert_eq!(status, StatusCode::Ok);
    }

    #[utils::test(setup = before_each, teardown = after_each)]
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

    mod helper {
        pub use crate::helper::Context;
        use gateway::cors;

        pub async fn before_each() -> Context {
            crate::helper::setup(|server_builder| {
                server_builder.register_middleware(
                    1,
                    cors::Builder::new()
                        .add_auth(
                            "app",
                            "token".to_string(),
                            vec!["http://localhost:3000".to_string()],
                        )
                        .add_token("app", "token2".to_string())
                        .build(),
                )
            })
            .await
        }

        pub async fn after_each(_ctx: ()) {}
    }
}
