use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec, DeploymentStatus},
        core::v1::{
            ConfigMapVolumeSource, Container, ContainerPort, EmptyDirVolumeSource, PodSpec,
            PodTemplateSpec, Probe, ServicePort, TCPSocketAction, Volume, VolumeMount, SecretVolumeSource,
        },
    },
    apimachinery::pkg::{apis::meta::v1::LabelSelector, util::intstr::IntOrString},
};
use kube::{
    api::{ObjectMeta, PostParams},
    Api, Client,
};

use crate::{
    config::IpfsConfig,
    utils::{create_configmap, create_service, ServiceData, create_secret},
};

pub const NAME: &str = "sf-ipfs";

const CONFIG_FILE: &str = r#"
#!/bin/sh
set -e
set -x
user=ipfs

# First start with current persistent volume
[ -f $IPFS_PATH/version ] || {
    echo "No ipfs repo found in $IPFS_PATH. Initializing..."
    ipfs init
    ipfs config Addresses.API /ip4/0.0.0.0/tcp/5001
    ipfs config Addresses.Gateway /ip4/0.0.0.0/tcp/8080
    ipfs config Datastore.StorageMax 5GB
    ipfs config --json API.HTTPHeaders.Access-Control-Allow-Origin '["*"]'
    ipfs config --json API.HTTPHeaders.Access-Control-Allow-Methods '["PUT", "POST"]'
    chown -R ipfs $IPFS_PATH
}

# Check for the swarm key
[ -f $IPFS_PATH/swarm.key ] || {
    echo "No swarm.key found, copying from mounted secret"
    [ -f /swarm/swarm.key ] || {
        echo "No swarm.key found in IPFS secret... Exiting swarm configuration"
        exit 0
    }
    echo "Removing all bootstrap nodes..."
    ipfs bootstrap rm --all
    cp -v /swarm/swarm.key $IPFS_PATH/swarm.key
    chmod 600 $IPFS_PATH/swarm.key
    chown -R ipfs $IPFS_PATH
}

"#;

fn init_container(config: IpfsConfig) -> Container {

    let mut volume_mounts: Vec<VolumeMount> = vec![];

    let data_volume_mount = VolumeMount {
        name: NAME.to_string() + "-data",
        mount_path: "/data/ipfs".to_string(),
        ..Default::default()
    };

    volume_mounts.push(data_volume_mount);

    let config_volume_mount = VolumeMount {
        name: NAME.to_string() + "-config",
        mount_path: "/custom".to_string(),
        ..Default::default()
    };

    volume_mounts.push(config_volume_mount);

    if config.swarm_key.is_some() {
        let swarm_volume_mount = VolumeMount {
            name: NAME.to_string() + "-swarm",
            mount_path: "/swarm".to_string(),
            ..Default::default()
        };

        volume_mounts.push(swarm_volume_mount);
    }

    Container {
        image: Some(config.image),
        image_pull_policy: Some("IfNotPresent".to_string()),
        command: Some(vec![
            "sh".to_string(),
            "/custom/configure-ipfs.sh".to_string(),
        ]),
        name: "configure-".to_string() + NAME,
        volume_mounts: Some(volume_mounts),
        ..Default::default()
    }
}

fn container(config: IpfsConfig) -> Container {
    let swarm_tcp_port = ContainerPort {
        name: Some("swarm-tcp-port".to_string()),
        container_port: config.swarm_tcp_port,
        ..Default::default()
    };

    let swarm_upd_port = ContainerPort {
        name: Some("swarm-upd-port".to_string()),
        container_port: config.swarm_udp_port,
        ..Default::default()
    };

    let api_container_port = ContainerPort {
        name: Some("api-port".to_string()),
        container_port: config.api_port,
        ..Default::default()
    };

    let volume_mount = VolumeMount {
        name: NAME.to_string() + "-data",
        mount_path: "/data/ipfs".to_string(),
        ..Default::default()
    };

    Container {
        image: Some(config.image),
        image_pull_policy: Some("IfNotPresent".to_string()),
        name: NAME.to_string(),
        ports: Some(vec![swarm_tcp_port, swarm_upd_port, api_container_port]),
        volume_mounts: Some(vec![volume_mount]),
        liveness_probe: Some(Probe {
            tcp_socket: Some(TCPSocketAction {
                port: IntOrString::Int(config.swarm_tcp_port),
                ..Default::default()
            }),
            initial_delay_seconds: Some(30),
            timeout_seconds: Some(5),
            period_seconds: Some(15),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub async fn deployment(namespace: &str, config: IpfsConfig) -> anyhow::Result<Deployment> {
    let client = Client::try_default().await?;

    let mut volumes: Vec<Volume> = vec![];

    let metadata = ObjectMeta {
        name: Some(NAME.to_string()),
        labels: Some(BTreeMap::from([(
            "app.kubernetes.io/name".to_string(),
            NAME.to_string(),
        )])),
        ..Default::default()
    };

    let service_data = ServiceData {
        service_port: ServicePort {
            protocol: Some("TCP".to_string()),
            port: config.api_port,
            target_port: Some(IntOrString::Int(config.api_port)),
            ..Default::default()
        },
        name: NAME.to_string(),
        service_type: Some("NodePort".to_string()),
        ..Default::default()
    };

    let _service =
        create_service(client.clone(), namespace, metadata.clone(), service_data).await?;

    if let Some(ref swarm_key) = config.swarm_key {
        let key_file = "/key/swarm/psk/1.0.0/\n/base16/\n".to_string() + swarm_key;
        let secret_data = BTreeMap::from([("swarm.key".to_string(), key_file)]);

        let _secret = create_secret(client.clone(), namespace, metadata.clone(), secret_data).await?;

        let swarm_key_volume = Volume {
            name: NAME.to_string() + "-swarm",
            secret: Some(SecretVolumeSource {
                secret_name: Some(NAME.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
    
        volumes.push(swarm_key_volume);
    }

    let configmap_data =
        BTreeMap::from([("configure-ipfs.sh".to_string(), CONFIG_FILE.to_string())]);

    let _configmap =
        create_configmap(client.clone(), namespace, metadata.clone(), configmap_data).await?;

    let deployments: Api<Deployment> = Api::namespaced(client, namespace);

    let init_container = init_container(config.clone());

    let container = container(config);

    let data_volume = Volume {
        name: NAME.to_string() + "-data",
        empty_dir: Some(EmptyDirVolumeSource::default()),
        ..Default::default()
    };

    volumes.push(data_volume);

    let config_files_volume = Volume {
        name: NAME.to_string() + "-config",
        config_map: Some(ConfigMapVolumeSource {
            name: Some(NAME.to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    volumes.push(config_files_volume);

    let api = Deployment {
        metadata: metadata.clone(),
        spec: Some(DeploymentSpec {
            template: PodTemplateSpec {
                metadata: Some(metadata.clone()),
                spec: Some(PodSpec {
                    init_containers: Some(vec![init_container]),
                    containers: vec![container],
                    volumes: Some(volumes),
                    ..Default::default()
                }),
            },
            selector: LabelSelector {
                match_labels: Some(BTreeMap::from([(
                    "app.kubernetes.io/name".to_string(),
                    NAME.to_string(),
                )])),
                ..Default::default()
            },
            ..Default::default()
        }),
        status: Some(DeploymentStatus::default()),
    };

    let pp = PostParams::default();
    deployments.create(&pp, &api).await.map_err(|e| e.into())
}
