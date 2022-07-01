use std::collections::BTreeMap;

use k8s_openapi::api::{
    core::v1::Service,
    networking::v1::{
        HTTPIngressPath, HTTPIngressRuleValue, Ingress, IngressBackend, IngressRule,
        IngressServiceBackend, IngressSpec, IngressStatus, IngressTLS, ServiceBackendPort,
    },
};
use kube::{api::PostParams, core::ObjectMeta, Api, Client};

use crate::{config::Config, SugarfungeResource};

pub async fn get_service_port(client: Client, namespace: &str, name: &str) -> i32 {
    let services: Api<Service> = Api::namespaced(client, namespace);

    match services.get_opt(name).await {
        Ok(result) => match result {
            Some(data) => {
                let spec_not_found = format!("{}: spec for the service is not defined", name);
                let port_not_found = format!("{}: port for the service is not defined", name);

                data.spec
                    .expect(&spec_not_found)
                    .ports
                    .expect(&port_not_found)[0]
                    .port
            }
            None => {
                println!("{}: service does not exist, failed to create ingress", name);
                std::process::exit(1);
            }
        },
        Err(e) => {
            println!("{}: error when getting the service: {}", name, e);
            std::process::exit(1);
        }
    }
}

pub async fn create(
    namespace: &str,
    config: Config,
    resources: Vec<SugarfungeResource>,
) -> anyhow::Result<Ingress> {
    let client = Client::try_default().await?;
    let ingress_res: Api<Ingress> = Api::namespaced(client.clone(), namespace);
    let ingress_error = format!("failed to load config for {}", SugarfungeResource::Ingress);
    let ingress_config = config.ingress.clone().expect(&ingress_error);
    let mut tls_hosts: Vec<String> = vec![];
    let mut rules: Vec<IngressRule> = vec![];

    for resource in resources {
        let mut service_name: String = "".to_string();
        let mut service_port: i32 = 80;
        let error_message = format!("failed to load config for {}", resource);

        match resource {
            SugarfungeResource::Api => {
                service_name = config.api.clone().expect(&error_message).name.to_string();
                service_port = get_service_port(client.clone(), namespace, &service_name).await;
            }
            SugarfungeResource::Explorer => {
                service_name = config
                    .explorer
                    .clone()
                    .expect(&error_message)
                    .name
                    .to_string();
                service_port = get_service_port(client.clone(), namespace, &service_name).await;
            }
            SugarfungeResource::Ipfs => {
                service_name = config.ipfs.clone().expect(&error_message).name.to_string();
                service_port = get_service_port(client.clone(), namespace, &service_name).await;
            }
            SugarfungeResource::Keycloak => {
                service_name = config
                    .keycloak
                    .clone()
                    .expect(&error_message)
                    .name
                    .to_string();
                service_port = get_service_port(client.clone(), namespace, &service_name).await;
            }
            SugarfungeResource::Node => {
                service_name = config.node.clone().expect(&error_message).name.to_string();
                service_port = get_service_port(client.clone(), namespace, &service_name).await;
            }
            SugarfungeResource::Status => {
                service_name = config
                    .status
                    .clone()
                    .expect(&error_message)
                    .name
                    .to_string();
                service_port = get_service_port(client.clone(), namespace, &service_name).await;
            }
            _ => {}
        }

        let path = HTTPIngressPath {
            backend: IngressBackend {
                service: Some(IngressServiceBackend {
                    name: service_name.to_owned(),
                    port: Some(ServiceBackendPort {
                        number: Some(service_port),
                        ..Default::default()
                    }),
                }),
                ..Default::default()
            },
            path: Some("/".to_string()),
            path_type: "Prefix".to_string(),
        };

        let service_name_parsed_as_host =
            service_name + "." + &ingress_config.host;

        let rule = IngressRule {
            host: Some(service_name_parsed_as_host.to_string()),
            http: Some(HTTPIngressRuleValue { paths: vec![path] }),
        };

        tls_hosts.push(service_name_parsed_as_host);
        rules.push(rule);
    }

    let tls = IngressTLS {
        hosts: Some(tls_hosts),
        secret_name: Some(ingress_config.tls_secret.to_string()),
    };

    let ingress = Ingress {
        metadata: ObjectMeta {
            name: Some(ingress_config.name.to_string()),
            labels: Some(BTreeMap::from([(
                "app.kubernetes.io/name".to_string(),
                ingress_config.name.to_string(),
            )])),
            annotations: Some(BTreeMap::from([(
                "cert-manager.io/cluster-issuer".to_string(),
                ingress_config.tls_issuer.to_string(),
            )])),
            ..Default::default()
        },
        spec: Some(IngressSpec {
            ingress_class_name: Some("nginx".to_string()),
            rules: Some(rules),
            tls: Some(vec![tls]),
            ..Default::default()
        }),
        status: Some(IngressStatus::default()),
    };

    let pp = PostParams::default();
    ingress_res
        .create(&pp, &ingress)
        .await
        .map_err(|e| e.into())
}
