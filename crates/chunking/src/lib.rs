use common::{ChunkInfo, ContentHash};
use std::collections::BTreeMap;
use tokio::io::{AsyncRead, AsyncReadExt};

pub const DEFAULT_CHUNK_SIZE: u64 = 1024 * 1024;

#[derive(Debug, Clone)]
pub struct ChunkResult {
    pub chunks: Vec<(u64, Vec<u8>)>,
    pub chunk_infos: Vec<ChunkInfo>,
}

#[derive(Debug, Clone)]
pub struct Chunker {
    chunk_size: u64,
}

impl Chunker {
    pub fn new(chunk_size: u64) -> Self {
        assert!(chunk_size > 0, "chunk_size must be greater than 0");
        Self { chunk_size }
    }

    pub fn chunk_data(&self, data: &[u8]) -> ChunkResult {
        let chunk_size = self.chunk_size as usize;
        let mut chunks = Vec::new();
        let mut chunk_infos = Vec::new();
        let mut offset: u64 = 0;

        for (index, chunk) in (0_u64..).zip(data.chunks(chunk_size)) {
            let chunk_data = chunk.to_vec();
            let size = chunk_data.len() as u64;
            let hash = ContentHash::new_blake3(&chunk_data);
            let info = ChunkInfo {
                index,
                offset,
                size,
                hash,
            };
            chunks.push((index, chunk_data));
            chunk_infos.push(info);
            offset += size;
        }

        ChunkResult {
            chunks,
            chunk_infos,
        }
    }

    pub fn chunk_stream<R: AsyncRead + Unpin>(
        &self,
        reader: R,
    ) -> impl futures::Stream<Item = anyhow::Result<(u64, Vec<u8>, ChunkInfo)>> {
        let chunk_size = self.chunk_size as usize;
        futures::stream::unfold(
            (reader, 0u64, 0u64),
            move |(mut reader, index, offset)| async move {
                let mut buf = vec![0u8; chunk_size];
                match reader.read(&mut buf).await {
                    Ok(0) => None,
                    Ok(n) => {
                        buf.truncate(n);
                        let hash = ContentHash::new_blake3(&buf);
                        let info = ChunkInfo {
                            index,
                            offset,
                            size: n as u64,
                            hash,
                        };
                        Some((
                            Ok((index, buf, info)),
                            (reader, index + 1, offset + n as u64),
                        ))
                    }
                    Err(e) => Some((Err(anyhow::anyhow!(e)), (reader, index, offset))),
                }
            },
        )
    }
}

pub struct AsyncChunker {
    chunk_size: u64,
}

impl AsyncChunker {
    pub fn new(chunk_size: u64) -> Self {
        assert!(chunk_size > 0, "chunk_size must be greater than 0");
        Self { chunk_size }
    }

    pub async fn chunk_all<R: AsyncRead + Unpin>(
        &self,
        mut reader: R,
    ) -> anyhow::Result<ChunkResult> {
        let chunk_size = self.chunk_size as usize;
        let mut chunks = Vec::new();
        let mut chunk_infos = Vec::new();
        let mut index: u64 = 0;
        let mut offset: u64 = 0;

        loop {
            let mut buf = vec![0u8; chunk_size];
            let n = reader.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            buf.truncate(n);
            let hash = ContentHash::new_blake3(&buf);
            let info = ChunkInfo {
                index,
                offset,
                size: n as u64,
                hash,
            };
            chunks.push((index, buf));
            chunk_infos.push(info);
            offset += n as u64;
            index += 1;
        }

        Ok(ChunkResult {
            chunks,
            chunk_infos,
        })
    }
}

#[derive(Debug, Default)]
pub struct ChunkAssembler {
    chunks: BTreeMap<u64, Vec<u8>>,
}

impl ChunkAssembler {
    pub fn new() -> Self {
        Self {
            chunks: BTreeMap::new(),
        }
    }

