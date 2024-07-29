mod helper;

#[cfg(feature = "rate-limit")]
mod tests {
    use essentials::{debug, info};
    use helper::*;
    use http::header;
    use pretty_assertions::assert_eq;
    use testing_utils::{
        macros as utils,
        surf::{self, StatusCode},
    };

    #[utils::test(setup = before_each, teardown = after_each)]
    async fn should_succeed(ctx: Context) {
        let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
    async fn should_fail_after_2_requests(ctx: Context) {
        {
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
    async fn should_fail_after_6_requests_from_different_ips(ctx: Context) {
        for i in 0..5 {
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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
            let response = surf::get(format!("http://127.0.0.1:{}/hello", &ctx.context.app))
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

    mod helper {
        use std::collections::HashMap;

        use bb8_redis::{bb8, RedisConnectionManager};
        use essentials::debug;
        use gateway::{rate_limit, time};
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
