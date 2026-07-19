use common::{NodeId, Timestamp};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: NodeId,
    pub addresses: Vec<String>,
    pub public_key: Vec<u8>,
    pub last_seen: Timestamp,
    pub connection_state: ConnectionState,
}

impl PeerInfo {
    pub fn new(node_id: NodeId, addresses: Vec<String>, public_key: Vec<u8>) -> Self {
        Self {
            node_id,
            addresses,
            public_key,
            last_seen: common::now(),
            connection_state: ConnectionState::Disconnected,
        }
    }

    pub fn is_reachable(&self) -> bool {
        !self.addresses.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::NodeId;

    #[test]
    fn test_peer_info_new() {
        let node_id = NodeId::new();
        let addresses = vec!["/ip4/192.168.1.1/tcp/8080".to_string()];
        let public_key = vec![1u8; 32];

        let peer = PeerInfo::new(node_id.clone(), addresses.clone(), public_key.clone());
        assert_eq!(peer.node_id, node_id);
        assert_eq!(peer.addresses, addresses);
        assert_eq!(peer.public_key, public_key);
        assert_eq!(peer.connection_state, ConnectionState::Disconnected);
        assert!(peer.is_reachable());
    }

    #[test]
    fn test_peer_info_not_reachable_without_addresses() {
        let node_id = NodeId::new();
        let peer = PeerInfo::new(node_id, vec![], vec![1u8; 32]);
        assert!(!peer.is_reachable());
    }

    #[test]
    fn test_connection_state_serialization() {
        let states = vec![
            ConnectionState::Disconnected,
            ConnectionState::Connecting,
            ConnectionState::Connected,
            ConnectionState::Disconnecting,
        ];
        for state in states {
            let json = serde_json::to_string(&state).unwrap();
            let deserialized: ConnectionState = serde_json::from_str(&json).unwrap();
            assert_eq!(state, deserialized);
        }
    }

    #[test]
    fn test_peer_info_serialization_roundtrip() {
        let peer = PeerInfo::new(
            NodeId::new(),
            vec!["/ip4/10.0.0.1/tcp/9090".to_string()],
            vec![2u8; 32],
        );
        let json = serde_json::to_string(&peer).unwrap();
        let deserialized: PeerInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(peer.node_id, deserialized.node_id);
        assert_eq!(peer.addresses, deserialized.addresses);
        assert_eq!(peer.public_key, deserialized.public_key);
        assert_eq!(peer.connection_state, deserialized.connection_state);
    }

    #[test]
    fn test_connection_state_machine() {
        let mut peer = PeerInfo::new(NodeId::new(), vec!["/ip4/127.0.0.1/tcp/8080".to_string()], vec![1u8; 32]);

        assert_eq!(peer.connection_state, ConnectionState::Disconnected);
        peer.connection_state = ConnectionState::Connecting;
        assert_eq!(peer.connection_state, ConnectionState::Connecting);
        peer.connection_state = ConnectionState::Connected;
        assert_eq!(peer.connection_state, ConnectionState::Connected);
        peer.connection_state = ConnectionState::Disconnecting;
        assert_eq!(peer.connection_state, ConnectionState::Disconnecting);
        peer.connection_state = ConnectionState::Disconnected;
        assert_eq!(peer.connection_state, ConnectionState::Disconnected);
    }
}
