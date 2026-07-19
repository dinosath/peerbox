use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

fn default_node_name() -> String {
    "peerbox-node".to_string()
}

fn default_data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("peerbox")
}

fn default_database_url() -> String {
    let db_path = dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("peerbox")
        .join("database.sqlite");
    format!("sqlite://{}?mode=rwc", db_path.display())
}

fn default_listen_port() -> u16 {
    0
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerBoxConfig {
    #[serde(default = "default_node_name")]
    pub node_name: String,

    #[serde(default = "default_data_dir")]
    pub data_dir: PathBuf,

    #[serde(default = "default_database_url")]
    pub database_url: String,

    #[serde(default = "default_storage_path")]
    pub storage_dir: PathBuf,

    #[serde(default = "default_listen_port")]
    pub listen_port: u16,

    #[serde(default)]
    pub bootstrap_nodes: Vec<String>,

    #[serde(default)]
    pub federation_enabled: bool,

    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_storage_path() -> PathBuf {
    default_data_dir().join("storage")
}

impl Default for PeerBoxConfig {
    fn default() -> Self {
        let data_dir = default_data_dir();
        Self {
            node_name: default_node_name(),
            data_dir: data_dir.clone(),
            database_url: default_database_url(),
            storage_dir: data_dir.join("storage"),
            listen_port: default_listen_port(),
            bootstrap_nodes: Vec::new(),
            federation_enabled: false,
            log_level: default_log_level(),
        }
    }
}

impl PeerBoxConfig {
    pub fn load() -> Result<Self> {
        let path = Self::default_config_path();
        Self::load_from_path(&path)
    }

    pub fn load_from_path(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .with_context(|| format!("failed to read config from {}", path.display()))?;
            serde_json::from_str(&content)
                .with_context(|| format!("failed to parse config from {}", path.display()))
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::default_config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("failed to create config dir {}", parent.display()))?;
        }
        let content = serde_json::to_string_pretty(self).context("failed to serialize config")?;
        std::fs::write(&path, content)
            .with_context(|| format!("failed to write config to {}", path.display()))?;
        Ok(())
    }

    pub fn default_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("peerbox")
            .join("config.json")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = PeerBoxConfig::default();
        assert_eq!(config.node_name, "peerbox-node");
        assert_eq!(config.listen_port, 0);
        assert_eq!(config.bootstrap_nodes, Vec::<String>::new());
        assert!(!config.federation_enabled);
        assert_eq!(config.log_level, "info");
        assert!(config.data_dir.ends_with("peerbox"));
        assert!(config.storage_dir.ends_with("storage"));
        assert!(config.database_url.contains("sqlite://"));
        assert!(config.database_url.contains("mode=rwc"));
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut config = PeerBoxConfig::default();
        config.node_name = "test-node".to_string();
        config.listen_port = 8080;
        config.bootstrap_nodes = vec!["node1:8080".to_string(), "node2:8080".to_string()];
        config.federation_enabled = true;
        config.log_level = "debug".to_string();

        let json = serde_json::to_string_pretty(&config).unwrap();
        let deserialized: PeerBoxConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.node_name, "test-node");
        assert_eq!(deserialized.listen_port, 8080);
        assert_eq!(deserialized.bootstrap_nodes.len(), 2);
        assert!(deserialized.federation_enabled);
        assert_eq!(deserialized.log_level, "debug");
    }

    #[test]
    fn test_load_from_path() {
        let mut config = PeerBoxConfig::default();
        config.node_name = "loaded-node".to_string();
        config.listen_port = 9999;

        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        let json = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, json).unwrap();

        let loaded = PeerBoxConfig::load_from_path(&config_path).unwrap();
        assert_eq!(loaded.node_name, "loaded-node");
        assert_eq!(loaded.listen_port, 9999);
    }

    #[test]
    fn test_load_from_path_missing_file_returns_defaults() {
        let path = PathBuf::from("/nonexistent/config.json");
        let config = PeerBoxConfig::load_from_path(&path).unwrap();
        assert_eq!(config.node_name, "peerbox-node");
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.json");

        let mut config = PeerBoxConfig::default();
        config.node_name = "save-test".to_string();

        let json = serde_json::to_string_pretty(&config).unwrap();
        std::fs::write(&config_path, json).unwrap();

        let loaded = PeerBoxConfig::load_from_path(&config_path).unwrap();
        assert_eq!(loaded.node_name, "save-test");
    }

    #[test]
    fn test_default_config_path() {
        let path = PeerBoxConfig::default_config_path();
        assert!(path.ends_with("config.json"));
        assert!(path.to_str().unwrap().contains("peerbox"));
    }
}
