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
pub struct IpfsConfig {
    pub image: String,
    pub swarm_tcp_port: i32,
    pub swarm_udp_port: i32,
    pub api_port: i32,
    pub swarm_key: Option<String>,
}

impl Default for IpfsConfig {
    fn default() -> Self {
        Self {
            image: "ipfs/go-ipfs:v0.13.0".to_string(),
            swarm_tcp_port: 4001,
            swarm_udp_port: 4002,
            api_port: 5001,
            swarm_key: None,
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
    pub port: i32,
    pub admin_username: String,
    pub admin_password: String,
    pub db_config: KeycloakDatabaseConfig,
}

impl Default for KeycloakConfig {
    fn default() -> Self {
        Self {
            image: "quay.io/keycloak/keycloak:18.0.0".to_string(),
            port: 8080,
            admin_username: "keycloak".to_string(),
            admin_password: "keycloak".to_string(),
            db_config: Default::default(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct BootNode {
    pub dns_url: Option<String>,
    pub dns_ip: Option<String>,
    pub p2p_port: i32,
    pub private_key: String,
}

impl Default for BootNode {
    fn default() -> Self {
        Self {
            dns_url: Some("sf-node.default.svc.cluster.local".to_string()),
            dns_ip: None,
            p2p_port: 30334,
            private_key: "12D3KooWGzN9EZLNkxEVeApishpq8d3pzChPmw9jQ9kra3csTAhk".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct ChainSpecExternal {
    pub wget_image: String,
    pub chainspec_url: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct NodeConfig {
    pub image: String,
    pub ws_port: i32,
    pub p2p_port: i32,
    pub prometheus_port: i32,
    pub node_name: String,
    pub chainspec_file_name: Option<String>,
    pub chainspec_ext: Option<ChainSpecExternal>,
    pub bootnode: Option<BootNode>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            image: "sugarfunge.azurecr.io/node:latest".to_string(),
            ws_port: 9944,
            p2p_port: 30334,
            prometheus_port: 9090,
            node_name: "alice".to_string(),
            chainspec_file_name: Some("customSpec.json".to_string()),
            chainspec_ext: None,
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
pub struct IngressConfig {
    pub host: String,
    pub tls_secret: String,
    pub tls_issuer: String,
}

impl Default for IngressConfig {
    fn default() -> Self {
        Self {
            host: "demo.sugarfunge.dev".to_string(),
            tls_secret: "sf-ingress-tls".to_string(),
            tls_issuer: "letsencrypt-staging".to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub api: Option<ApiConfig>,
    pub explorer: Option<ExplorerConfig>,
    pub ipfs: Option<IpfsConfig>,
    pub keycloak: Option<KeycloakConfig>,
    pub node: Option<NodeConfig>,
    pub status: Option<StatusConfig>,
    pub ingress: Option<IngressConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            api: Some(Default::default()),
            explorer: Some(Default::default()),
            ipfs: Some(Default::default()),
            keycloak: Some(Default::default()),
            node: Some(Default::default()),
            status: Some(Default::default()),
            ingress: Some(Default::default()),
        }
    }
}
