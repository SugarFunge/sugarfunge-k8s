use std::fs::File;

use clap::{ArgEnum, Parser};
use config::Config;
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

#[derive(ArgEnum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
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

    match cli.service {
        SugarfungeResource::Api => match cli.action {
            CliAction::Create => {
                if let Some(api_config) = config.api {
                    resources::api::deployment(&cli.namespace, api_config).await?;
                    Ok(())
                } else {
                    println!("{}: failed to load config", resources::api::NAME);
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::Deployment,
                ];
                Ok(delete_resources(&cli.namespace, resources::api::NAME, resource_types).await?)
            }
        },
        SugarfungeResource::Explorer => match cli.action {
            CliAction::Create => {
                if let Some(explorer_config) = config.explorer {
                    resources::explorer::deployment(&cli.namespace, explorer_config).await?;
                    Ok(())
                } else {
                    println!("{}: failed to load config", resources::explorer::NAME);
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::Deployment,
                ];
                Ok(
                    delete_resources(&cli.namespace, resources::explorer::NAME, resource_types)
                        .await?,
                )
            }
        },
        SugarfungeResource::Ipfs => match cli.action {
            CliAction::Create => {
                if let Some(ipfs_config) = config.ipfs {
                    resources::ipfs::deployment(&cli.namespace, ipfs_config).await?;
                    Ok(())
                } else {
                    println!("{}: failed to load config", resources::ipfs::NAME);
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::Secret,
                    K8sResource::Deployment,
                ];
                Ok(delete_resources(&cli.namespace, resources::ipfs::NAME, resource_types).await?)
            }
        },
        SugarfungeResource::Keycloak => match cli.action {
            CliAction::Create => {
                if let Some(keycloak_config) = config.keycloak {
                    resources::keycloak::deployment(&cli.namespace, keycloak_config).await?;
                    Ok(())
                } else {
                    println!("{}: failed to load config", resources::keycloak::NAME);
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::Secret,
                    K8sResource::Deployment,
                ];
                Ok(
                    delete_resources(&cli.namespace, resources::keycloak::NAME, resource_types)
                        .await?,
                )
            }
        },
        SugarfungeResource::Node => match cli.action {
            CliAction::Create => {
                if let Some(node_config) = config.node {
                    resources::node::statefulset(&cli.namespace, chain, node_config).await?;
                    Ok(())
                } else {
                    println!("{}: failed to load config", resources::node::NAME);
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::StatefulSet,
                ];
                Ok(delete_resources(&cli.namespace, resources::node::NAME, resource_types).await?)
            }
        },
        SugarfungeResource::Status => match cli.action {
            CliAction::Create => {
                if let Some(status_config) = config.status {
                    resources::status::deployment(&cli.namespace, status_config).await?;
                    Ok(())
                } else {
                    println!("{}: failed to load config", resources::status::NAME);
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<K8sResource> = vec![
                    K8sResource::Service,
                    K8sResource::ConfigMap,
                    K8sResource::Deployment,
                ];
                Ok(
                    delete_resources(&cli.namespace, resources::status::NAME, resource_types)
                        .await?,
                )
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
                if let Some(ingress_config) = config.ingress {
                    resources::ingress::create(&cli.namespace, ingress_config, resources).await?;
                    Ok(())
                } else {
                    println!(
                        "{}: failed to load the ingress_host in the config file",
                        resources::ingress::NAME
                    );
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<K8sResource> = vec![K8sResource::Ingress];
                Ok(
                    delete_resources(&cli.namespace, resources::ingress::NAME, resource_types)
                        .await?,
                )
            }
        },
    }
}
