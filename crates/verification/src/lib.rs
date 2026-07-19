use std::collections::{HashMap, HashSet};

use common::{ChunkInfo, ContentHash};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerificationResult {
    Valid,
    CorruptedChunk {
        index: u64,
        expected: String,
        got: String,
    },
    MissingChunks(Vec<u64>),
    SizeMismatch {
        expected: u64,
        got: u64,
    },
}

#[derive(Debug, Error)]
pub enum VerificationError {
    #[error("hash mismatch: algorithm={algorithm}, expected={expected}, got={got}")]
    HashMismatch {
        algorithm: String,
        expected: String,
        got: String,
    },
    #[error("chunk verification failed: {0:?}")]
    ChunkVerification(Vec<VerificationResult>),
    #[error("size mismatch: expected={expected}, got={got}")]
    SizeMismatch { expected: u64, got: u64 },
    #[error("missing chunks: {0:?}")]
    MissingChunks(Vec<u64>),
}

pub struct Verifier;

impl Verifier {
    pub fn verify_data(data: &[u8], expected_hash: &ContentHash) -> bool {
        let actual = ContentHash::new_blake3(data);
        actual.hash == expected_hash.hash
    }

    pub fn verify_chunks(
        chunks: &[(u64, Vec<u8>)],
        chunk_infos: &[ChunkInfo],
    ) -> Vec<VerificationResult> {
        let info_map: HashMap<u64, &ChunkInfo> = chunk_infos.iter().map(|c| (c.index, c)).collect();
        let mut results = Vec::new();
        for (index, data) in chunks {
            if let Some(info) = info_map.get(index) {
                let actual_hash = ContentHash::new_blake3(data);
                if actual_hash.hash == info.hash.hash {
                    results.push(VerificationResult::Valid);
                } else {
                    results.push(VerificationResult::CorruptedChunk {
                        index: *index,
                        expected: info.hash.hash.clone(),
                        got: actual_hash.hash,
                    });
                }
            }
        }
        results
    }

    pub fn verify_file(data: &[u8], chunk_infos: &[ChunkInfo]) -> Result<(), VerificationError> {
        let expected_size: u64 = chunk_infos.iter().map(|c| c.size).sum();
        if data.len() as u64 != expected_size {
            return Err(VerificationError::SizeMismatch {
                expected: expected_size,
                got: data.len() as u64,
            });
        }

        let mut results = Vec::new();
        for info in chunk_infos {
            let start = info.offset as usize;
            let end = start + info.size as usize;
            if end > data.len() {
                results.push(VerificationResult::MissingChunks(vec![info.index]));
                continue;
            }
            let chunk_data = &data[start..end];
            let actual_hash = ContentHash::new_blake3(chunk_data);
            if actual_hash.hash == info.hash.hash {
                results.push(VerificationResult::Valid);
            } else {
                results.push(VerificationResult::CorruptedChunk {
                    index: info.index,
                    expected: info.hash.hash.clone(),
                    got: actual_hash.hash,
                });
            }
        }

        let all_valid = results
            .iter()
            .all(|r| matches!(r, VerificationResult::Valid));
        if all_valid {
            Ok(())
        } else {
            Err(VerificationError::ChunkVerification(results))
        }
    }

    pub fn verify_size(data_len: u64, expected_size: u64) -> bool {
        data_len == expected_size
    }
}

pub struct ProgressiveVerifier {
    chunk_infos: Vec<ChunkInfo>,
    verified: HashSet<u64>,
}

impl ProgressiveVerifier {
    pub fn new(chunk_infos: Vec<ChunkInfo>) -> Self {
        Self {
            chunk_infos,
            verified: HashSet::new(),
        }
    }

    pub fn feed_chunk(
        &mut self,
        index: u64,
        data: Vec<u8>,
    ) -> Result<VerificationResult, VerificationError> {
        let info = self.chunk_infos.iter().find(|c| c.index == index);
        match info {
            Some(info) => {
                let actual = ContentHash::new_blake3(&data);
                self.verified.insert(index);
                if actual.hash == info.hash.hash {
                    Ok(VerificationResult::Valid)
                } else {
                    Ok(VerificationResult::CorruptedChunk {
                        index,
                        expected: info.hash.hash.clone(),
                        got: actual.hash,
                    })
                }
            }
            None => Err(VerificationError::MissingChunks(vec![index])),
        }
    }

