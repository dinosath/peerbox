use async_trait::async_trait;
use bytes::Bytes;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::connection::{Connection, ConnectionManager, StubConnection};
use crate::discovery::{Discovery, LocalDiscovery};
use crate::identity::NodeIdentity;
use crate::peer::PeerInfo;
use crate::transport::Transport;

pub struct IrohTransport {
    identity: NodeIdentity,
    connection_manager: ConnectionManager,
    discovery: Arc<RwLock<Box<dyn Discovery>>>,
}

impl IrohTransport {
    pub fn new() -> Self {
        let identity = NodeIdentity::generate();
        let discovery: Box<dyn Discovery> = Box::new(LocalDiscovery::new());
        Self {
            identity,
            connection_manager: ConnectionManager::default(),
            discovery: Arc::new(RwLock::new(discovery)),
        }
    }

    pub fn with_identity(identity: NodeIdentity) -> Self {
        let discovery: Box<dyn Discovery> = Box::new(LocalDiscovery::new());
        Self {
            identity,
            connection_manager: ConnectionManager::default(),
            discovery: Arc::new(RwLock::new(discovery)),
        }
    }

    pub fn with_identity_and_discovery(identity: NodeIdentity, discovery: Box<dyn Discovery>) -> Self {
        Self {
            identity,
            connection_manager: ConnectionManager::default(),
            discovery: Arc::new(RwLock::new(discovery)),
        }
    }

    pub fn node_id(&self) -> &common::NodeId {
        self.identity.node_id()
    }
}

impl Default for IrohTransport {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Transport for IrohTransport {
    async fn start(&self) -> anyhow::Result<()> {
        tracing::info!("starting IrohTransport for node {}", self.identity.node_id());
        Ok(())
    }

    async fn stop(&self) -> anyhow::Result<()> {
        tracing::info!("stopping IrohTransport for node {}", self.identity.node_id());
        Ok(())
    }

    async fn discover_peers(&self) -> anyhow::Result<Vec<PeerInfo>> {
        let discovery = self.discovery.read().await;
        discovery.discover().await
    }

    async fn connect(&self, peer: &PeerInfo) -> anyhow::Result<Box<dyn Connection>> {
        tracing::debug!("connecting to peer {}", peer.node_id);
        let conn = Box::new(StubConnection);
        let _ = self.connection_manager.add_connection(peer, conn).await;
        Ok(Box::new(StubConnection))
    }

    async fn send_chunk(&self, peer: &PeerInfo, chunk_index: u64, _data: Bytes) -> anyhow::Result<()> {
        tracing::debug!("sending chunk {} to peer {} (stub)", chunk_index, peer.node_id);
        Ok(())
    }

    async fn receive_chunk(&self, peer: &PeerInfo, chunk_index: u64) -> anyhow::Result<Option<Bytes>> {
        tracing::debug!("receiving chunk {} from peer {} (stub)", chunk_index, peer.node_id);
        Ok(None)
    }

    async fn is_connected(&self, peer: &PeerInfo) -> bool {
        self.connection_manager.is_connected(&peer.node_id.to_string()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::PeerInfo;

    #[tokio::test]
    async fn test_transport_start_stop() {
        let transport = IrohTransport::new();
        assert!(transport.start().await.is_ok());
        assert!(transport.stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_transport_discover_peers_returns_empty() {
        let transport = IrohTransport::new();
        transport.start().await.unwrap();
        let peers = transport.discover_peers().await.unwrap();
        assert!(peers.is_empty());
    }

    #[tokio::test]
    async fn test_transport_connect_returns_stub() {
        let transport = IrohTransport::new();
        transport.start().await.unwrap();

        let peer = PeerInfo::new(
            common::NodeId::new(),
            vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
            vec![1u8; 32],
        );

        let conn = transport.connect(&peer).await.unwrap();
        assert!(!conn.is_alive().await);
    }

    #[tokio::test]
    async fn test_transport_send_receive_chunk_stubs() {
        let transport = IrohTransport::new();
        transport.start().await.unwrap();

        let peer = PeerInfo::new(
            common::NodeId::new(),
            vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
            vec![1u8; 32],
        );

        assert!(transport.send_chunk(&peer, 0, Bytes::from("hello")).await.is_ok());
        let result = transport.receive_chunk(&peer, 0).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_transport_is_connected() {
        let transport = IrohTransport::new();
        let peer = PeerInfo::new(
            common::NodeId::new(),
            vec!["/ip4/127.0.0.1/tcp/8080".to_string()],
            vec![1u8; 32],
        );
        assert!(!transport.is_connected(&peer).await);
    }

    #[tokio::test]
    async fn test_transport_with_identity() {
        let identity = NodeIdentity::generate();
        let node_id = identity.node_id().clone();
        let transport = IrohTransport::with_identity(identity);
        assert_eq!(transport.node_id(), &node_id);
    }

    #[test]
    fn test_transport_default() {
        let transport = IrohTransport::default();
        assert!(!transport.node_id().0.is_empty());
    }
}
