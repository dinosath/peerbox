use crate::transport::Transport;
use crate::types::PeerId;
use bytes::Bytes;
use common::{ManifestId, ObjectId};
use std::sync::Arc;
use storage::StorageProvider;

pub struct TransferManager {
    transport: Arc<dyn Transport>,
    storage: Arc<dyn StorageProvider>,
    peer_id: PeerId,
}

impl TransferManager {
    pub fn new(
        transport: Arc<dyn Transport>,
        storage: Arc<dyn StorageProvider>,
        peer_id: PeerId,
    ) -> Self {
        Self {
            transport,
            storage,
            peer_id,
        }
    }

    pub fn peer_id(&self) -> &PeerId {
        &self.peer_id
    }

    pub async fn download_manifest(&self, manifest_id: &ManifestId) -> anyhow::Result<Vec<PeerId>> {
        self.transport.find_manifest(manifest_id).await
    }

    pub async fn download_chunk(
        &self,
        peer: &PeerId,
        manifest_id: &ManifestId,
        chunk_index: u64,
    ) -> anyhow::Result<Option<Bytes>> {
        self.transport
            .request_chunk(peer, manifest_id, chunk_index)
            .await
    }

    pub async fn upload_chunk(
        &self,
        manifest_id: &ManifestId,
        chunk_index: u64,
        data: Bytes,
    ) -> anyhow::Result<()> {
        let object_id = ObjectId::from(format!(
            "{}_{}_{}",
            manifest_id.algorithm, manifest_id.hash, chunk_index
        ));
        self.storage.put(&object_id, data).await?;
        Ok(())
    }
}
