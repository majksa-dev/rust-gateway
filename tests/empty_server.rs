mod helper;

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
            .await;
        debug!("{:?}", response);
        let mut response = response.unwrap();
        let status = response.status();
        assert_eq!(status, StatusCode::Ok);
        assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
    }

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_succeed_when_running_1000_times(ctx: Context) {
        for _ in 0..1000 {
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.app))
                .header("Host", "app")
                .await;
            debug!("{:?}", response);
            let mut response = response.unwrap();
            let status = response.status();
            assert_eq!(status, StatusCode::Ok);
            assert_eq!(response.body_string().await.unwrap(), "Hello, world!");
        }
    }

    mod helper {
        pub use crate::helper::Context;

        pub async fn before_each() -> Context {
            crate::helper::setup(|server_builder| server_builder).await
        }

        pub async fn after_each(_ctx: ()) {}
    }
}
