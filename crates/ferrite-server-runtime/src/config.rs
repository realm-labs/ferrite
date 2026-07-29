//! Versioned process, role, endpoint, discovery, and capacity configuration.

use ferrite_region_runtime::lattice::authority::{LatticeNodeIdentity, RegionAuthorityError};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fs;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const SERVER_CONFIG_SCHEMA: u16 = 1;
pub const REGION_PLACEMENT_DOMAIN: &str = "ferrite-region-v1";
const MAX_CLUSTER_NODES: u16 = 64;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServerConfig {
    pub schema_version: u16,
    pub cluster: ClusterConfig,
    pub node: NodeConfig,
    pub remoting: RemotingConfig,
    pub discovery: DiscoveryConfig,
    pub placement: PlacementConfig,
    pub storage: StorageConfig,
    pub management: ManagementConfig,
    pub minecraft: MinecraftConfig,
    pub limits: AdmissionLimits,
    pub shutdown: ShutdownConfig,
}

impl ServerConfig {
    pub fn load(path: impl AsRef<Path>) -> Result<ValidatedServerConfig, ConfigError> {
        let path = path.as_ref();
        let text = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::from_toml(&text)
    }

    pub fn from_toml(text: &str) -> Result<ValidatedServerConfig, ConfigError> {
        let mut config: Self = toml::from_str(text)?;
        config.resolve_environment(&|name| std::env::var(name))?;
        config.validate()
    }

    pub fn to_toml(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(self).map_err(ConfigError::Serialize)
    }

    pub fn development_node(
        node_index: u16,
        node_count: u16,
        base_port: u16,
        state_root: &Path,
    ) -> Result<Self, ConfigError> {
        if node_index == 0 || node_index > node_count {
            return Err(ConfigError::Invalid(
                "development node index must be within 1..=node_count".to_owned(),
            ));
        }
        if !(1..=MAX_CLUSTER_NODES).contains(&node_count) {
            return Err(ConfigError::Invalid(format!(
                "development node count must be within 1..={MAX_CLUSTER_NODES}"
            )));
        }
        let node_offset = (node_index - 1)
            .checked_mul(3)
            .ok_or_else(|| ConfigError::Invalid("development port offset overflowed".to_owned()))?;
        let remoting_port = checked_port(base_port, node_offset)?;
        let management_port = checked_port(base_port, node_offset + 1)?;
        let minecraft_port = checked_port(base_port, node_offset + 2)?;
        let peers = (1..=node_count)
            .map(|index| {
                let offset = (index - 1) * 3;
                Ok(AdvertisedAddress {
                    host: "127.0.0.1".to_owned(),
                    port: checked_port(base_port, offset)?,
                })
            })
            .collect::<Result<Vec<_>, ConfigError>>()?;

        Ok(Self {
            schema_version: SERVER_CONFIG_SCHEMA,
            cluster: ClusterConfig {
                name: "ferrite-dev".to_owned(),
            },
            node: NodeConfig {
                id: format!("dev-node-{node_index}"),
                incarnation: None,
                roles: vec![
                    NodeRole::Gateway,
                    NodeRole::RegionWorker,
                    NodeRole::CoordinatorCandidate,
                    NodeRole::Administration,
                ],
            },
            remoting: RemotingConfig {
                bind: SocketAddr::from(([127, 0, 0, 1], remoting_port)),
                advertise: AdvertisedAddress {
                    host: "127.0.0.1".to_owned(),
                    port: remoting_port,
                },
            },
            discovery: DiscoveryConfig::DevelopmentStatic {
                peers,
                minimum_members: usize::from(node_count),
            },
            placement: PlacementConfig {
                capacity_regions: 256,
                required_domains: vec![REGION_PLACEMENT_DOMAIN.to_owned()],
            },
            storage: StorageConfig {
                root: state_root.join(format!("node-{node_index}")),
            },
            management: ManagementConfig {
                bind: SocketAddr::from(([127, 0, 0, 1], management_port)),
                allow_remote_drain: false,
            },
            minecraft: MinecraftConfig {
                enabled: true,
                bind: SocketAddr::from(([127, 0, 0, 1], minecraft_port)),
            },
            limits: AdmissionLimits {
                max_sessions: 4_096,
                max_region_mailbox: 8_192,
                max_management_request_bytes: 4_096,
            },
            shutdown: ShutdownConfig {
                drain_timeout_millis: 10_000,
            },
        })
    }

