use async_trait::async_trait;

use crate::peer::PeerInfo;

#[async_trait]
pub trait Discovery: Send + Sync {
    async fn discover(&self) -> anyhow::Result<Vec<PeerInfo>>;
    async fn announce(&self) -> anyhow::Result<()>;
    async fn add_bootstrap_nodes(&mut self, nodes: Vec<String>);
}

pub struct LocalDiscovery {
    bootstrap_nodes: Vec<String>,
}

impl LocalDiscovery {
    pub fn new() -> Self {
        Self {
            bootstrap_nodes: Vec::new(),
        }
    }
}

impl Default for LocalDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Discovery for LocalDiscovery {
    async fn discover(&self) -> anyhow::Result<Vec<PeerInfo>> {
        Ok(Vec::new())
    }

    async fn announce(&self) -> anyhow::Result<()> {
        tracing::debug!("announcing presence on local network (stub)");
        Ok(())
    }

    async fn add_bootstrap_nodes(&mut self, nodes: Vec<String>) {
        self.bootstrap_nodes.extend(nodes);
    }
}

pub struct BootstrapDiscovery {
    bootstrap_nodes: Vec<String>,
}

impl BootstrapDiscovery {
    pub fn new(bootstrap_nodes: Vec<String>) -> Self {
        Self { bootstrap_nodes }
    }
}

#[async_trait]
impl Discovery for BootstrapDiscovery {
    async fn discover(&self) -> anyhow::Result<Vec<PeerInfo>> {
        tracing::debug!("discovering peers from {} bootstrap nodes", self.bootstrap_nodes.len());
        Ok(Vec::new())
    }

    async fn announce(&self) -> anyhow::Result<()> {
        tracing::debug!("announcing presence to bootstrap nodes (stub)");
        Ok(())
    }

    async fn add_bootstrap_nodes(&mut self, nodes: Vec<String>) {
        self.bootstrap_nodes.extend(nodes);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_discovery_discover_returns_empty() {
        let discovery = LocalDiscovery::new();
        let peers = discovery.discover().await.unwrap();
        assert!(peers.is_empty());
    }

    #[tokio::test]
    async fn test_local_discovery_announce_succeeds() {
        let discovery = LocalDiscovery::new();
        assert!(discovery.announce().await.is_ok());
    }

    #[tokio::test]
    async fn test_local_discovery_add_bootstrap_nodes() {
        let mut discovery = LocalDiscovery::new();
        assert!(discovery.bootstrap_nodes.is_empty());
        discovery.add_bootstrap_nodes(vec!["node1:8080".to_string()]).await;
        assert_eq!(discovery.bootstrap_nodes.len(), 1);
    }

    #[tokio::test]
    async fn test_bootstrap_discovery_holds_nodes() {
        let nodes = vec!["seed1:8080".to_string(), "seed2:8080".to_string()];
        let mut discovery = BootstrapDiscovery::new(nodes.clone());
        assert_eq!(discovery.bootstrap_nodes, nodes);

        discovery.add_bootstrap_nodes(vec!["seed3:8080".to_string()]).await;
        assert_eq!(discovery.bootstrap_nodes.len(), 3);
    }

    #[tokio::test]
    async fn test_bootstrap_discovery_discover_returns_empty() {
        let discovery = BootstrapDiscovery::new(vec!["seed:8080".to_string()]);
        let peers = discovery.discover().await.unwrap();
        assert!(peers.is_empty());
    }
}
