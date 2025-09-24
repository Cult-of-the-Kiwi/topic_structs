use std::process::Stdio;

use fluvio::{Fluvio, FluvioClusterConfig};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{ContainerPort, WaitFor},
    runners::AsyncRunner,
};
use uuid::Uuid;

use crate::{events::DevcordEvent, publisher::topic::fluvio::FluvioHandler};

async fn build_sc_setup_image() -> String {
    let image_name = "sc-setup:local".to_string();

    let status = std::process::Command::new("docker")
        .args(&["build", "-t", &image_name, "."])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .expect("Failed to start docker build");

    assert!(status.success(), "docker build failed");
    image_name
}

pub struct FluvioStack {
    sc: ContainerAsync<GenericImage>,
    spu: ContainerAsync<GenericImage>,
    sc_setup: ContainerAsync<GenericImage>,
}

impl FluvioStack {
    pub async fn get_fluvio_handler() -> anyhow::Result<FluvioHandler<DevcordEvent>> {
        let stack = Self::new().await;

        FluvioHandler::new(Some(&format!(
            "127.0.0.1:{}",
            stack.sc.get_host_port_ipv4(9003).await?
        )))
        .await
    }
    async fn new() -> Self {
        let network = Uuid::new_v4().to_string();
        // Build sc-setup image dynamically
        let sc_setup_image_name = build_sc_setup_image().await;

        let sc = GenericImage::new("infinyon/fluvio", "stable")
            .with_exposed_port(ContainerPort::Tcp(9003))
            .with_wait_for(WaitFor::message_on_stdout(
                "Streaming Controller started successfully",
            ))
            .with_cmd(vec!["./fluvio-run", "sc", "--local", "/fluvio/metadata"])
            .with_network(&network)
            .start()
            .await
            .expect("failed to start sc");

        let spu = GenericImage::new("infinyon/fluvio", "stable")
            .with_exposed_port(ContainerPort::Tcp(9010))
            .with_exposed_port(ContainerPort::Tcp(9011))
            //.with_wait_for(testcontainers::core::WaitFor::message_on_stdout("Starting SPU"))
            .with_cmd(vec![
                "./fluvio-run",
                "spu",
                "-i",
                "5001",
                "-p",
                "0.0.0.0:9010",
                "-v",
                "spu:9011",
                "--sc-addr",
                "sc:9004",
                "--log-base-dir",
                "/fluvio/data",
            ])
            .with_network(&network)
            .start()
            .await
            .expect("failed to start spu");

        let mut parts = sc_setup_image_name.splitn(2, ':');
        let name = parts.next().unwrap();
        let tag = parts.next().unwrap_or("latest");

        let sc_setup = GenericImage::new(name, tag)
            .with_cmd(vec![
                "/bin/sh",
                "-c",
                "fluvio profile add docker sc:9003 docker; \
                fluvio cluster spu register --id 5001 -p 0.0.0.0:9110 --private-server spu:9011; \
                exit 0;",
            ])
            .with_network(&network)
            .start()
            .await
            .expect("failed to start sc-setup");

        Self { sc, spu, sc_setup }
    }
}

#[tokio::test]
async fn test_fluvio_stack() {
    let stack = FluvioStack::new().await;

    let sc_port = stack.sc.get_host_port_ipv4(9003).await.unwrap();
    let spu_port = stack.spu.get_host_port_ipv4(9010).await.unwrap();

    Fluvio::connect_with_config(&FluvioClusterConfig::new(format!("127.0.0.1:{sc_port}")))
        .await
        .unwrap();

    println!("SC running on localhost:{sc_port}");
    println!("SPU running on localhost:{spu_port}");

    assert!(sc_port > 0);
    assert!(spu_port > 0);
}
