use testcontainers::{core::WaitFor, runners::AsyncRunner, ContainerAsync, GenericImage};

pub async fn start_redis() -> ContainerAsync<GenericImage> {
    GenericImage::new("redis", "latest")
        .with_exposed_port(6379)
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .start()
        .await
}