    pub fn is_complete(&self) -> bool {
        self.verified.len() == self.chunk_infos.len()
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.verified.len(), self.chunk_infos.len())
    }

    pub fn remaining(&self) -> Vec<u64> {
        self.chunk_infos
            .iter()
            .filter_map(|c| {
                if self.verified.contains(&c.index) {
                    None
                } else {
                    Some(c.index)
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_chunk_info(index: u64, offset: u64, data: &[u8]) -> ChunkInfo {
        ChunkInfo {
            index,
            offset,
            size: data.len() as u64,
            hash: ContentHash::new_blake3(data),
        }
    }

    #[test]
    fn test_verify_correct_data() {
        let data = b"hello world";
        let hash = ContentHash::new_blake3(data);
        assert!(Verifier::verify_data(data, &hash));
    }

    #[test]
    fn test_verify_corrupted_data() {
        let data = b"hello world";
        let hash = ContentHash::new_blake3(data);
        let corrupted = b"hello worlD";
        assert!(!Verifier::verify_data(corrupted, &hash));
    }

    #[test]
    fn test_verify_size() {
        assert!(Verifier::verify_size(100, 100));
        assert!(!Verifier::verify_size(100, 200));
    }

    #[test]
    fn test_verify_correct_chunks() {
        let chunk1 = b"hello";
        let chunk2 = b"world";
        let ci1 = make_chunk_info(0, 0, chunk1);
        let ci2 = make_chunk_info(1, 5, chunk2);
        let chunks = vec![(0u64, chunk1.to_vec()), (1u64, chunk2.to_vec())];
        let results = Verifier::verify_chunks(&chunks, &[ci1, ci2]);
        assert!(results
            .iter()
            .all(|r| matches!(r, VerificationResult::Valid)));
    }

    #[test]
    fn test_verify_corrupted_chunk() {
        let chunk1 = b"hello";
        let chunk2 = b"world";
        let ci1 = make_chunk_info(0, 0, chunk1);
        let ci2 = make_chunk_info(1, 5, chunk2);
        let corrupted = b"worlD";
        let chunks = vec![(0u64, chunk1.to_vec()), (1u64, corrupted.to_vec())];
        let results = Verifier::verify_chunks(&chunks, &[ci1, ci2]);
        assert_eq!(results.len(), 2);
        assert!(matches!(results[0], VerificationResult::Valid));
        assert!(matches!(
            results[1],
            VerificationResult::CorruptedChunk { index: 1, .. }
        ));
    }

    #[test]
    fn test_verify_file_success() {
        let chunk1 = b"hello";
        let chunk2 = b"world";
        let data: Vec<u8> = [chunk1.as_slice(), chunk2.as_slice()].concat();
        let ci1 = make_chunk_info(0, 0, chunk1);
        let ci2 = make_chunk_info(1, 5, chunk2);
        let result = Verifier::verify_file(&data, &[ci1, ci2]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_file_corrupted_chunk() {
        let chunk1 = b"hello";
        let bad_data: Vec<u8> = [chunk1.as_slice(), b"worlD"].concat();
        let ci1 = make_chunk_info(0, 0, b"hello");
        let ci2 = make_chunk_info(1, 5, b"world");
        let result = Verifier::verify_file(&bad_data, &[ci1, ci2]);
        assert!(matches!(
            result,
            Err(VerificationError::ChunkVerification(_))
        ));
    }

    #[test]
    fn test_verify_file_size_mismatch() {
        let chunk1 = b"hello";
        let chunk2 = b"world";
        let data = b"hello";
        let ci1 = make_chunk_info(0, 0, chunk1);
        let ci2 = make_chunk_info(1, 5, chunk2);
        let result = Verifier::verify_file(data, &[ci1, ci2]);
        assert!(matches!(
            result,
            Err(VerificationError::SizeMismatch { .. })
        ));
    }

    #[test]
    fn test_missing_chunk_detection() {
        let data = b"hello";
        let ci1 = ChunkInfo {
            index: 0,
            offset: 0,
            size: 5,
            hash: ContentHash::new_blake3(b"hello"),
        };
        let ci2 = ChunkInfo {
            index: 1,
            offset: 5,
            size: 5,
            hash: ContentHash::new_blake3(b"world"),
        };
        let result = Verifier::verify_file(data, &[ci1, ci2]);
        assert!(matches!(
            result,
            Err(VerificationError::SizeMismatch { .. })
        ));
    }

    #[test]
    fn test_multiple_corrupted_chunks() {
        let chunk1 = b"hello";
        let chunk2 = b"world";
        let chunk3 = b"test";
        let ci1 = make_chunk_info(0, 0, chunk1);
        let ci2 = make_chunk_info(1, 5, chunk2);
        let ci3 = make_chunk_info(2, 10, chunk3);
        let corrupted1 = b"hellO";
        let corrupted3 = b"tesT";
        let chunks = vec![
            (0u64, corrupted1.to_vec()),
            (1u64, chunk2.to_vec()),
            (2u64, corrupted3.to_vec()),
        ];
        let results = Verifier::verify_chunks(&chunks, &[ci1, ci2, ci3]);
        assert_eq!(results.len(), 3);
        assert!(matches!(
            results[0],
            VerificationResult::CorruptedChunk { index: 0, .. }
        ));
        assert!(matches!(results[1], VerificationResult::Valid));
        assert!(matches!(
            results[2],
            VerificationResult::CorruptedChunk { index: 2, .. }
        ));
    }

    #[test]
    fn test_progressive_verification() {
        let chunk1 = b"hello";
        let chunk2 = b"world";
        let ci1 = make_chunk_info(0, 0, chunk1);
        let ci2 = make_chunk_info(1, 5, chunk2);
        let mut verifier = ProgressiveVerifier::new(vec![ci1, ci2]);

        assert!(!verifier.is_complete());
        assert_eq!(verifier.progress(), (0, 2));
        assert_eq!(verifier.remaining(), vec![0, 1]);

        let result = verifier.feed_chunk(0, chunk1.to_vec()).unwrap();
        assert_eq!(result, VerificationResult::Valid);
        assert_eq!(verifier.progress(), (1, 2));
        assert!(!verifier.is_complete());
        assert_eq!(verifier.remaining(), vec![1]);

        let result = verifier.feed_chunk(1, chunk2.to_vec()).unwrap();
        assert_eq!(result, VerificationResult::Valid);
        assert_eq!(verifier.progress(), (2, 2));
        assert!(verifier.is_complete());
        assert!(verifier.remaining().is_empty());
    }

    #[test]
    fn test_progressive_corrupted_chunk() {
        let chunk1 = b"hello";
        let ci1 = make_chunk_info(0, 0, chunk1);
        let mut verifier = ProgressiveVerifier::new(vec![ci1]);

        let result = verifier.feed_chunk(0, b"corrupted".to_vec()).unwrap();
        assert!(matches!(
            result,
            VerificationResult::CorruptedChunk { index: 0, .. }
        ));
        assert!(verifier.is_complete());
    }

    #[test]
    fn test_progressive_missing_chunk() {
        let ci1 = make_chunk_info(0, 0, b"hello");
        let mut verifier = ProgressiveVerifier::new(vec![ci1]);

        let result = verifier.feed_chunk(99, b"unknown".to_vec());
        assert!(matches!(result, Err(VerificationError::MissingChunks(_))));
        assert!(!verifier.is_complete());
    }

    #[test]
    fn test_empty_data_verification() {
        let data: Vec<u8> = vec![];
        let hash = ContentHash::new_blake3(&data);
        assert!(Verifier::verify_data(&data, &hash));

        let result = Verifier::verify_file(&[], &[]);
        assert!(result.is_ok());

        let verifier = ProgressiveVerifier::new(vec![]);
        assert!(verifier.is_complete());
        assert_eq!(verifier.progress(), (0, 0));
        assert!(verifier.remaining().is_empty());
    }
}
