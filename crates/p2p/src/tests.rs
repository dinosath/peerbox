use super::*;
use bytes::Bytes;
use common::{now, ContentHash, NodeId};
use simulated::{SimulatedNetwork, SimulatedTransport};
use std::sync::Arc;
use storage::{MemoryStorageProvider, StorageProvider};
use tokio::sync::RwLock;
use transfer::TransferManager;
use transport::Transport;
use types::{PeerId, PeerInfo};

fn make_peer_info(id: &str, node_id: &str) -> PeerInfo {
    PeerInfo {
        id: PeerId(id.to_string()),
        node_id: NodeId(node_id.to_string()),
        addresses: vec![format!("/addr/{id}")],
        connected: true,
        last_seen: now(),
    }
}

#[tokio::test]
async fn test_peer_discovery() {
    let net = Arc::new(RwLock::new(SimulatedNetwork::new()));

    let a = PeerId("A".into());
    let b = PeerId("B".into());
    let c = PeerId("C".into());

    net.write().await.add_peer(make_peer_info("A", "node-a"));
    net.write().await.add_peer(make_peer_info("B", "node-b"));
    net.write().await.add_peer(make_peer_info("C", "node-c"));

    net.write().await.connect_peers(&a, &b);
    net.write().await.connect_peers(&a, &c);
    net.write().await.connect_peers(&b, &c);

    let transport_a = SimulatedTransport::new(net.clone(), a.clone());
    let transport_b = SimulatedTransport::new(net.clone(), b.clone());
    let transport_c = SimulatedTransport::new(net.clone(), c.clone());

    let a_peers = transport_a.discover_peers().await.unwrap();
    let b_peers = transport_b.discover_peers().await.unwrap();
    let c_peers = transport_c.discover_peers().await.unwrap();

    assert_eq!(a_peers.len(), 2);
    assert_eq!(b_peers.len(), 2);
    assert_eq!(c_peers.len(), 2);

    let a_ids: Vec<&str> = a_peers.iter().map(|p| p.id.0.as_str()).collect();
    assert!(a_ids.contains(&"B"));
    assert!(a_ids.contains(&"C"));
}

#[tokio::test]
async fn test_chunk_transfer() {
    let net = Arc::new(RwLock::new(SimulatedNetwork::new()));

    let a = PeerId("A".into());
    let b = PeerId("B".into());

    net.write().await.add_peer(make_peer_info("A", "node-a"));
    net.write().await.add_peer(make_peer_info("B", "node-b"));
    net.write().await.connect_peers(&a, &b);

    let chunk_data = Bytes::from(vec![0xDE, 0xAD, 0xBE, 0xEF]);

    net.write().await.store_chunk(&a, 0, chunk_data.clone());

    let transport_b = SimulatedTransport::new(net.clone(), b.clone());

    let received = transport_b.receive_chunk(&a, 0).await.unwrap();
    assert_eq!(received, Some(chunk_data));
}

#[tokio::test]
async fn test_multi_hop_replication() {
    let net = Arc::new(RwLock::new(SimulatedNetwork::new()));

    let a = PeerId("A".into());
    let b = PeerId("B".into());
    let c = PeerId("C".into());

    net.write().await.add_peer(make_peer_info("A", "node-a"));
    net.write().await.add_peer(make_peer_info("B", "node-b"));
    net.write().await.add_peer(make_peer_info("C", "node-c"));

    net.write().await.connect_peers(&a, &b);
    net.write().await.connect_peers(&b, &c);
    net.write().await.connect_peers(&a, &c);

    let chunk_0 = Bytes::from(vec![0x01; 256]);
    let chunk_1 = Bytes::from(vec![0x02; 256]);
    let chunk_2 = Bytes::from(vec![0x03; 256]);

    let hash_0 = ContentHash::new_blake3(&chunk_0);
    let hash_1 = ContentHash::new_blake3(&chunk_1);
    let hash_2 = ContentHash::new_blake3(&chunk_2);

    net.write().await.store_chunk(&a, 0, chunk_0.clone());
    net.write().await.store_chunk(&a, 1, chunk_1.clone());
    net.write().await.store_chunk(&a, 2, chunk_2.clone());

    let _transport_a = SimulatedTransport::new(net.clone(), a.clone());
    let transport_b = SimulatedTransport::new(net.clone(), b.clone());
    let transport_c = SimulatedTransport::new(net.clone(), c.clone());

    let b_chunk_0 = transport_b.receive_chunk(&a, 0).await.unwrap().unwrap();
    let b_chunk_1 = transport_b.receive_chunk(&a, 1).await.unwrap().unwrap();
    let b_chunk_2 = transport_b.receive_chunk(&a, 2).await.unwrap().unwrap();

    assert_eq!(ContentHash::new_blake3(&b_chunk_0), hash_0);
    assert_eq!(ContentHash::new_blake3(&b_chunk_1), hash_1);
    assert_eq!(ContentHash::new_blake3(&b_chunk_2), hash_2);

    net.write().await.store_chunk(&b, 0, b_chunk_0);
    net.write().await.store_chunk(&b, 1, b_chunk_1);
    net.write().await.store_chunk(&b, 2, b_chunk_2);

    let c_chunk_0 = transport_c.receive_chunk(&b, 0).await.unwrap().unwrap();
    let c_chunk_1 = transport_c.receive_chunk(&b, 1).await.unwrap().unwrap();
    let c_chunk_2 = transport_c.receive_chunk(&b, 2).await.unwrap().unwrap();

    assert_eq!(ContentHash::new_blake3(&c_chunk_0), hash_0);
    assert_eq!(ContentHash::new_blake3(&c_chunk_1), hash_1);
    assert_eq!(ContentHash::new_blake3(&c_chunk_2), hash_2);
}

