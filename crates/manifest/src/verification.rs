use common::{ChunkInfo, ContentHash};

pub struct Verifier;

impl Verifier {
    pub fn verify_chunk(chunk_data: &[u8], expected_hash: &ContentHash) -> bool {
        let computed = ContentHash::new_blake3(chunk_data);
        &computed == expected_hash
    }

    pub fn verify_manifest(data: &[u8], chunks: &[ChunkInfo]) -> bool {
        if chunks.is_empty() && data.is_empty() {
            return true;
        }

        let total_size: u64 = chunks.iter().map(|c| c.size).sum();
        if total_size != data.len() as u64 {
            return false;
        }

        for chunk_info in chunks {
            let start = chunk_info.offset as usize;
            let end = (chunk_info.offset + chunk_info.size) as usize;
            if end > data.len() {
                return false;
            }
            let chunk_data = &data[start..end];
            if !Self::verify_chunk(chunk_data, &chunk_info.hash) {
                return false;
            }
        }
        true
    }

    pub fn compute_manifest_hash(chunks: &[ChunkInfo], total_size: u64) -> ContentHash {
        let mut hasher = blake3::Hasher::new();
        for chunk in chunks {
            hasher.update(chunk.hash.to_string().as_bytes());
        }
        hasher.update(&total_size.to_le_bytes());
        ContentHash::from_blake3(hasher.finalize().as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::ChunkInfo;

    #[test]
    fn test_verify_valid_chunk() {
        let data = vec![0x41u8; 256];
        let hash = ContentHash::new_blake3(&data);
        assert!(Verifier::verify_chunk(&data, &hash));
    }

    #[test]
    fn test_verify_corrupted_chunk() {
        let data = vec![0x41u8; 256];
        let hash = ContentHash::new_blake3(&data);
        let mut corrupted = data.clone();
        corrupted[100] = 0x99;
        assert!(!Verifier::verify_chunk(&corrupted, &hash));
    }

    #[test]
    fn test_verify_empty_chunk() {
        let empty = [];
        let hash = ContentHash::new_blake3(&empty);
        assert!(Verifier::verify_chunk(&empty, &hash));
    }

    #[test]
    fn test_verify_full_manifest() {
        use crate::chunking::Chunker;

        let data = vec![0x42u8; 1024];
        let chunker = Chunker::new(256);
        let chunks = chunker.chunk(&data);

        assert!(Verifier::verify_manifest(&data, &chunks));
    }

    #[test]
    fn test_verify_manifest_wrong_data_size() {
        use crate::chunking::Chunker;

        let data = vec![0x42u8; 1024];
        let chunker = Chunker::new(256);
        let chunks = chunker.chunk(&data);

        let smaller = vec![0x42u8; 512];
        assert!(!Verifier::verify_manifest(&smaller, &chunks));
    }

    #[test]
    fn test_verify_manifest_corrupted_data() {
        use crate::chunking::Chunker;

        let data = vec![0x42u8; 1024];
        let chunker = Chunker::new(256);
        let chunks = chunker.chunk(&data);

        let mut corrupted = data.clone();
        corrupted[500] = 0xFF;
        assert!(!Verifier::verify_manifest(&corrupted, &chunks));
    }

    #[test]
    fn test_verify_empty_manifest() {
        let data: Vec<u8> = vec![];
        let chunks: Vec<ChunkInfo> = vec![];
        assert!(Verifier::verify_manifest(&data, &chunks));
    }

    #[test]
    fn test_compute_manifest_hash() {
        use crate::chunking::Chunker;

        let data = vec![0x43u8; 512];
        let chunker = Chunker::new(128);
        let chunks = chunker.chunk(&data);
        let total_size = data.len() as u64;

        let id = Verifier::compute_manifest_hash(&chunks, total_size);
        assert!(!id.hash.is_empty());
        assert_eq!(id.hash.len(), 64);
    }

    #[test]
    fn test_manifest_hash_changes_with_different_chunks() {
        use crate::chunking::Chunker;

        let data1 = vec![0x44u8; 512];
        let data2 = vec![0x45u8; 512];
        let chunker = Chunker::new(128);
        let chunks1 = chunker.chunk(&data1);
        let chunks2 = chunker.chunk(&data2);

        let id1 = Verifier::compute_manifest_hash(&chunks1, 512);
        let id2 = Verifier::compute_manifest_hash(&chunks2, 512);
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_manifest_hash_changes_with_different_chunk_sizes() {
        let data = vec![0x46u8; 512];
        let chunker1 = super::super::chunking::Chunker::new(128);
        let chunker2 = super::super::chunking::Chunker::new(256);
        let chunks1 = chunker1.chunk(&data);
        let chunks2 = chunker2.chunk(&data);

        let id1 = Verifier::compute_manifest_hash(&chunks1, 512);
        let id2 = Verifier::compute_manifest_hash(&chunks2, 512);
        assert_ne!(id1, id2);
    }
}