    pub fn add_chunk(&mut self, index: u64, data: Vec<u8>) -> Result<(), ChunkingError> {
        if self.chunks.contains_key(&index) {
            return Err(ChunkingError::DuplicateChunk(index));
        }
        self.chunks.insert(index, data);
        Ok(())
    }

    pub fn assemble(&self) -> Result<Vec<u8>, ChunkingError> {
        if self.chunks.is_empty() {
            return Err(ChunkingError::NoChunks);
        }
        let max_index = *self.chunks.keys().max().unwrap();
        let mut missing = Vec::new();
        let mut result = Vec::new();
        for i in 0..=max_index {
            match self.chunks.get(&i) {
                Some(data) => result.extend_from_slice(data),
                None => missing.push(i),
            }
        }
        if !missing.is_empty() {
            return Err(ChunkingError::MissingChunks(missing));
        }
        Ok(result)
    }

    pub fn is_complete(&self, chunk_infos: &[ChunkInfo]) -> bool {
        chunk_infos
            .iter()
            .all(|ci| self.chunks.contains_key(&ci.index))
    }

    pub fn missing_chunks(&self, chunk_infos: &[ChunkInfo]) -> Vec<u64> {
        chunk_infos
            .iter()
            .map(|ci| ci.index)
            .filter(|idx| !self.chunks.contains_key(idx))
            .collect()
    }
}

pub struct ChunkVerifier;

impl ChunkVerifier {
    pub fn verify_chunk(data: &[u8], info: &ChunkInfo) -> bool {
        let computed = ContentHash::new_blake3(data);
        computed == info.hash
    }

