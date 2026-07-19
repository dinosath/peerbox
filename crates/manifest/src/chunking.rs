use common::ChunkInfo;
use std::io::Read;

pub struct Chunker {
    chunk_size: u64,
}

impl Chunker {
    const DEFAULT_CHUNK_SIZE: u64 = 64 * 1024;

    pub fn new(chunk_size: u64) -> Self {
        Chunker { chunk_size }
    }

    pub fn chunk(&self, data: &[u8]) -> Vec<ChunkInfo> {
        let mut chunks = Vec::new();
        let mut offset = 0u64;
        for chunk_data in data.chunks(self.chunk_size as usize) {
            let size = chunk_data.len() as u64;
            chunks.push(ChunkInfo {
                index: chunks.len() as u64,
                offset,
                size,
                hash: common::ContentHash::new_blake3(chunk_data),
            });
            offset += size;
        }
        chunks
    }

    pub fn chunk_stream<R: Read>(&self, reader: &mut R) -> anyhow::Result<Vec<ChunkInfo>> {
        let mut chunks = Vec::new();
        let mut offset = 0u64;
        let mut buffer = vec![0u8; self.chunk_size as usize];
        loop {
            let mut total_read = 0;
            while total_read < self.chunk_size as usize {
                match reader.read(&mut buffer[total_read..]) {
                    Ok(0) => break,
                    Ok(n) => total_read += n,
                    Err(e) => return Err(e.into()),
                }
            }
            if total_read == 0 {
                break;
            }
            let chunk_data = &buffer[..total_read];
            chunks.push(ChunkInfo {
                index: chunks.len() as u64,
                offset,
                size: total_read as u64,
                hash: common::ContentHash::new_blake3(chunk_data),
            });
            offset += total_read as u64;
            if total_read < self.chunk_size as usize {
                break;
            }
        }
        Ok(chunks)
    }
}

impl Default for Chunker {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CHUNK_SIZE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_empty_data() {
        let chunker = Chunker::default();
        let chunks = chunker.chunk(&[]);
        assert!(chunks.is_empty());
    }

    #[test]
    fn test_chunk_smaller_than_chunk_size() {
        let chunker = Chunker::new(1024);
        let data = vec![0x41u8; 256];
        let chunks = chunker.chunk(&data);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].offset, 0);
        assert_eq!(chunks[0].size, 256);
        let expected_hash = common::ContentHash::new_blake3(&data);
        assert_eq!(chunks[0].hash, expected_hash);
    }

    #[test]
    fn test_chunk_exactly_chunk_size() {
        let chunker = Chunker::new(512);
        let data = vec![0x42u8; 512];
        let chunks = chunker.chunk(&data);
        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].offset, 0);
        assert_eq!(chunks[0].size, 512);
    }

    #[test]
    fn test_chunk_larger_than_chunk_size() {
        let chunker = Chunker::new(200);
        let data = vec![0x43u8; 550];
        let chunks = chunker.chunk(&data);
        assert_eq!(chunks.len(), 3);

        assert_eq!(chunks[0].offset, 0);
        assert_eq!(chunks[0].size, 200);

        assert_eq!(chunks[1].offset, 200);
        assert_eq!(chunks[1].size, 200);

        assert_eq!(chunks[2].offset, 400);
        assert_eq!(chunks[2].size, 150);
    }

    #[test]
    fn test_chunk_offsets_and_sizes() {
        let chunker = Chunker::new(100);
        let data = vec![0x44u8; 250];
        let chunks = chunker.chunk(&data);

        assert_eq!(chunks.len(), 3);
        assert_eq!(chunks[0].offset, 0);
        assert_eq!(chunks[0].size, 100);
        assert_eq!(chunks[1].offset, 100);
        assert_eq!(chunks[1].size, 100);
        assert_eq!(chunks[2].offset, 200);
        assert_eq!(chunks[2].size, 50);
    }

    #[test]
    fn test_chunk_hashes_differ() {
        let chunker = Chunker::new(100);
        let mut data = vec![0x45u8; 200];
        data[150] = 0x46;
        let chunks = chunker.chunk(&data);

        assert_eq!(chunks.len(), 2);
        assert_ne!(chunks[0].hash, chunks[1].hash);
    }

    #[test]
    fn test_chunk_hashes_match_direct_computation() {
        let chunker = Chunker::new(256);
        let data = vec![0x47u8; 400];
        let chunks = chunker.chunk(&data);

        for chunk in &chunks {
            let expected = common::ContentHash::new_blake3(
                &data[chunk.offset as usize..(chunk.offset + chunk.size) as usize],
            );
            assert_eq!(chunk.hash, expected);
        }
    }

    #[test]
    fn test_chunk_stream_small() {
        let chunker = Chunker::new(1024);
        let data = vec![0x48u8; 512];
        let mut cursor = std::io::Cursor::new(&data);
        let chunks = chunker.chunk_stream(&mut cursor).unwrap();

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].offset, 0);
        assert_eq!(chunks[0].size, 512);
    }

    #[test]
    fn test_chunk_stream_large() {
        let chunker = Chunker::new(1024);
        let data = vec![0x49u8; 5000];
        let mut cursor = std::io::Cursor::new(&data);
        let chunks = chunker.chunk_stream(&mut cursor).unwrap();

        assert_eq!(chunks.len(), 5);
        for (i, chunk) in chunks.iter().enumerate() {
            assert_eq!(chunk.offset, (i * 1024) as u64);
            let expected_size = if i == 4 { 5000 - 4096 } else { 1024 };
            assert_eq!(chunk.size, expected_size as u64);
        }
    }

    #[test]
    fn test_chunk_stream_and_chunk_produce_same_result() {
        let chunker = Chunker::new(256);
        let data = vec![0x50u8; 1000];
        let chunks_1 = chunker.chunk(&data);
        let mut cursor = std::io::Cursor::new(&data);
        let chunks_2 = chunker.chunk_stream(&mut cursor).unwrap();

        assert_eq!(chunks_1.len(), chunks_2.len());
        for (a, b) in chunks_1.iter().zip(chunks_2.iter()) {
            assert_eq!(a.offset, b.offset);
            assert_eq!(a.size, b.size);
            assert_eq!(a.hash, b.hash);
        }
    }
}
