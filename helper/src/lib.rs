use assert_fs::prelude::*;
use essentials::{debug, info};
use futures_util::Future;
use serde_json::Value;

pub use helper_macros::*;
use testcontainers::{ContainerAsync, GenericImage};

mod bin;
mod redis;
mod utils;

#[derive(Clone)]
pub struct Context {
    pub app: u16,
    pub health_check: u16,
    pub redis: u16,
    pub services: Vec<u16>,
}

struct Processes {
    pub app: std::process::Child,
    pub redis: ContainerAsync<GenericImage>,
}

async fn setup(config: Value) -> Result<(Processes, Context), String> {
    utils::setup_env();
    let redis = redis::start_redis().await;
    let ports_vec = utils::get_random_ports(2);
    let ports = Context {
        app: ports_vec[0],
        health_check: ports_vec[1],
        redis: redis.get_host_port_ipv4(6379).await,
        services: vec![],
    };
    let temp = assert_fs::TempDir::new().unwrap();
    let input_file = temp.child("config.json");
    input_file.touch().unwrap();
    input_file.write_str(&config.to_string()).unwrap();
    debug!("Provided config: {}", config.to_string());
    info!("Starting redis on port {}", ports.redis.to_string());
    let app = bin::server_cmd()
        .env("RUST_BACKTRACE", "full")
        .env("RUST_LOG", "debug")
        .env("PORT", ports.app.to_string())
        .env("HEALTHCHECK_PORT", ports.health_check.to_string())
        .env("CONFIG_FILE", input_file.path())
        .env("REDIS_URL", format!("redis://localhost:{}", ports.redis))
        .spawn();
    if app.is_err() {
        redis.stop().await;
    }
    let app = app.unwrap();
    for _ in 0..10 {
        if let Ok(status) = surf::get(format!("http://localhost:{}", ports.health_check))
            .await
            .map(|res| res.status())
        {
            if status == 200 {
                return Ok((Processes { app, redis }, ports));
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
    }
    teardown(&mut Processes { app, redis }).await;
    Err("Could not start the server".to_string())
}

async fn teardown(ctx: &mut Processes) {
    ctx.redis.stop().await;
    ctx.app.kill().unwrap();
    ctx.app.wait().unwrap();
}

pub async fn test<T, C, Fut>(servers: u16, config: C, test: T)
where
    C: Fn(&[u16]) -> Value,
    T: FnOnce(Context) -> Fut,
    Fut: Future<Output = ()>,
{
    let apps = utils::random_listeners(servers);
    let services_ports: Vec<_> = apps
        .iter()
        .map(|listener| listener.local_addr().unwrap().port())
        .collect();
    let config = config(&services_ports);
    let setup = setup(config).await;
    drop(apps); // Drop the listeners
    let (mut ctx, mut ports) = setup.unwrap();
    ports.services = services_ports;
    test(ports.clone()).await;
    teardown(&mut ctx).await;
}