    pub fn verify_all(chunks: &[(u64, Vec<u8>)], infos: &[ChunkInfo]) -> bool {
        let info_map: BTreeMap<u64, &ChunkInfo> = infos.iter().map(|ci| (ci.index, ci)).collect();
        for (index, data) in chunks {
            match info_map.get(index) {
                Some(info) if !Self::verify_chunk(data, info) => return false,
                None => return false,
                _ => {}
            }
        }
        infos.len() == chunks.len()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ChunkingError {
    #[error("chunk {0} already added")]
    DuplicateChunk(u64),
    #[error("missing chunks: {0:?}")]
    MissingChunks(Vec<u64>),
    #[error("no chunks to assemble")]
    NoChunks,
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;

    #[test]
    fn test_chunk_and_reassemble_small_file() {
        let data = b"Hello, Peerbox! This is a small test file for chunking.";
        let chunker = Chunker::new(16);
        let result = chunker.chunk_data(data);

        assert!(!result.chunks.is_empty());
        assert!(!result.chunk_infos.is_empty());
        assert_eq!(result.chunks.len(), result.chunk_infos.len());

        let mut assembler = ChunkAssembler::new();
        for (index, chunk_data) in &result.chunks {
            assembler.add_chunk(*index, chunk_data.clone()).unwrap();
        }
        let reassembled = assembler.assemble().unwrap();
        assert_eq!(reassembled, data);
    }

    #[test]
    fn test_chunk_large_file() {
        let size = 5 * 1024 * 1024;
        let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let chunker = Chunker::new(DEFAULT_CHUNK_SIZE);
        let result = chunker.chunk_data(&data);

        assert_eq!(result.chunks.len(), 5);
        assert_eq!(result.chunk_infos.len(), 5);

        for info in &result.chunk_infos {
            assert_eq!(info.size, DEFAULT_CHUNK_SIZE);
        }

        let mut assembler = ChunkAssembler::new();
        for (index, chunk_data) in &result.chunks {
            assembler.add_chunk(*index, chunk_data.clone()).unwrap();
        }
        let reassembled = assembler.assemble().unwrap();
        assert_eq!(reassembled, data);
    }

    #[test]
    fn test_empty_data() {
        let chunker = Chunker::new(DEFAULT_CHUNK_SIZE);
        let result = chunker.chunk_data(&[]);
        assert!(result.chunks.is_empty());
        assert!(result.chunk_infos.is_empty());
    }

    #[test]
    fn test_data_exactly_at_chunk_boundary() {
        let chunk_size = 1024;
        let data = vec![0xABu8; chunk_size];
        let chunker = Chunker::new(chunk_size as u64);
        let result = chunker.chunk_data(&data);

        assert_eq!(result.chunks.len(), 1);
        assert_eq!(result.chunk_infos[0].size, chunk_size as u64);

        let mut assembler = ChunkAssembler::new();
        assembler.add_chunk(0, result.chunks[0].1.clone()).unwrap();
        let reassembled = assembler.assemble().unwrap();
        assert_eq!(reassembled, data);
    }

    #[test]
    fn test_data_exactly_double_chunk_boundary() {
        let chunk_size = 512;
        let data = vec![0xCDu8; chunk_size * 2];
        let chunker = Chunker::new(chunk_size as u64);
        let result = chunker.chunk_data(&data);

        assert_eq!(result.chunks.len(), 2);
        assert_eq!(result.chunk_infos[0].size, chunk_size as u64);
        assert_eq!(result.chunk_infos[1].size, chunk_size as u64);

        let mut assembler = ChunkAssembler::new();
        assembler.add_chunk(0, result.chunks[0].1.clone()).unwrap();
        assembler.add_chunk(1, result.chunks[1].1.clone()).unwrap();
        let reassembled = assembler.assemble().unwrap();
        assert_eq!(reassembled, data);
    }

    #[test]
    fn test_missing_chunk_detection() {
        let chunker = Chunker::new(100);
        let data = vec![0u8; 250];
        let result = chunker.chunk_data(&data);

        let mut assembler = ChunkAssembler::new();
        assembler.add_chunk(0, result.chunks[0].1.clone()).unwrap();
        // Skip chunk 1, add chunk 2
        assembler.add_chunk(2, result.chunks[2].1.clone()).unwrap();

        assert!(!assembler.is_complete(&result.chunk_infos));
        let missing = assembler.missing_chunks(&result.chunk_infos);
        assert_eq!(missing, vec![1]);

        let assemble_result = assembler.assemble();
        assert!(assemble_result.is_err());
    }

    #[test]
    fn test_out_of_order_assembly() {
        let chunker = Chunker::new(16);
        let data = b"out of order assembly test data for peerbox";
        let result = chunker.chunk_data(data);

        let mut assembler = ChunkAssembler::new();
        // Add chunks in reverse order
        for i in (0..result.chunks.len()).rev() {
            assembler
                .add_chunk(i as u64, result.chunks[i].1.clone())
                .unwrap();
        }

        let reassembled = assembler.assemble().unwrap();
        assert_eq!(reassembled, data);
    }

    #[test]
    fn test_chunk_verification() {
        let chunker = Chunker::new(64);
        let data = b"verification test data for the chunking crate in peerbox";
        let result = chunker.chunk_data(data);

        for i in 0..result.chunks.len() {
            assert!(ChunkVerifier::verify_chunk(
                &result.chunks[i].1,
                &result.chunk_infos[i]
            ));
        }

        assert!(ChunkVerifier::verify_all(
            &result.chunks,
            &result.chunk_infos
        ));
    }

    #[test]
    fn test_corrupted_chunk_detection() {
        let chunker = Chunker::new(64);
        let data = b"corruption detection test data for peerbox chunking";
        let result = chunker.chunk_data(data);

        // Verify original passes
        assert!(ChunkVerifier::verify_chunk(
            &result.chunks[0].1,
            &result.chunk_infos[0]
        ));

        // Corrupt a chunk
        let mut corrupted = result.chunks[0].1.clone();
        if !corrupted.is_empty() {
            corrupted[0] = corrupted[0].wrapping_add(1);
        }

        assert!(!ChunkVerifier::verify_chunk(
            &corrupted,
            &result.chunk_infos[0]
        ));
    }

    #[test]
    fn test_verify_all_detects_corruption() {
        let chunker = Chunker::new(64);
        let data = b"verify all corruption detection test for peerbox";
        let result = chunker.chunk_data(data);

        let mut bad_chunks = result.chunks.clone();
        if !bad_chunks[0].1.is_empty() {
            bad_chunks[0].1[0] = bad_chunks[0].1[0].wrapping_add(1);
        }

        assert!(!ChunkVerifier::verify_all(&bad_chunks, &result.chunk_infos));
    }

    #[test]
    fn test_duplicate_chunk_error() {
        let mut assembler = ChunkAssembler::new();
        assembler.add_chunk(0, vec![1, 2, 3]).unwrap();
        let result = assembler.add_chunk(0, vec![4, 5, 6]);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_chunks_error() {
        let assembler = ChunkAssembler::new();
        let result = assembler.assemble();
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_chunks_returns_all_missing() {
        let chunker = Chunker::new(50);
        let data = vec![0u8; 200];
        let result = chunker.chunk_data(&data);

        let assembler = ChunkAssembler::new();
        // Don't add any chunks
        let missing = assembler.missing_chunks(&result.chunk_infos);
        assert_eq!(missing.len(), result.chunk_infos.len());
        assert_eq!(
            missing,
            (0..result.chunk_infos.len() as u64).collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_chunk_info_fields() {
        let chunk_size = 256;
        let chunker = Chunker::new(chunk_size);
        // Data is 2.5 chunks
        let data = vec![0x42u8; (chunk_size as f64 * 2.5) as usize];
        let result = chunker.chunk_data(&data);

        assert_eq!(result.chunk_infos.len(), 3);
        assert_eq!(result.chunk_infos[0].index, 0);
        assert_eq!(result.chunk_infos[0].offset, 0);
        assert_eq!(result.chunk_infos[0].size, chunk_size);

        assert_eq!(result.chunk_infos[1].index, 1);
        assert_eq!(result.chunk_infos[1].offset, chunk_size);
        assert_eq!(result.chunk_infos[1].size, chunk_size);

        assert_eq!(result.chunk_infos[2].index, 2);
        assert_eq!(result.chunk_infos[2].offset, chunk_size * 2);
        assert_eq!(result.chunk_infos[2].size, chunk_size / 2);

        assert_eq!(result.chunk_infos[0].hash, result.chunk_infos[1].hash);
    }

    #[tokio::test]
    async fn test_async_chunker() {
        let data = b"async chunker test data for peerbox chunking crate";
        let reader = std::io::Cursor::new(data.to_vec());
        let chunker = AsyncChunker::new(16);
        let result = chunker.chunk_all(reader).await.unwrap();

        assert!(!result.chunks.is_empty());
        let mut assembler = ChunkAssembler::new();
        for (index, chunk_data) in &result.chunks {
            assembler.add_chunk(*index, chunk_data.clone()).unwrap();
        }
        let reassembled = assembler.assemble().unwrap();
        assert_eq!(reassembled, data);
    }

    #[tokio::test]
    async fn test_chunk_stream() {
        let data = b"stream chunker test data for the peerbox chunking crate";
        let reader = std::io::Cursor::new(data.to_vec());
        let chunker = Chunker::new(16);

        let mut stream = Box::pin(chunker.chunk_stream(reader));
        let mut chunks = Vec::new();
        let mut infos = Vec::new();

        while let Some(item) = stream.next().await {
            let (index, chunk_data, info) = item.unwrap();
            chunks.push((index, chunk_data));
            infos.push(info);
        }

        assert!(!chunks.is_empty());
        assert_eq!(chunks.len(), infos.len());

        let mut assembler = ChunkAssembler::new();
        for (index, chunk_data) in &chunks {
            assembler.add_chunk(*index, chunk_data.clone()).unwrap();
        }
        let reassembled = assembler.assemble().unwrap();
        assert_eq!(reassembled, data);
    }

    #[tokio::test]
    async fn test_chunk_stream_empty() {
        let reader = std::io::Cursor::new(Vec::new());
        let chunker = Chunker::new(1024);

        let mut stream = Box::pin(chunker.chunk_stream(reader));
        let item = stream.next().await;
        assert!(item.is_none());
    }
}
