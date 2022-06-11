use std::collections::BTreeMap;

use k8s_openapi::api::{
    apps::v1::{Deployment, StatefulSet},
    core::v1::{ConfigMap, Secret, Service, ServicePort, ServiceSpec},
};
use kube::{
    api::{DeleteParams, PostParams},
    core::ObjectMeta,
    Api, Client,
};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ServiceData {
    pub service_port: ServicePort,
    pub name: String,
    pub cluster_ip: Option<String>,
    pub service_type: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResourceType {
    Service,
    ConfigMap,
    Secret,
    Deployment,
    StatefulSet,
}

pub async fn create_service(
    client: Client,
    namespace: &str,
    metadata: ObjectMeta,
    service_data: ServiceData,
) -> anyhow::Result<Service> {
    let services: Api<Service> = Api::namespaced(client, namespace);

    let service = Service {
        metadata,
        spec: Some(ServiceSpec {
            ports: Some(vec![service_data.service_port]),
            selector: Some(BTreeMap::from([(
                "app.kubernetes.io/name".to_string(),
                service_data.name,
            )])),
            type_: Some("NodePort".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let pp = PostParams::default();
    services.create(&pp, &service).await.map_err(|e| e.into())
}

pub async fn create_secret(
    client: Client,
    namespace: &str,
    metadata: ObjectMeta,
    data: BTreeMap<String, String>,
) -> anyhow::Result<Secret> {
    let secrets: Api<Secret> = Api::namespaced(client, namespace);

    let secret = Secret {
        metadata,
        string_data: Some(data),
        ..Default::default()
    };

    let pp = PostParams::default();
    secrets.create(&pp, &secret).await.map_err(|e| e.into())
}

pub async fn create_configmap(
    client: Client,
    namespace: &str,
    metadata: ObjectMeta,
    data: BTreeMap<String, String>,
) -> anyhow::Result<ConfigMap> {
    let config_maps: Api<ConfigMap> = Api::namespaced(client, namespace);

    let config_map = ConfigMap {
        data: Some(data),
        metadata,
        ..Default::default()
    };

    let pp = PostParams::default();
    config_maps
        .create(&pp, &config_map)
        .await
        .map_err(|e| e.into())
}

pub async fn delete_resources(
    namespace: &str,
    name: &str,
    resources: Vec<ResourceType>,
) -> anyhow::Result<()> {
    let client = Client::try_default().await?;
    let dp = DeleteParams::default();

    for resource_type in resources {
        match resource_type {
            ResourceType::Service => {
                let services: Api<Service> = Api::namespaced(client.clone(), namespace);
                if services.get_opt(name).await.is_ok() {
                    services.delete(name, &dp).await?;
                }
            }
            ResourceType::ConfigMap => {
                let configmaps: Api<ConfigMap> = Api::namespaced(client.clone(), namespace);
                if configmaps.get_opt(name).await.is_ok() {
                    configmaps.delete(name, &dp).await?;
                }
            }
            ResourceType::Secret => {
                let secrets: Api<Secret> = Api::namespaced(client.clone(), namespace);
                if secrets.get_opt(name).await.is_ok() {
                    secrets.delete(name, &dp).await?;
                }
            }
            ResourceType::Deployment => {
                let deployments: Api<Deployment> = Api::namespaced(client.clone(), namespace);
                if deployments.get_opt(name).await.is_ok() {
                    deployments.delete(name, &dp).await?;
                }
            }
            ResourceType::StatefulSet => {
                let statefulsets: Api<StatefulSet> = Api::namespaced(client.clone(), namespace);
                if statefulsets.get_opt(name).await.is_ok() {
                    statefulsets.delete(name, &dp).await?;
                }
            }
        }
    }

    Ok(())
}
