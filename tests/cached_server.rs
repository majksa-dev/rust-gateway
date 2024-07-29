mod helper;

#[cfg(feature = "cache")]
mod tests {
    use essentials::debug;
    use helper::*;
    use pretty_assertions::assert_eq;
    use testing_utils::{
        macros as utils,
        surf::{self, StatusCode},
    };

    use crate::assert_req_count;

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_return_uncached_response(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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

    mod helper {
        use bb8_redis::{bb8, RedisConnectionManager};
        use essentials::debug;
        use gateway::{cache, time};
        use testing_utils::testcontainers::{
            core::{ContainerPort, WaitFor},
            runners::AsyncRunner,
            ContainerAsync, GenericImage,
        };

        pub struct Context {
            pub context: crate::helper::Context,
            _redis_server: ContainerAsync<GenericImage>,
        }

        pub async fn before_each() -> Context {
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
            let context = crate::helper::setup(|server_builder| {
                server_builder.register_middleware(
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
            })
            .await;
            Context {
                context,
                _redis_server: redis,
            }
        }

        pub async fn after_each(_ctx: ()) {}
    }
}
