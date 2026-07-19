use crate::types::{PeerId, PeerInfo};
use bytes::Bytes;
use common::ManifestId;

#[async_trait::async_trait]
pub trait Transport: Send + Sync {
    async fn discover_peers(&self) -> anyhow::Result<Vec<PeerInfo>>;
    async fn send_chunk(&self, peer: &PeerId, chunk_index: u64, data: Bytes) -> anyhow::Result<()>;
    async fn receive_chunk(&self, peer: &PeerId, chunk_index: u64)
        -> anyhow::Result<Option<Bytes>>;
    async fn request_chunk(
        &self,
        peer: &PeerId,
        manifest_id: &ManifestId,
        chunk_index: u64,
    ) -> anyhow::Result<Option<Bytes>>;
    async fn announce_manifest(&self, manifest_id: &ManifestId) -> anyhow::Result<()>;
    async fn find_manifest(&self, manifest_id: &ManifestId) -> anyhow::Result<Vec<PeerId>>;
}
