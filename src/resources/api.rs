use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec, DeploymentStatus},
        core::v1::{
            ConfigMapEnvSource, Container, ContainerPort, EnvFromSource, PodSpec, PodTemplateSpec,
            ServicePort,
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
    config::ApiConfig,
    utils::{create_configmap, create_service, ServiceData},
};

pub const NAME: &str = "sf-api";

fn container(config: ApiConfig) -> Container {
    let env = EnvFromSource {
        config_map_ref: Some(ConfigMapEnvSource {
            name: Some(NAME.to_string()),
            optional: Some(false),
        }),
        ..Default::default()
    };

    let container_port = ContainerPort {
        container_port: config.port,
        ..Default::default()
    };

    Container {
        env_from: Some(vec![env]),
        image: Some(config.image),
        image_pull_policy: Some("IfNotPresent".to_string()),
        name: NAME.to_string(),
        args: Some(vec![
            "-l".to_string(),
            config.listen_url,
            "-s".to_string(),
            "$(NODE_URL)".to_string(),
        ]),
        ports: Some(vec![container_port]),
        ..Default::default()
    }
}

pub async fn deployment(namespace: &str, config: ApiConfig) -> anyhow::Result<Deployment> {
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
            port: config.port,
            target_port: Some(IntOrString::Int(config.port)),
            ..Default::default()
        },
        name: NAME.to_string(),
        service_type: Some("NodePort".to_string()),
        ..Default::default()
    };

    let _service =
        create_service(client.clone(), namespace, metadata.clone(), service_data).await?;

    let configmap_data = BTreeMap::from([("NODE_URL".to_string(), config.node_url.to_owned())]);

    let _configmap =
        create_configmap(client.clone(), namespace, metadata.clone(), configmap_data).await?;

    let deployments: Api<Deployment> = Api::namespaced(client, namespace);

    let container = container(config);

    let api = Deployment {
        metadata: metadata.clone(),
        spec: Some(DeploymentSpec {
            template: PodTemplateSpec {
                metadata: Some(metadata.clone()),
                spec: Some(PodSpec {
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
        status: Some(DeploymentStatus::default()),
    };

    let pp = PostParams::default();
    deployments.create(&pp, &api).await.map_err(|e| e.into())
}
