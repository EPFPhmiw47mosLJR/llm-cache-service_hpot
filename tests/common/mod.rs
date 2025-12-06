#![allow(dead_code)] // TODO: FIX THIS!

use testcontainers::{
    ContainerAsync, GenericImage,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};

pub fn setup_logger(default_level: &str) {
    // testcontainers=debug,llm_cache_service=debug
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(default_level));

    let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
}

pub async fn setup_redis_testcontainer()
-> Result<(String, u16, ContainerAsync<GenericImage>), Box<dyn std::error::Error + 'static>> {
    let redis_port: u16 = 6379;

    let container = GenericImage::new("redis", "latest")
        .with_exposed_port(redis_port.tcp())
        .with_wait_for(WaitFor::message_on_stdout("Ready to accept connections"))
        .start()
        .await?;
    let host = container.get_host().await?;
    let host_port = container.get_host_port_ipv4(redis_port).await?;

    Ok((host.to_string(), host_port, container))
}
