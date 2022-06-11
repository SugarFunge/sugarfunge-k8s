use std::fs::File;

use clap::{ArgEnum, Parser};
use config::Config;
use ron::de::from_reader;
use utils::{delete_resources, ResourceType};

pub mod config;
pub mod services;
pub mod utils;

#[derive(ArgEnum, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum SugarfungeService {
    Api,
    Explorer,
    Keycloak,
    Node,
    Status,
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
    service: SugarfungeService,

    // Namespace to apply the action
    #[clap(short, long, default_value = "default")]
    namespace: String,

    // Configuration file when creating the service
    #[clap(short)]
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

    match cli.service {
        SugarfungeService::Api => match cli.action {
            CliAction::Create => {
                if let Some(api_config) = config.api {
                    services::api::deployment(&cli.namespace, api_config).await?;
                    Ok(())
                } else {
                    println!("Failed to load API config");
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<ResourceType> = vec![
                    ResourceType::Service,
                    ResourceType::ConfigMap,
                    ResourceType::Deployment,
                ];
                Ok(delete_resources(&cli.namespace, services::api::NAME, resource_types).await?)
            }
        },
        SugarfungeService::Explorer => match cli.action {
            CliAction::Create => {
                if let Some(explorer_config) = config.explorer {
                    services::explorer::deployment(&cli.namespace, explorer_config).await?;
                    Ok(())
                } else {
                    println!("Failed to load Explorer config");
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<ResourceType> = vec![
                    ResourceType::Service,
                    ResourceType::ConfigMap,
                    ResourceType::Deployment,
                ];
                Ok(
                    delete_resources(&cli.namespace, services::explorer::NAME, resource_types)
                        .await?,
                )
            }
        },
        SugarfungeService::Keycloak => match cli.action {
            CliAction::Create => {
                if let Some(keycloak_config) = config.keycloak {
                    services::keycloak::deployment(&cli.namespace, keycloak_config).await?;
                    Ok(())
                } else {
                    println!("Failed to load Keycloak config");
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<ResourceType> = vec![
                    ResourceType::Service,
                    ResourceType::ConfigMap,
                    ResourceType::Secret,
                    ResourceType::Deployment,
                ];
                Ok(
                    delete_resources(&cli.namespace, services::keycloak::NAME, resource_types)
                        .await?,
                )
            }
        },
        SugarfungeService::Node => match cli.action {
            CliAction::Create => {
                if let Some(node_config) = config.node {
                    services::node::statefulset(&cli.namespace, node_config).await?;
                    Ok(())
                } else {
                    println!("Failed to load Node config");
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<ResourceType> = vec![
                    ResourceType::Service,
                    ResourceType::ConfigMap,
                    ResourceType::StatefulSet,
                ];
                Ok(delete_resources(&cli.namespace, services::node::NAME, resource_types).await?)
            }
        },
        SugarfungeService::Status => match cli.action {
            CliAction::Create => {
                if let Some(status_config) = config.status {
                    services::status::deployment(&cli.namespace, status_config).await?;
                    Ok(())
                } else {
                    println!("Failed to load Status config");
                    std::process::exit(1);
                }
            }
            CliAction::Delete => {
                let resource_types: Vec<ResourceType> = vec![
                    ResourceType::Service,
                    ResourceType::ConfigMap,
                    ResourceType::Deployment,
                ];
                Ok(
                    delete_resources(&cli.namespace, services::status::NAME, resource_types)
                        .await?,
                )
            }
        },
    }
}
