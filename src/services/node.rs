use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        apps::v1::{StatefulSet, StatefulSetSpec, StatefulSetStatus},
        core::v1::{
            ConfigMapEnvSource, Container, ContainerPort, EnvFromSource, PodSecurityContext,
            PodSpec, PodTemplateSpec, ServicePort,
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
    config::NodeConfig,
    utils::{create_configmap, create_service, ServiceData},
};

pub const NAME: &str = "sf-node";

fn container(config: NodeConfig) -> Container {
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
    ];

    if let Some(bootnode) = config.bootnode {
        args.push(format!(
            "--bootnodes=/dns4/{}/tcp/{}/p2p/{}",
            bootnode.dns_url, bootnode.p2p_port, bootnode.private_key
        ));
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

    Container {
        env_from: Some(vec![env]),
        image: Some(config.image),
        image_pull_policy: Some("IfNotPresent".to_string()),
        name: NAME.to_string(),
        ports: Some(vec![ws_container_port, p2p_container_port]),
        args: Some(args),
        ..Default::default()
    }
}

pub async fn statefulset(namespace: &str, config: NodeConfig) -> anyhow::Result<StatefulSet> {
    let client = Client::try_default().await?;

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

    let statefulsets: Api<StatefulSet> = Api::namespaced(client, namespace);

    let container = container(config);

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
                    containers: vec![container],
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
