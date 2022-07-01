use std::fs::File;

use clap::{ArgEnum, Parser};
use config::Config;
use derive_more::Display;
use ron::de::from_reader;
use utils::{delete_resources, K8sResource};

pub mod config;
pub mod resources;
pub mod utils;

#[derive(ArgEnum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SugarfungeChainType {
    Local,
    Testnet,
}

#[derive(ArgEnum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Display)]
pub enum SugarfungeResource {
    Api,
    Explorer,
    Ipfs,
    Keycloak,
    Node,
    Status,
    Ingress,
}

#[derive(ArgEnum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum CliAction {
    Create,
    Delete,
}

/// Manage your SugarFunge Infrastructure in Kubernetes
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    // Action to take on the service or the config file
    #[clap(arg_enum)]
    action: CliAction,

    /// Name of the service
    #[clap(arg_enum)]
    service: SugarfungeResource,

    // Namespace to apply the action
    #[clap(short, long, default_value = "default")]
    namespace: String,

    // Chain type to configure
    #[clap(long, arg_enum)]
    chain: Option<SugarfungeChainType>,

    // Configuration file when creating the service
    #[clap(long)]
    config: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut config = Config::default();

    if let Some(config_path) = cli.config {
        let file = File::open(config_path).expect("Cannot find config file");
        config = match from_reader(file) {
            Ok(x) => x,
            Err(e) => {
                println!("Failed to load config: {}", e);
                std::process::exit(1);
            }
        };
    }

    let mut chain = SugarfungeChainType::Local;

    if let Some(chain_type) = cli.chain {
        chain = chain_type;
    }

    let error_message = format!("failed to load config for {}", cli.service);

    match cli.service {
        SugarfungeResource::Api => match cli.action {
            CliAction::Create => {
                let api_config = config.api.expect(&error_message);
                resources::api::deployment(&cli.namespace, api_config).await?;
                Ok(())
            }
            CliAction::Delete => {
                let api_name = config.api.expect(&error_message).name;
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::Deployment,
                ];
                Ok(delete_resources(&cli.namespace, &api_name, resource_types).await?)
            }
        },
        SugarfungeResource::Explorer => match cli.action {
            CliAction::Create => {
                let explorer_config = config.explorer.expect(&error_message);
                resources::explorer::deployment(&cli.namespace, explorer_config).await?;
                Ok(())
            }
            CliAction::Delete => {
                let explorer_name = config.explorer.expect(&error_message).name;
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::Deployment,
                ];
                Ok(delete_resources(&cli.namespace, &explorer_name, resource_types).await?)
            }
        },
        SugarfungeResource::Ipfs => match cli.action {
            CliAction::Create => {
                let ipfs_config = config.ipfs.expect(&error_message);
                resources::ipfs::deployment(&cli.namespace, ipfs_config).await?;
                Ok(())
            }
            CliAction::Delete => {
                let ipfs_name = config.ipfs.expect(&error_message).name;
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::Secret,
                    K8sResource::Deployment,
                ];
                Ok(delete_resources(&cli.namespace, &ipfs_name, resource_types).await?)
            }
        },
        SugarfungeResource::Keycloak => match cli.action {
            CliAction::Create => {
                let keycloak_config = config.keycloak.expect(&error_message);
                resources::keycloak::deployment(&cli.namespace, keycloak_config).await?;
                Ok(())
            }
            CliAction::Delete => {
                let keycloak_name = config.keycloak.expect(&error_message).name;
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::Secret,
                    K8sResource::Deployment,
                ];
                Ok(delete_resources(&cli.namespace, &keycloak_name, resource_types).await?)
            }
        },
        SugarfungeResource::Node => match cli.action {
            CliAction::Create => {
                let node_config = config.node.expect(&error_message);
                resources::node::statefulset(&cli.namespace, chain, node_config).await?;
                Ok(())
            }
            CliAction::Delete => {
                let node_name = config.node.expect(&error_message).name;
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::StatefulSet,
                ];
                Ok(delete_resources(&cli.namespace, &node_name, resource_types).await?)
            }
        },
        SugarfungeResource::Status => match cli.action {
            CliAction::Create => {
                let status_config = config.status.expect(&error_message);
                resources::status::deployment(&cli.namespace, status_config).await?;
                Ok(())
            }
            CliAction::Delete => {
                let status_name = config.status.expect(&error_message).name;
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::Deployment,
                ];
                Ok(delete_resources(&cli.namespace, &status_name, resource_types).await?)
            }
        },
        SugarfungeResource::Ingress => match cli.action {
            CliAction::Create => {
                let resources: Vec<SugarfungeResource> = vec![
                    SugarfungeResource::Api,
                    SugarfungeResource::Explorer,
                    SugarfungeResource::Ipfs,
                    SugarfungeResource::Keycloak,
                    SugarfungeResource::Node,
                    SugarfungeResource::Status,
                ];
                resources::ingress::create(&cli.namespace, config, resources).await?;
                Ok(())
            }
            CliAction::Delete => {
                let ingress_name = config.ingress.expect(&error_message).name;
                let resource_types: Vec<K8sResource> = vec![K8sResource::Ingress];
                Ok(delete_resources(&cli.namespace, &ingress_name, resource_types).await?)
            }
        },
    }
}