    fn resolve_environment(
        &mut self,
        read: &impl Fn(&str) -> Result<String, std::env::VarError>,
    ) -> Result<(), ConfigError> {
        self.node.id = resolve_template(&self.node.id, read)?;
        self.remoting.advertise.host = resolve_template(&self.remoting.advertise.host, read)?;
        if let DiscoveryConfig::Kubernetes {
            namespace, service, ..
        } = &mut self.discovery
        {
            *namespace = resolve_template(namespace, read)?;
            *service = resolve_template(service, read)?;
        }
        Ok(())
    }

    fn validate(self) -> Result<ValidatedServerConfig, ConfigError> {
        if self.schema_version != SERVER_CONFIG_SCHEMA {
            return Err(ConfigError::Invalid(format!(
                "unsupported server configuration schema {}, expected {SERVER_CONFIG_SCHEMA}",
                self.schema_version
            )));
        }
        validate_name("cluster name", &self.cluster.name)?;
        validate_name("node id", &self.node.id)?;
        validate_roles(&self)?;
        validate_endpoints(&self)?;
        validate_discovery(&self.discovery)?;
        validate_placement(&self)?;
        validate_limits(&self)?;
        if self.storage.root.as_os_str().is_empty() {
            return Err(ConfigError::Invalid(
                "storage root cannot be empty".to_owned(),
            ));
        }
        if self.shutdown.drain_timeout_millis == 0 {
            return Err(ConfigError::Invalid(
                "drain timeout must be positive".to_owned(),
            ));
        }
        let identity = match self.node.incarnation {
            Some(incarnation) => LatticeNodeIdentity::new(
                self.node.id.clone(),
                self.remoting.advertise.host.clone(),
                self.remoting.advertise.port,
                incarnation,
            ),
            None => LatticeNodeIdentity::generate(
                self.node.id.clone(),
                self.remoting.advertise.host.clone(),
                self.remoting.advertise.port,
            ),
        }
        .map_err(ConfigError::NodeIdentity)?;
        Ok(ValidatedServerConfig {
            config: self,
            identity,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedServerConfig {
    config: ServerConfig,
    identity: LatticeNodeIdentity,
}

impl ValidatedServerConfig {
    pub const fn config(&self) -> &ServerConfig {
        &self.config
    }

    pub const fn incarnation(&self) -> u128 {
        self.identity.incarnation()
    }

    pub fn node_identity(&self) -> Result<LatticeNodeIdentity, ConfigError> {
        Ok(self.identity.clone())
    }

    pub fn required_domains(&self) -> BTreeSet<String> {
        self.config
            .placement
            .required_domains
            .iter()
            .cloned()
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClusterConfig {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeConfig {
    pub id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incarnation: Option<u128>,
    pub roles: Vec<NodeRole>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NodeRole {
    Gateway,
    RegionWorker,
    CoordinatorCandidate,
    Administration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemotingConfig {
    pub bind: SocketAddr,
    pub advertise: AdvertisedAddress,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdvertisedAddress {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "kebab-case")]
pub enum DiscoveryConfig {
    DevelopmentStatic {
        peers: Vec<AdvertisedAddress>,
        minimum_members: usize,
    },
    Kubernetes {
        namespace: String,
        service: String,
        remoting_port: u16,
        minimum_members: usize,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlacementConfig {
    pub capacity_regions: u32,
    pub required_domains: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StorageConfig {
    pub root: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagementConfig {
    pub bind: SocketAddr,
    #[serde(default)]
    pub allow_remote_drain: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinecraftConfig {
    pub enabled: bool,
    pub bind: SocketAddr,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissionLimits {
    pub max_sessions: usize,
    pub max_region_mailbox: usize,
    pub max_management_request_bytes: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShutdownConfig {
    pub drain_timeout_millis: u64,
}

fn validate_name(label: &str, value: &str) -> Result<(), ConfigError> {
    let valid = !value.is_empty()
        && value.len() <= 63
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        && value
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_alphanumeric)
        && value
            .as_bytes()
            .last()
            .is_some_and(u8::is_ascii_alphanumeric);
    if valid {
        Ok(())
    } else {
        Err(ConfigError::Invalid(format!(
            "{label} must be 1..=63 ASCII alphanumeric or hyphen characters and cannot begin or end with a hyphen"
        )))
    }
}

fn validate_roles(config: &ServerConfig) -> Result<(), ConfigError> {
    let roles = config.node.roles.iter().copied().collect::<BTreeSet<_>>();
    if roles.is_empty() {
        return Err(ConfigError::Invalid(
            "a node must declare at least one role".to_owned(),
        ));
    }
    if roles.len() != config.node.roles.len() {
        return Err(ConfigError::Invalid(
            "node roles must not contain duplicates".to_owned(),
        ));
    }
    let worker = roles.contains(&NodeRole::RegionWorker);
    if worker != (config.placement.capacity_regions > 0) {
        return Err(ConfigError::Invalid(
            "Region worker role and non-zero placement capacity must be declared together"
                .to_owned(),
        ));
    }
    if config.minecraft.enabled != roles.contains(&NodeRole::Gateway) {
        return Err(ConfigError::Invalid(
            "gateway role and enabled Minecraft listener must be declared together".to_owned(),
        ));
    }
    Ok(())
}

fn validate_endpoints(config: &ServerConfig) -> Result<(), ConfigError> {
    validate_bind("remoting", config.remoting.bind)?;
    validate_bind("management", config.management.bind)?;
    if config.minecraft.enabled {
        validate_bind("Minecraft", config.minecraft.bind)?;
    }
    validate_advertised("remoting advertise", &config.remoting.advertise)?;
    let mut binds = vec![config.remoting.bind, config.management.bind];
    if config.minecraft.enabled {
        binds.push(config.minecraft.bind);
    }
    for (index, left) in binds.iter().enumerate() {
        for right in &binds[index + 1..] {
            if left == right {
                return Err(ConfigError::Invalid(format!(
                    "process listeners cannot share bind address {left}"
                )));
            }
        }
    }
    Ok(())
}

fn validate_bind(label: &str, address: SocketAddr) -> Result<(), ConfigError> {
    if address.port() == 0 {
        Err(ConfigError::Invalid(format!(
            "{label} bind port cannot be zero"
        )))
    } else {
        Ok(())
    }
}

fn validate_advertised(label: &str, address: &AdvertisedAddress) -> Result<(), ConfigError> {
    if address.port == 0 || address.host.is_empty() || address.host.len() > 253 {
        return Err(ConfigError::Invalid(format!(
            "{label} must contain a non-empty host and non-zero port"
        )));
    }
    if address
        .host
        .parse::<IpAddr>()
        .is_ok_and(|ip| ip.is_unspecified())
    {
        return Err(ConfigError::Invalid(format!(
            "{label} cannot use an unspecified IP address"
        )));
    }
    Ok(())
}

fn validate_discovery(discovery: &DiscoveryConfig) -> Result<(), ConfigError> {
    match discovery {
        DiscoveryConfig::DevelopmentStatic {
            peers,
            minimum_members,
        } => {
            if peers.is_empty() || *minimum_members == 0 || *minimum_members > peers.len() {
                return Err(ConfigError::Invalid(
                    "development-static discovery requires peers and a minimum within the peer count"
                        .to_owned(),
                ));
            }
            let unique = peers.iter().collect::<BTreeSet<_>>();
            if unique.len() != peers.len() {
                return Err(ConfigError::Invalid(
                    "development-static discovery peers must be unique".to_owned(),
                ));
            }
            for peer in peers {
                validate_advertised("discovery peer", peer)?;
            }
        }
        DiscoveryConfig::Kubernetes {
            namespace,
            service,
            remoting_port,
            minimum_members,
        } => {
            validate_name("Kubernetes namespace", namespace)?;
            validate_name("Kubernetes service", service)?;
            if *remoting_port == 0 || *minimum_members == 0 {
                return Err(ConfigError::Invalid(
                    "Kubernetes discovery port and minimum membership must be positive".to_owned(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_placement(config: &ServerConfig) -> Result<(), ConfigError> {
    let domains = config
        .placement
        .required_domains
        .iter()
        .collect::<BTreeSet<_>>();
    if domains.len() != config.placement.required_domains.len() {
        return Err(ConfigError::Invalid(
            "required placement domains must be unique".to_owned(),
        ));
    }
    for domain in domains {
        validate_name("placement domain", domain)?;
    }
    if config.placement.capacity_regions > 0
        && !config
            .placement
            .required_domains
            .iter()
            .any(|domain| domain == REGION_PLACEMENT_DOMAIN)
    {
        return Err(ConfigError::Invalid(format!(
            "Region workers must require the {REGION_PLACEMENT_DOMAIN} placement domain"
        )));
    }
    Ok(())
}

fn validate_limits(config: &ServerConfig) -> Result<(), ConfigError> {
    let limits = &config.limits;
    if limits.max_sessions == 0
        || limits.max_region_mailbox == 0
        || !(512..=65_536).contains(&limits.max_management_request_bytes)
    {
        return Err(ConfigError::Invalid(
            "admission limits must be positive and management requests must be within 512..=65536 bytes"
                .to_owned(),
        ));
    }
    Ok(())
}

fn resolve_template(
    value: &str,
    read: &impl Fn(&str) -> Result<String, std::env::VarError>,
) -> Result<String, ConfigError> {
    if !value.contains("${") {
        return Ok(value.to_owned());
    }
    if !value.starts_with("${")
        || !value.ends_with('}')
        || value[2..value.len() - 1].contains(['$', '{', '}'])
    {
        return Err(ConfigError::Invalid(format!(
            "environment template must be one exact ${{NAME}} value: {value}"
        )));
    }
    let name = &value[2..value.len() - 1];
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err(ConfigError::Invalid(format!(
            "environment variable name is invalid: {name}"
        )));
    }
    read(name).map_err(|_| ConfigError::MissingEnvironment(name.to_owned()))
}

fn checked_port(base: u16, offset: u16) -> Result<u16, ConfigError> {
    base.checked_add(offset)
        .filter(|port| *port != 0)
        .ok_or_else(|| ConfigError::Invalid("development port range overflowed".to_owned()))
}

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("read server configuration {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("parse server configuration: {0}")]
    Parse(#[from] toml::de::Error),
    #[error("serialize server configuration: {0}")]
    Serialize(toml::ser::Error),
    #[error("invalid server configuration: {0}")]
    Invalid(String),
    #[error("server configuration requires environment variable {0}")]
    MissingEnvironment(String),
    #[error("construct Lattice node identity: {0}")]
    NodeIdentity(RegionAuthorityError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn development_configs_are_complete_and_non_colliding() {
        let root = Path::new("target/dev-cluster-test");
        let configs = (1..=3)
            .map(|index| ServerConfig::development_node(index, 3, 27_000, root).unwrap())
            .collect::<Vec<_>>();
        let ports = configs
            .iter()
            .flat_map(|config| {
                [
                    config.remoting.bind.port(),
                    config.management.bind.port(),
                    config.minecraft.bind.port(),
                ]
            })
            .collect::<BTreeSet<_>>();
        assert_eq!(ports.len(), 9);
        for config in configs {
            let encoded = config.to_toml().unwrap();
            let validated = ServerConfig::from_toml(&encoded).unwrap();
            assert_ne!(validated.incarnation(), 0);
            assert_eq!(
                validated.required_domains(),
                BTreeSet::from([REGION_PLACEMENT_DOMAIN.to_owned()])
            );
            validated.node_identity().unwrap();
        }
    }

    #[test]
    fn schema_roles_addresses_and_bounds_fail_closed() {
        let mut config =
            ServerConfig::development_node(1, 1, 28_000, Path::new("target/node")).unwrap();
        config.schema_version = 2;
        assert!(config.validate().is_err());

        let mut config =
            ServerConfig::development_node(1, 1, 28_000, Path::new("target/node")).unwrap();
        config.node.roles.push(NodeRole::Gateway);
        assert!(config.validate().is_err());

        let mut config =
            ServerConfig::development_node(1, 1, 28_000, Path::new("target/node")).unwrap();
        config.limits.max_region_mailbox = 0;
        assert!(config.validate().is_err());

        let mut config =
            ServerConfig::development_node(1, 1, 28_000, Path::new("target/node")).unwrap();
        config.remoting.advertise.host = "0.0.0.0".to_owned();
        assert!(config.validate().is_err());
    }

    #[test]
    fn environment_templates_are_exact_and_bounded() {
        let read = |name: &str| match name {
            "POD_NAME" => Ok("ferrite-2".to_owned()),
            _ => Err(std::env::VarError::NotPresent),
        };
        assert_eq!(resolve_template("${POD_NAME}", &read).unwrap(), "ferrite-2");
        assert!(resolve_template("node-${POD_NAME}", &read).is_err());
        assert!(resolve_template("${missing}", &read).is_err());
        assert!(resolve_template("${POD_IP}", &read).is_err());
    }
}
