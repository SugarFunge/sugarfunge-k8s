use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        apps::v1::{StatefulSet, StatefulSetSpec, StatefulSetStatus},
        core::v1::{
            ConfigMapEnvSource, Container, ContainerPort, EmptyDirVolumeSource, EnvFromSource,
            PodSecurityContext, PodSpec, PodTemplateSpec, Secret, SecretVolumeSource, ServicePort,
            Volume, VolumeMount,
        },
    },
    apimachinery::pkg::{apis::meta::v1::LabelSelector, util::intstr::IntOrString},
};

use kube::{
    api::{Api, PostParams},
    core::ObjectMeta,
    Client,
};

use crate::{
    config::{ChainSpecExternal, NodeConfig},
    utils::{create_configmap, create_service, ServiceData},
    SugarfungeChainType,
};

pub const NAME: &str = "sf-node";

fn init_container(config: &ChainSpecExternal) -> Container {
    let volume_mount = VolumeMount {
        name: NAME.to_string() + "-config",
        mount_path: "/chainspec".to_string(),
        ..Default::default()
    };

    Container {
        name: NAME.to_owned() + "-config",
        image: Some(config.wget_image.to_string()),
        image_pull_policy: Some("IfNotPresent".to_string()),
        command: Some(vec![
            "wget".to_string(),
            "-O".to_string(),
            "/chainspec/customSpec.json".to_string(),
            config.chainspec_url.to_string(),
        ]),
        volume_mounts: Some(vec![volume_mount]),
        ..Default::default()
    }
}

fn container(chain_type: SugarfungeChainType, config: NodeConfig) -> Container {
    let env = EnvFromSource {
        config_map_ref: Some(ConfigMapEnvSource {
            name: Some(NAME.to_string()),
            optional: Some(false),
        }),
        ..Default::default()
    };

    let mut args = vec![
        "--".to_string() + &config.node_name,
        "--port=".to_string() + &config.p2p_port.to_string(),
        "--ws-port=".to_string() + &config.ws_port.to_string(),
        "--unsafe-ws-external".to_string(),
        "--unsafe-rpc-external".to_string(),
        "--rpc-methods=Unsafe".to_string(),
        "--rpc-cors=all".to_string(),
        "--prometheus-external".to_string(),
    ];

    if let Some(bootnode) = config.bootnode {
        if let Some(dns_url) = bootnode.dns_url {
            args.push(format!(
                "--bootnodes=/dns4/{}/tcp/{}/p2p/{}",
                dns_url, bootnode.p2p_port, bootnode.private_key
            ));
        } else if let Some(dns_ip) = bootnode.dns_ip {
            args.push(format!(
                "--bootnodes=/ip4/{}/tcp/{}/p2p/{}",
                dns_ip, bootnode.p2p_port, bootnode.private_key
            ));
        } else {
            args.push(format!(
                "--bootnodes=/ip4/127.0.0.1/tcp/{}/p2p/{}",
                bootnode.p2p_port, bootnode.private_key
            ));
        }
    }

    let ws_container_port = ContainerPort {
        name: Some("rpc-port".to_string()),
        container_port: config.ws_port,
        ..Default::default()
    };

    let p2p_container_port = ContainerPort {
        name: Some("p2p-port".to_string()),
        container_port: config.p2p_port,
        ..Default::default()
    };

    let prometheus_container_port = ContainerPort {
        name: Some("prometheus-port".to_string()),
        container_port: config.prometheus_port,
        ..Default::default()
    };

    let mut volume_mounts: Option<Vec<VolumeMount>> = None;

    if chain_type == SugarfungeChainType::Testnet {
        let mut chainspec_file_name = "customSpec.json".to_string();

        if let Some(file_name) = config.chainspec_file_name {
            chainspec_file_name = file_name.to_string();
        }

        volume_mounts = Some(vec![VolumeMount {
            name: NAME.to_owned() + "-config",
            mount_path: "/chainspec/".to_string() + &chainspec_file_name,
            sub_path: Some(chainspec_file_name.to_string()),
            ..Default::default()
        }]);
        args.push("--chain=/chainspec/".to_string() + &chainspec_file_name);
    }

    Container {
        env_from: Some(vec![env]),
        image: Some(config.image),
        image_pull_policy: Some("IfNotPresent".to_string()),
        name: NAME.to_string(),
        ports: Some(vec![
            ws_container_port,
            p2p_container_port,
            prometheus_container_port,
        ]),
        args: Some(args),
        volume_mounts,
        ..Default::default()
    }
}

pub async fn statefulset(
    namespace: &str,
    chain_type: SugarfungeChainType,
    config: NodeConfig,
) -> anyhow::Result<StatefulSet> {
    let client = Client::try_default().await?;

    let metadata = ObjectMeta {
        name: Some(NAME.to_string()),
        labels: Some(BTreeMap::from([(
            "app.kubernetes.io/name".to_string(),
            NAME.to_string(),
        )])),
        ..Default::default()
    };

    let mut init_containers: Option<Vec<Container>> = None;

    let mut volumes: Option<Vec<Volume>> = None;

    if chain_type == SugarfungeChainType::Testnet {
        // Check if the chainspec comes from an external url using the config file.
        // Otherwise check if a secret was created without the tool that contains the chainspec.
        if let Some(ref chainspec) = config.chainspec_ext {
            volumes = Some(vec![Volume {
                name: NAME.to_string() + "-config",
                empty_dir: Some(EmptyDirVolumeSource::default()),
                ..Default::default()
            }]);

            init_containers = Some(vec![init_container(chainspec)]);
        } else {
            let secrets: Api<Secret> = Api::namespaced(client.clone(), namespace);

            if secrets.get_opt(NAME).await?.is_none() {
                return Err(anyhow::Error::msg(format!(
                    "The secret {} does not exist",
                    NAME
                )));
            }

            volumes = Some(vec![Volume {
                name: NAME.to_string() + "-config",
                secret: Some(SecretVolumeSource {
                    secret_name: Some(NAME.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }]);
        }
    }

    let service_data = ServiceData {
        service_port: ServicePort {
            protocol: Some("TCP".to_string()),
            port: config.ws_port,
            target_port: Some(IntOrString::Int(config.ws_port)),
            ..Default::default()
        },
        name: NAME.to_string(),
        cluster_ip: Some("None".to_string()),
        ..Default::default()
    };

    let _service =
        create_service(client.clone(), namespace, metadata.clone(), service_data).await?;

    let configmap_data = BTreeMap::from([("CHAIN".to_string(), "sugarfunge".to_string())]);

    let _configmap =
        create_configmap(client.clone(), namespace, metadata.clone(), configmap_data).await?;

    let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);

    let container = container(chain_type, config);

    let node = StatefulSet {
        metadata: metadata.clone(),
        spec: Some(StatefulSetSpec {
            template: PodTemplateSpec {
                metadata: Some(metadata.clone()),
                spec: Some(PodSpec {
                    security_context: Some(PodSecurityContext {
                        fs_group: Some(1000),
                        ..Default::default()
                    }),
                    init_containers,
                    containers: vec![container],
                    volumes,
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
        status: Some(StatefulSetStatus::default()),
    };

    let pp = PostParams::default();
    statefulsets.create(&pp, &node).await.map_err(|e| e.into())
}
