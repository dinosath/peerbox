use async_trait::async_trait;
use bytes::Bytes;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::peer::{ConnectionState, PeerInfo};

#[async_trait]
pub trait Connection: Send + Sync {
    async fn send(&self, data: Bytes) -> anyhow::Result<()>;
    async fn receive(&self) -> anyhow::Result<Option<Bytes>>;
    async fn close(&self) -> anyhow::Result<()>;
    async fn is_alive(&self) -> bool;
}

pub struct StubConnection;

#[async_trait]
impl Connection for StubConnection {
    async fn send(&self, _data: Bytes) -> anyhow::Result<()> {
        Ok(())
    }

    async fn receive(&self) -> anyhow::Result<Option<Bytes>> {
        Ok(None)
    }

    async fn close(&self) -> anyhow::Result<()> {
        Ok(())
    }

    async fn is_alive(&self) -> bool {
        false
    }
}

pub struct ConnectionManager {
    connections: Arc<RwLock<HashMap<String, (Box<dyn Connection>, ConnectionState)>>>,
    max_connections: usize,
}

impl ConnectionManager {
    pub fn new(max_connections: usize) -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            max_connections,
        }
    }

    pub async fn add_connection(&self, peer: &PeerInfo, conn: Box<dyn Connection>) -> bool {
        let mut connections = self.connections.write().await;
        if connections.len() >= self.max_connections {
            tracing::warn!("connection pool full, rejecting connection from {}", peer.node_id);
            return false;
        }
        connections.insert(peer.node_id.to_string(), (conn, ConnectionState::Connected));
        true
    }

    pub async fn remove_connection(&self, node_id: &str) {
        let mut connections = self.connections.write().await;
        connections.remove(node_id);
    }

    pub async fn get_connection(&self, node_id: &str) -> Option<(
        std::sync::Arc<dyn Connection>,
        ConnectionState,
    )> {
        let connections = self.connections.read().await;
        // Can't return references from RwLock, so this is a simplified API
        let _ = connections.get(node_id);
        None
    }

    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    pub async fn update_state(&self, node_id: &str, state: ConnectionState) {
        let mut connections = self.connections.write().await;
        if let Some((_, ref mut current_state)) = connections.get_mut(node_id) {
            *current_state = state;
        }
    }

    pub async fn is_connected(&self, node_id: &str) -> bool {
        let connections = self.connections.read().await;
        connections
            .get(node_id)
            .map(|(_, state)| *state == ConnectionState::Connected)
            .unwrap_or(false)
    }

    pub async fn active_connections(&self) -> Vec<String> {
        let connections = self.connections.read().await;
        connections.keys().cloned().collect()
    }
}

impl Default for ConnectionManager {
    fn default() -> Self {
        Self::new(32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::NodeId;

    #[tokio::test]
    async fn test_connection_manager_create() {
        let manager = ConnectionManager::new(16);
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_connection_manager_max_connections() {
        let manager = ConnectionManager::new(2);
        let peer1 = PeerInfo::new(NodeId::new(), vec!["addr1".to_string()], vec![1u8; 32]);
        let peer2 = PeerInfo::new(NodeId::new(), vec!["addr2".to_string()], vec![2u8; 32]);
        let peer3 = PeerInfo::new(NodeId::new(), vec!["addr3".to_string()], vec![3u8; 32]);

        assert!(manager.add_connection(&peer1, Box::new(StubConnection)).await);
        assert_eq!(manager.connection_count().await, 1);

        assert!(manager.add_connection(&peer2, Box::new(StubConnection)).await);
        assert_eq!(manager.connection_count().await, 2);

        assert!(!manager.add_connection(&peer3, Box::new(StubConnection)).await);
        assert_eq!(manager.connection_count().await, 2);
    }

    #[tokio::test]
    async fn test_connection_manager_remove() {
        let manager = ConnectionManager::new(16);
        let peer = PeerInfo::new(NodeId::new(), vec!["addr".to_string()], vec![1u8; 32]);
        let node_id = peer.node_id.to_string();

        assert!(manager.add_connection(&peer, Box::new(StubConnection)).await);
        assert_eq!(manager.connection_count().await, 1);

        manager.remove_connection(&node_id).await;
        assert_eq!(manager.connection_count().await, 0);
    }

    #[tokio::test]
    async fn test_connection_state_transitions() {
        let manager = ConnectionManager::new(16);
        let peer = PeerInfo::new(NodeId::new(), vec!["addr".to_string()], vec![1u8; 32]);
        let node_id = peer.node_id.to_string();

        assert!(manager.add_connection(&peer, Box::new(StubConnection)).await);
        assert!(manager.is_connected(&node_id).await);

        manager.update_state(&node_id, ConnectionState::Disconnecting).await;
        assert!(!manager.is_connected(&node_id).await);

        manager.update_state(&node_id, ConnectionState::Disconnected).await;
        assert!(!manager.is_connected(&node_id).await);
    }

    #[tokio::test]
    async fn test_stub_connection() {
        let conn = StubConnection;
        assert!(conn.send(Bytes::from("test")).await.is_ok());
        assert!(conn.receive().await.unwrap().is_none());
        assert!(conn.close().await.is_ok());
        assert!(!conn.is_alive().await);
    }

    #[test]
    fn test_default_connection_manager() {
        let manager = ConnectionManager::default();
        assert_eq!(manager.max_connections, 32);
    }
}
