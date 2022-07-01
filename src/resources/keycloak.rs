use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec, DeploymentStatus},
        core::v1::{
            ConfigMapEnvSource, Container, ContainerPort, EnvFromSource, HTTPGetAction, PodSpec,
            PodTemplateSpec, Probe, SecretEnvSource, ServicePort,
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
    config::KeycloakConfig,
    utils::ServiceData,
    utils::{create_configmap, create_secret, create_service},
};

fn container(config: KeycloakConfig) -> Container {
    let env_configmap = EnvFromSource {
        config_map_ref: Some(ConfigMapEnvSource {
            name: Some(config.name.to_string()),
            optional: Some(false),
        }),
        ..Default::default()
    };

    let env_secret = EnvFromSource {
        secret_ref: Some(SecretEnvSource {
            name: Some(config.name.to_string()),
            optional: Some(false),
        }),
        ..Default::default()
    };

    let container_port = ContainerPort {
        container_port: config.port,
        ..Default::default()
    };

    Container {
        env_from: Some(vec![env_configmap, env_secret]),
        image: Some(config.image.to_owned()),
        image_pull_policy: Some("IfNotPresent".to_string()),
        name: config.name.to_string(),
        ports: Some(vec![container_port]),
        args: Some(vec!["start-dev".to_string()]),
        liveness_probe: Some(Probe {
            http_get: Some(HTTPGetAction {
                path: Some("/health".to_string()),
                port: IntOrString::Int(config.port),
                ..Default::default()
            }),
            initial_delay_seconds: Some(30),
            timeout_seconds: Some(5),
            ..Default::default()
        }),
        readiness_probe: Some(Probe {
            http_get: Some(HTTPGetAction {
                path: Some("/realms/master".to_string()),
                port: IntOrString::Int(config.port),
                ..Default::default()
            }),
            initial_delay_seconds: Some(30),
            timeout_seconds: Some(5),
            ..Default::default()
        }),
        ..Default::default()
    }
}

pub async fn deployment(namespace: &str, config: KeycloakConfig) -> anyhow::Result<Deployment> {
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

    let secret_data = BTreeMap::from([
        (
            "KC_DB_USERNAME".to_string(),
            config.db_config.db_user.to_owned(),
        ),
        (
            "KC_DB_PASSWORD".to_string(),
            config.db_config.db_password.to_owned(),
        ),
        (
            "KEYCLOAK_ADMIN".to_string(),
            config.admin_username.to_owned(),
        ),
        (
            "KEYCLOAK_ADMIN_PASSWORD".to_string(),
            config.admin_password.to_owned(),
        ),
    ]);

    let _secret = create_secret(client.clone(), namespace, metadata.clone(), secret_data).await?;

    let configmap_data = BTreeMap::from([
        ("KC_DB".to_string(), "postgres".to_string()),
        ("KC_HEALTH_ENABLED".to_string(), "true".to_string()),
        (
            "KC_DB_URL_HOST".to_string(),
            config.db_config.db_address.to_owned(),
        ),
        (
            "KC_DB_URL_DATABASE".to_string(),
            config.db_config.db_database.to_owned(),
        ),
        (
            "KC_DB_SCHEMA".to_string(),
            config.db_config.db_schema.to_owned(),
        ),
        (
            "KC_DB_URL_PORT".to_string(),
            config.db_config.db_port.to_string(),
        ),
    ]);

    let _configmap =
        create_configmap(client.clone(), namespace, metadata.clone(), configmap_data).await?;

    let deployments: Api<Deployment> = Api::namespaced(client, namespace);

    let container = container(config.clone());

    let keycloak = Deployment {
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
    deployments
        .create(&pp, &keycloak)
        .await
        .map_err(|e| e.into())
}