#[tokio::test]
async fn test_network_failure() {
    let net = Arc::new(RwLock::new(SimulatedNetwork::new()));

    let a = PeerId("A".into());
    let b = PeerId("B".into());

    net.write().await.add_peer(make_peer_info("A", "node-a"));
    net.write().await.add_peer(make_peer_info("B", "node-b"));
    net.write().await.connect_peers(&a, &b);

    let chunk_data = Bytes::from(vec![0xCA, 0xFE]);
    net.write().await.store_chunk(&a, 0, chunk_data.clone());

    let transport_b = SimulatedTransport::new(net.clone(), b.clone());
    let received = transport_b.receive_chunk(&a, 0).await.unwrap();
    assert_eq!(received, Some(chunk_data));

    net.write().await.disconnect_peer(&a);

    let result = transport_b.receive_chunk(&a, 0).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_manifest_announcement() {
    let net = Arc::new(RwLock::new(SimulatedNetwork::new()));

    let a = PeerId("A".into());
    let b = PeerId("B".into());

    net.write().await.add_peer(make_peer_info("A", "node-a"));
    net.write().await.add_peer(make_peer_info("B", "node-b"));
    net.write().await.connect_peers(&a, &b);

    let manifest_id = ContentHash::new_blake3(b"test-manifest");

    let transport_a = SimulatedTransport::new(net.clone(), a.clone());
    let transport_b = SimulatedTransport::new(net.clone(), b.clone());

    transport_a.announce_manifest(&manifest_id).await.unwrap();

    let peers = transport_b.find_manifest(&manifest_id).await.unwrap();
    assert_eq!(peers.len(), 1);
    assert_eq!(peers[0], a);

    transport_b.announce_manifest(&manifest_id).await.unwrap();

    let peers = transport_a.find_manifest(&manifest_id).await.unwrap();
    assert_eq!(peers.len(), 2);
}

#[tokio::test]
async fn test_concurrent_transfers() {
    let net = Arc::new(RwLock::new(SimulatedNetwork::new()));

    let a = PeerId("A".into());
    let b = PeerId("B".into());

    net.write().await.add_peer(make_peer_info("A", "node-a"));
    net.write().await.add_peer(make_peer_info("B", "node-b"));
    net.write().await.connect_peers(&a, &b);

    const NUM_CHUNKS: u64 = 10;
    for i in 0..NUM_CHUNKS {
        net.write()
            .await
            .store_chunk(&a, i, Bytes::from(vec![i as u8; 64]));
    }

    let transport_b = Arc::new(SimulatedTransport::new(net.clone(), b.clone()));

    let mut handles = Vec::new();
    for i in 0..NUM_CHUNKS {
        let t = transport_b.clone();
        let a = a.clone();
        handles.push(tokio::spawn(async move {
            let data = t.receive_chunk(&a, i).await.unwrap().unwrap();
            assert_eq!(data.len(), 64);
            assert_eq!(data[0], i as u8);
            data
        }));
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[tokio::test]
async fn test_transfer_manager_download_chunk() {
    let net = Arc::new(RwLock::new(SimulatedNetwork::new()));

    let a = PeerId("A".into());
    let b = PeerId("B".into());

    net.write().await.add_peer(make_peer_info("A", "node-a"));
    net.write().await.add_peer(make_peer_info("B", "node-b"));
    net.write().await.connect_peers(&a, &b);

    let manifest_id = ContentHash::new_blake3(b"my-file");
    let chunk_data = Bytes::from(vec![0x42; 128]);

    net.write().await.store_chunk(&a, 7, chunk_data.clone());

    let _transport_a: Arc<dyn Transport> =
        Arc::new(SimulatedTransport::new(net.clone(), a.clone()));
    let transport_b: Arc<dyn Transport> = Arc::new(SimulatedTransport::new(net.clone(), b.clone()));

    let storage = Arc::new(MemoryStorageProvider::new());
    let manager = TransferManager::new(transport_b, storage, b.clone());

    let downloaded = manager.download_chunk(&a, &manifest_id, 7).await.unwrap();
    assert_eq!(downloaded, Some(chunk_data));
}

#[tokio::test]
async fn test_transfer_manager_upload_chunk() {
    let net = Arc::new(RwLock::new(SimulatedNetwork::new()));

    let a = PeerId("A".into());
    net.write().await.add_peer(make_peer_info("A", "node-a"));

    let transport_a: Arc<dyn Transport> = Arc::new(SimulatedTransport::new(net.clone(), a.clone()));
    let storage = Arc::new(MemoryStorageProvider::new());
    let manager = TransferManager::new(transport_a, storage.clone(), a.clone());

    let manifest_id = ContentHash::new_blake3(b"upload-test");
    let chunk_data = Bytes::from(vec![0x77; 512]);

    manager
        .upload_chunk(&manifest_id, 42, chunk_data.clone())
        .await
        .unwrap();

    let object_id =
        common::ObjectId::from(format!("{}_{}_42", manifest_id.algorithm, manifest_id.hash));
    let stored = storage.get(&object_id).await.unwrap();
    assert_eq!(stored, Some(chunk_data));
}

#[tokio::test]
async fn test_node_identity_generate() {
    let identity = identity::NodeIdentity::generate();
    assert!(!identity.peer_id.0.is_empty());
    assert_eq!(identity.peer_id.0.len(), 64);

    let data = b"test data for signing";
    let sig = identity.sign(data);
    assert_eq!(sig.len(), 64);
    assert!(identity.verify(data, &sig));
    assert!(!identity.verify(b"wrong data", &sig));
}
