pub mod chunking;
pub mod verification;

mod download;
pub use download::ResumableDownload;

use common::{ChunkInfo, ContentHash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transport {
    Iroh,
    Https(String),
    Ipfs,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub id: ContentHash,
    pub size: u64,
    pub chunks: Vec<ChunkInfo>,
    pub transports: Vec<Transport>,
    pub mime_type: Option<String>,
    pub filename: Option<String>,
}

impl Manifest {
    pub fn new(
        size: u64,
        chunks: Vec<ChunkInfo>,
        transports: Vec<Transport>,
        mime_type: Option<String>,
        filename: Option<String>,
    ) -> Self {
        let id = verification::Verifier::compute_manifest_hash(&chunks, size);
        Manifest {
            id,
            size,
            chunks,
            transports,
            mime_type,
            filename,
        }
    }

    pub fn from_data(
        data: &[u8],
        chunk_size: u64,
        transports: Vec<Transport>,
        mime_type: Option<String>,
        filename: Option<String>,
    ) -> Self {
        let chunker = chunking::Chunker::new(chunk_size);
        let chunks = chunker.chunk(data);
        let size = data.len() as u64;
        Self::new(size, chunks, transports, mime_type, filename)
    }

    pub fn verify(&self, data: &[u8]) -> bool {
        verification::Verifier::verify_manifest(data, &self.chunks)
    }

    pub fn to_json(&self) -> anyhow::Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    pub fn from_json(json: &str) -> anyhow::Result<Self> {
        let mut manifest: Manifest = serde_json::from_str(json)?;
        manifest.id =
            verification::Verifier::compute_manifest_hash(&manifest.chunks, manifest.size);
        Ok(manifest)
    }
}

#[cfg(test)]
mod tests;
