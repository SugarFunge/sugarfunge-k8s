use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ApiConfig {
    pub image: String,
    pub port: i32,
    pub listen_url: String,
    pub node_url: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        Self {
            image: "sugarfunge.azurecr.io/api:latest".to_string(),
            port: 4000,
            listen_url: "http://0.0.0.0:4000".to_string(),
            node_url: "ws://sf-node:9944".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ExplorerConfig {
    pub image: String,
    pub port: i32,
    pub ws_url: String,
}

impl Default for ExplorerConfig {
    fn default() -> Self {
        Self {
            image: "sugarfunge.azurecr.io/explorer:latest".to_string(),
            port: 80,
            ws_url: "wss://node.sugarfunge.dev".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeycloakDatabaseConfig {
    pub db_database: String,
    pub db_user: String,
    pub db_password: String,
    pub db_address: String,
    pub db_port: i32,
    pub db_schema: String,
}

impl Default for KeycloakDatabaseConfig {
    fn default() -> Self {
        Self {
            db_database: "keycloak".to_string(),
            db_user: "keycloak".to_string(),
            db_password: "keycloak".to_string(),
            db_address: "sf-db-postgresql".to_string(),
            db_port: 5432,
            db_schema: "public".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct KeycloakConfig {
    pub image: String,
    pub wget_image: String,
    pub port: i32,
    pub db_config: KeycloakDatabaseConfig,
}

impl Default for KeycloakConfig {
    fn default() -> Self {
        Self {
            image: "quay.io/keycloak/keycloak:15.0.2".to_string(),
            wget_image: "vertexstudio.azurecr.io/wget:84cfc94ef093db2b20444b6a6793eb6ae6136602"
                .to_string(),
            port: 8080,
            db_config: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct BootNode {
    pub dns_url: String,
    pub p2p_port: i32,
    pub private_key: String,
}

impl Default for BootNode {
    fn default() -> Self {
        Self {
            dns_url: "sf-node-0.sf-node.default.svc.cluster.local".to_string(),
            p2p_port: 30334,
            private_key: "12D3KooWGzN9EZLNkxEVeApishpq8d3pzChPmw9jQ9kra3csTAhk".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub image: String,
    pub ws_port: i32,
    pub p2p_port: i32,
    pub node_name: String,
    pub bootnode: Option<BootNode>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            image: "sugarfunge.azurecr.io/node:latest".to_string(),
            ws_port: 9944,
            p2p_port: 30334,
            node_name: "alice".to_string(),
            bootnode: None,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct StatusConfig {
    pub image: String,
    pub port: i32,
    pub node_url: String,
}

impl Default for StatusConfig {
    fn default() -> Self {
        Self {
            image: "sugarfunge.azurecr.io/status:latest".to_string(),
            port: 8000,
            node_url: "wss://node.sugarfunge.dev".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub api: Option<ApiConfig>,
    pub explorer: Option<ExplorerConfig>,
    pub keycloak: Option<KeycloakConfig>,
    pub node: Option<NodeConfig>,
    pub status: Option<StatusConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api: Some(Default::default()),
            explorer: Some(Default::default()),
            keycloak: Some(Default::default()),
            node: Some(Default::default()),
            status: Some(Default::default()),
        }
    }
}
