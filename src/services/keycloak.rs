use std::collections::BTreeMap;

use k8s_openapi::{
    api::{
        apps::v1::{Deployment, DeploymentSpec, DeploymentStatus},
        core::v1::{
            ConfigMapEnvSource, Container, ContainerPort, EmptyDirVolumeSource, EnvFromSource,
            EnvVar, EnvVarSource, ExecAction, HTTPGetAction, Lifecycle, LifecycleHandler,
            ObjectFieldSelector, PodSpec, PodTemplateSpec, Probe, SecretEnvSource, ServicePort,
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
    config::KeycloakConfig,
    utils::ServiceData,
    utils::{create_configmap, create_secret, create_service},
};

pub const NAME: &str = "sf-keycloak";

fn init_container(config: KeycloakConfig) -> Container {
    let volume_mount = VolumeMount {
        name: "keycloak-config".to_string(),
        mount_path: "/opt/jboss/keycloak/standalone/configuration".to_string(),
        ..Default::default()
    };

    Container {
        name: "sf-keycloak-config".to_string(),
        image: Some(config.wget_image),
        image_pull_policy: Some("IfNotPresent".to_string()),
        command: Some(vec![
            "wget".to_string(),
            "-O".to_string(),
            "/opt/jboss/keycloak/standalone/configuration/standalone-ha.xml".to_string(),
            "https://raw.githubusercontent.com/VertexStudio/vx-template-app/main/keycloak/config/standalone-ha.xml".to_string(),
        ]),
        volume_mounts: Some(vec![volume_mount]),
        ..Default::default()
    }
}

fn container(config: KeycloakConfig) -> Container {
    let env_value = EnvVar {
        name: "HOST_IP".to_string(),
        value_from: Some(EnvVarSource {
            field_ref: Some(ObjectFieldSelector {
                api_version: Some("v1".to_string()),
                field_path: "status.podIP".to_string(),
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let env_configmap = EnvFromSource {
        config_map_ref: Some(ConfigMapEnvSource {
            name: Some(NAME.to_string()),
            optional: Some(false),
        }),
        ..Default::default()
    };

    let env_secret = EnvFromSource {
        secret_ref: Some(SecretEnvSource {
            name: Some(NAME.to_string()),
            optional: Some(false),
        }),
        ..Default::default()
    };

    let container_port = ContainerPort {
        container_port: config.port,
        ..Default::default()
    };

    let volume_mount = VolumeMount {
        name: "keycloak-config".to_string(),
        mount_path: "/opt/jboss/keycloak/standalone/configuration/standalone-ha.xml".to_string(),
        sub_path: Some("standalone-ha.xml".to_string()),
        ..Default::default()
    };

    Container {
        env: Some(vec![env_value]),
        env_from: Some(vec![env_configmap, env_secret]),
        image: Some(config.image.to_owned()),
        image_pull_policy: Some("IfNotPresent".to_string()),
        name: NAME.to_string(),
        ports: Some(vec![container_port]),
        command: Some(vec!["java".to_string()]),
        args: Some(vec![
            "-D[Standalone]".to_string(),
            "-server".to_string(),
            "-Xms64m".to_string(),
            "-Xmx512m".to_string(),
            "-XX:MetaspaceSize=96M".to_string(),
            "-XX:MaxMetaspaceSize=256m".to_string(),
            "-Djava.net.preferIPv4Stack=true".to_string(),
            "-Djboss.modules.system.pkgs=org.jboss.byteman".to_string(),
            "-Djava.awt.headless=true".to_string(),
            "--add-exports=java.base/sun.nio.ch=ALL-UNNAMED".to_string(),
            "--add-exports=jdk.unsupported/sun.misc=ALL-UNNAMED".to_string(),
            "--add-exports=jdk.unsupported/sun.reflect=ALL-UNNAMED".to_string(),
            "-Dorg.jboss.boot.log.file=/opt/jboss/keycloak/standalone/log/server.log".to_string(),
            "-Dlogging.configuration=file:/opt/jboss/keycloak/standalone/configuration/logging.properties".to_string(),
            "-jar".to_string(),
            "/opt/jboss/keycloak/jboss-modules.jar".to_string(),
            "-mp".to_string(),
            "/opt/jboss/keycloak/modules".to_string(),
            "org.jboss.as.standalone".to_string(),
            "-Djboss.home.dir=/opt/jboss/keycloak".to_string(),
            "-Djboss.server.base.dir=/opt/jboss/keycloak/standalone".to_string(),
            "-c=standalone-ha.xml".to_string(),
            "-Dkeycloak.profile.feature.upload_scripts=enabled".to_string(),
            "-Dkeycloak.profile.properties.profile=preview".to_string(),
            "-Dkeycloak.profile.properties.feature.account_api=enabled".to_string(),
            "-b=0.0.0.0".to_string(),
            "-bprivate=0.0.0.0".to_string(),
            "-bmanagement=0.0.0.0".to_string(),
            "-Djgroups.bind_addr=$(HOST_IP)".to_string()
        ]),
        liveness_probe: Some(Probe {
            http_get: Some(HTTPGetAction {
                path: Some("/auth/".to_string()),
                port: IntOrString::Int(config.port),
                ..Default::default()
            }),
            initial_delay_seconds: Some(30),
            timeout_seconds: Some(5),
            ..Default::default()
        }),
        readiness_probe: Some(Probe {
            http_get: Some(HTTPGetAction {
                path: Some("/auth/realms/master".to_string()),
                port: IntOrString::Int(config.port),
                ..Default::default()
            }),
            initial_delay_seconds: Some(30),
            timeout_seconds: Some(5),
            ..Default::default()
        }),
        termination_message_path: Some("/dev/termination-log".to_string()),
        termination_message_policy: Some("File".to_string()),
        lifecycle: Some(Lifecycle {
            pre_stop: Some(LifecycleHandler {
                exec: Some(ExecAction {
                    command: Some(vec![
                        "java".to_string(),
                        "--add-exports=java.base/sun.nio.ch=ALL-UNNAMED".to_string(),
                        "--add-exports=jdk.unsupported/sun.misc=ALL-UNNAMED".to_string(),
                        "--add-exports=jdk.unsupported/sun.reflect=ALL-UNNAMED".to_string(),
                        "-Djboss.modules.system.pkgs=com.sun.java.swing".to_string(),
                        "-Dcom.ibm.jsse2.overrideDefaultTLS=true".to_string(),
                        "-Dlogging.configuration=file:/opt/jboss/keycloak/bin/jboss-cli-logging.properties".to_string(),
                        "-jar".to_string(),
                        "/opt/jboss/keycloak/jboss-modules.jar".to_string(),
                        "-mp".to_string(),
                        "/opt/jboss/keycloak/modules".to_string(),
                        "org.jboss.as.cli".to_string(),
                        "--connect".to_string(),
                        "--commands=shutdown --timeout=20".to_string(),
                    ])
                }),
                ..Default::default()
            }),
            ..Default::default()
        }),
        volume_mounts: Some(vec![volume_mount]),
        ..Default::default()
    }
}

pub async fn deployment(namespace: &str, config: KeycloakConfig) -> anyhow::Result<Deployment> {
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

    let secret_data = BTreeMap::from([
        ("DB_USER".to_string(), config.db_config.db_user.to_owned()),
        (
            "DB_PASSWORD".to_string(),
            config.db_config.db_password.to_owned(),
        ),
    ]);

    let _secret = create_secret(client.clone(), namespace, metadata.clone(), secret_data).await?;

    let configmap_data = BTreeMap::from([
        ("DB_VENDOR".to_string(), "POSTGRES".to_string()),
        (
            "DB_ADDR".to_string(),
            config.db_config.db_address.to_owned(),
        ),
        (
            "DB_DATABASE".to_string(),
            config.db_config.db_database.to_owned(),
        ),
        (
            "DB_SCHEMA".to_string(),
            config.db_config.db_schema.to_owned(),
        ),
        ("DB_PORT".to_string(), config.db_config.db_port.to_string()),
    ]);

    let _configmap =
        create_configmap(client.clone(), namespace, metadata.clone(), configmap_data).await?;

    let deployments: Api<Deployment> = Api::namespaced(client, namespace);

    let init_container = init_container(config.clone());

    let container = container(config);

    let keycloak = Deployment {
        metadata: metadata.clone(),
        spec: Some(DeploymentSpec {
            template: PodTemplateSpec {
                metadata: Some(metadata.clone()),
                spec: Some(PodSpec {
                    containers: vec![container],
                    init_containers: Some(vec![init_container]),
                    volumes: Some(vec![Volume {
                        name: "keycloak-config".to_string(),
                        empty_dir: Some(EmptyDirVolumeSource::default()),
                        ..Default::default()
                    }]),
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
    deployments
        .create(&pp, &keycloak)
        .await
        .map_err(|e| e.into())
}
