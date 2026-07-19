use crate::transport::Transport;
use crate::types::{PeerId, PeerInfo};
use bytes::Bytes;
use common::ManifestId;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct SimulatedNetwork {
    peers: HashMap<PeerId, PeerInfo>,
    connections: HashMap<PeerId, HashSet<PeerId>>,
    chunks: HashMap<(PeerId, u64), Bytes>,
    manifests: HashMap<ManifestId, HashSet<PeerId>>,
}

impl SimulatedNetwork {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_peer(&mut self, info: PeerInfo) {
        self.peers.insert(info.id.clone(), info);
    }

    pub fn connect_peers(&mut self, a: &PeerId, b: &PeerId) {
        self.connections
            .entry(a.clone())
            .or_default()
            .insert(b.clone());
        self.connections
            .entry(b.clone())
            .or_default()
            .insert(a.clone());
    }

    pub fn disconnect_peer(&mut self, id: &PeerId) {
        self.connections.remove(id);
        for peers in self.connections.values_mut() {
            peers.remove(id);
        }
    }

    pub fn store_chunk(&mut self, peer: &PeerId, chunk_index: u64, data: Bytes) {
        self.chunks.insert((peer.clone(), chunk_index), data);
    }

    pub fn get_chunk(&self, peer: &PeerId, chunk_index: u64) -> Option<Bytes> {
        self.chunks.get(&(peer.clone(), chunk_index)).cloned()
    }

    pub fn announce_manifest(&mut self, peer: &PeerId, manifest_id: &ManifestId) {
        self.manifests
            .entry(manifest_id.clone())
            .or_default()
            .insert(peer.clone());
    }

    pub fn get_manifest_peers(&self, manifest_id: &ManifestId) -> Vec<PeerId> {
        self.manifests
            .get(manifest_id)
            .map(|s| s.iter().cloned().collect())
            .unwrap_or_default()
    }

    pub fn is_connected(&self, a: &PeerId, b: &PeerId) -> bool {
        self.connections
            .get(a)
            .map(|peers| peers.contains(b))
            .unwrap_or(false)
    }

    pub fn get_connected_peers(&self, peer: &PeerId) -> Vec<PeerInfo> {
        self.connections
            .get(peer)
            .map(|peers| {
                peers
                    .iter()
                    .filter_map(|p| self.peers.get(p).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn has_peer(&self, id: &PeerId) -> bool {
        self.peers.contains_key(id)
    }
}

pub struct SimulatedTransport {
    network: Arc<RwLock<SimulatedNetwork>>,
    peer_id: PeerId,
}

impl SimulatedTransport {
    pub fn new(network: Arc<RwLock<SimulatedNetwork>>, peer_id: PeerId) -> Self {
        Self { network, peer_id }
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    pub fn network(&self) -> &Arc<RwLock<SimulatedNetwork>> {
        &self.network
    }
}

#[async_trait::async_trait]
impl Transport for SimulatedTransport {
    async fn discover_peers(&self) -> anyhow::Result<Vec<PeerInfo>> {
        let net = self.network.read().await;
        Ok(net.get_connected_peers(&self.peer_id))
    }

    async fn send_chunk(&self, peer: &PeerId, chunk_index: u64, data: Bytes) -> anyhow::Result<()> {
        let net = self.network.read().await;
        if !net.is_connected(&self.peer_id, peer) {
            anyhow::bail!("not connected to peer {}", peer);
        }
        drop(net);
        self.network
            .write()
            .await
            .store_chunk(peer, chunk_index, data);
        Ok(())
    }

    async fn receive_chunk(
        &self,
        peer: &PeerId,
        chunk_index: u64,
    ) -> anyhow::Result<Option<Bytes>> {
        let net = self.network.read().await;
        if !net.is_connected(&self.peer_id, peer) {
            anyhow::bail!("not connected to peer {}", peer);
        }
        Ok(net.get_chunk(peer, chunk_index))
    }

    async fn request_chunk(
        &self,
        peer: &PeerId,
        _manifest_id: &ManifestId,
        chunk_index: u64,
    ) -> anyhow::Result<Option<Bytes>> {
        self.receive_chunk(peer, chunk_index).await
    }

    async fn announce_manifest(&self, manifest_id: &ManifestId) -> anyhow::Result<()> {
        self.network
            .write()
            .await
            .announce_manifest(&self.peer_id, manifest_id);
        Ok(())
    }

    async fn find_manifest(&self, manifest_id: &ManifestId) -> anyhow::Result<Vec<PeerId>> {
        let net = self.network.read().await;
        Ok(net.get_manifest_peers(manifest_id))
    }
}
