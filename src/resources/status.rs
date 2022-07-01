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
    config::StatusConfig,
    utils::{create_configmap, create_service, ServiceData},
};

fn container(config: StatusConfig) -> Container {
    let env = EnvFromSource {
        config_map_ref: Some(ConfigMapEnvSource {
            name: Some(config.name.to_string()),
            optional: Some(false),
        }),
        ..Default::default()
    };

    let container_port = ContainerPort {
        container_port: 8000,
        ..Default::default()
    };

    Container {
        env_from: Some(vec![env]),
        image: Some("sugarfunge.azurecr.io/status:latest".to_string()),
        image_pull_policy: Some("IfNotPresent".to_string()),
        name: config.name.to_string(),
        ports: Some(vec![container_port]),
        ..Default::default()
    }
}

pub async fn deployment(namespace: &str, config: StatusConfig) -> anyhow::Result<Deployment> {
    let client = Client::try_default().await?;

    let metadata = ObjectMeta {
        name: Some(config.name.to_string()),
        labels: Some(BTreeMap::from([(
            "app.kubernetes.io/name".to_string(),
            config.name.to_string(),
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
        name: config.name.to_string(),
        service_type: Some("NodePort".to_string()),
        ..Default::default()
    };

    let _service =
        create_service(client.clone(), namespace, metadata.clone(), service_data).await?;

    let configmap_data = BTreeMap::from([
        ("PORT".to_string(), config.port.to_string()),
        (
            "REACT_APP_PROVIDER_SOCKET".to_string(),
            config.node_url.to_owned(),
        ),
    ]);

    let _configmap =
        create_configmap(client.clone(), namespace, metadata.clone(), configmap_data).await?;

    let deployments: Api<Deployment> = Api::namespaced(client, namespace);

    let container = container(config.clone());

    let status = Deployment {
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
                    config.name.to_string(),
                )])),
                ..Default::default()
            },
            ..Default::default()
        }),
        status: Some(DeploymentStatus::default()),
    };

    let pp = PostParams::default();
    deployments.create(&pp, &status).await.map_err(|e| e.into())
}
