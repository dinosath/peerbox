use crate::verification::Verifier;
use crate::Manifest;
use std::collections::{HashMap, HashSet};

pub struct ResumableDownload {
    pub manifest: Manifest,
    pub available_chunks: HashSet<u64>,
    received_data: HashMap<u64, Vec<u8>>,
}

impl ResumableDownload {
    pub fn new(manifest: Manifest) -> Self {
        ResumableDownload {
            manifest,
            available_chunks: HashSet::new(),
            received_data: HashMap::new(),
        }
    }

    pub fn receive_chunk(&mut self, index: u64, data: Vec<u8>) -> anyhow::Result<bool> {
        let chunk_info = self
            .manifest
            .chunks
            .iter()
            .find(|c| c.index == index)
            .ok_or_else(|| anyhow::anyhow!("chunk index {} out of range", index))?;

        if data.len() as u64 != chunk_info.size {
            anyhow::bail!(
                "chunk {} size mismatch: expected {}, got {}",
                index,
                chunk_info.size,
                data.len()
            );
        }

        if !Verifier::verify_chunk(&data, &chunk_info.hash) {
            anyhow::bail!("chunk {} hash verification failed", index);
        }

        self.received_data.insert(index, data);
        Ok(self.received_data.len() == self.manifest.chunks.len())
    }

    pub fn assemble(&self) -> anyhow::Result<Vec<u8>> {
        if self.received_data.len() != self.manifest.chunks.len() {
            anyhow::bail!("not all chunks received");
        }

        let total_size = self.manifest.size as usize;
        let mut result = vec![0u8; total_size];

        for (&index, chunk_data) in &self.received_data {
            let chunk_info = self
                .manifest
                .chunks
                .iter()
                .find(|c| c.index == index)
                .ok_or_else(|| anyhow::anyhow!("chunk index {} not found in manifest", index))?;
            let start = chunk_info.offset as usize;
            let end = start + chunk_info.size as usize;
            result[start..end].copy_from_slice(chunk_data);
        }

        Ok(result)
    }

    pub fn progress(&self) -> f64 {
        if self.manifest.chunks.is_empty() {
            return 1.0;
        }
        self.received_data.len() as f64 / self.manifest.chunks.len() as f64
    }

    pub fn missing_chunks(&self) -> Vec<u64> {
        self.manifest
            .chunks
            .iter()
            .filter(|c| !self.received_data.contains_key(&c.index))
            .map(|c| c.index)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Manifest;

    fn create_test_manifest() -> Manifest {
        let data = vec![0x51u8; 1000];
        Manifest::from_data(&data, 200, vec![], None, None)
    }

    #[test]
    fn test_download_new() {
        let manifest = create_test_manifest();
        let download = ResumableDownload::new(manifest.clone());
        assert_eq!(download.missing_chunks().len(), manifest.chunks.len());
        assert_eq!(download.progress(), 0.0);
    }

    #[test]
    fn test_receive_chunks_in_order() {
        let manifest = create_test_manifest();
        let mut download = ResumableDownload::new(manifest.clone());

        let data = vec![0x51u8; 1000];
        let chunk_size = 200;

        for i in 0..manifest.chunks.len() as u64 {
            let start = (i as usize) * chunk_size;
            let end = std::cmp::min(start + chunk_size, data.len());
            let chunk_data = data[start..end].to_vec();
            let done = download.receive_chunk(i, chunk_data).unwrap();
            if i == manifest.chunks.len() as u64 - 1 {
                assert!(done);
            } else {
                assert!(!done);
            }
        }

        assert_eq!(download.missing_chunks().len(), 0);
        assert_eq!(download.progress(), 1.0);
    }

    #[test]
    fn test_receive_chunks_out_of_order() {
        let manifest = create_test_manifest();
        let mut download = ResumableDownload::new(manifest.clone());

        let data = vec![0x51u8; 1000];
        let chunk_size = 200;
        let num_chunks = manifest.chunks.len() as u64;
        let mut order: Vec<u64> = (0..num_chunks).collect();
        order.reverse();

        for i in order {
            let start = (i as usize) * chunk_size;
            let end = std::cmp::min(start + chunk_size, data.len());
            let chunk_data = data[start..end].to_vec();
            let done = download.receive_chunk(i, chunk_data).unwrap();

            if download.received_data.len() == manifest.chunks.len() {
                assert!(done);
            }
        }

        assert_eq!(download.missing_chunks().len(), 0);
        assert_eq!(download.progress(), 1.0);
    }

    #[test]
    fn test_assemble() {
        let manifest = create_test_manifest();
        let mut download = ResumableDownload::new(manifest.clone());

        let data = vec![0x51u8; 1000];
        let chunk_size = 200;

        for i in 0..manifest.chunks.len() as u64 {
            let start = (i as usize) * chunk_size;
            let end = std::cmp::min(start + chunk_size, data.len());
            let chunk_data = data[start..end].to_vec();
            download.receive_chunk(i, chunk_data).unwrap();
        }

        let assembled = download.assemble().unwrap();
        assert_eq!(assembled.len(), 1000);
        assert_eq!(assembled, data);
        assert!(manifest.verify(&assembled));
    }

    #[test]
    fn test_assemble_out_of_order() {
        let manifest = create_test_manifest();
        let mut download = ResumableDownload::new(manifest.clone());

        let data = vec![0x51u8; 1000];
        let chunk_size = 200;
        let num_chunks = manifest.chunks.len() as u64;

        download
            .receive_chunk(
                num_chunks - 1,
                data[(num_chunks as usize - 1) * chunk_size..].to_vec(),
            )
            .unwrap();
        download
            .receive_chunk(0, data[..chunk_size].to_vec())
            .unwrap();
        for i in 1..num_chunks - 1 {
            let start = (i as usize) * chunk_size;
            let end = std::cmp::min(start + chunk_size, data.len());
            download
                .receive_chunk(i, data[start..end].to_vec())
                .unwrap();
        }

        let assembled = download.assemble().unwrap();
        assert_eq!(assembled.len(), 1000);
        assert_eq!(assembled, data);
    }

    #[test]
    fn test_assemble_fails_with_missing_chunks() {
        let manifest = create_test_manifest();
        let mut download = ResumableDownload::new(manifest.clone());

        let data = vec![0x51u8; 1000];
        download.receive_chunk(0, data[..200].to_vec()).unwrap();

        assert!(download.assemble().is_err());
    }

    #[test]
    fn test_progress_tracking() {
        let manifest = create_test_manifest();
        let mut download = ResumableDownload::new(manifest.clone());
        let num_chunks = manifest.chunks.len() as u64;

        assert_eq!(download.progress(), 0.0);

        let data = vec![0x51u8; 1000];
        let chunk_size = 200;

        for i in 0..num_chunks {
            let start = (i as usize) * chunk_size;
            let end = std::cmp::min(start + chunk_size, data.len());
            download
                .receive_chunk(i, data[start..end].to_vec())
                .unwrap();

            let expected = (i + 1) as f64 / num_chunks as f64;
            assert!((download.progress() - expected).abs() < 0.001);
        }

        assert_eq!(download.progress(), 1.0);
    }

    #[test]
    fn test_missing_chunks() {
        let manifest = create_test_manifest();
        let mut download = ResumableDownload::new(manifest.clone());
        let num_chunks = manifest.chunks.len() as u64;

        let all_missing = download.missing_chunks();
        assert_eq!(all_missing.len(), num_chunks as usize);
        for i in 0..num_chunks {
            assert!(all_missing.contains(&i));
        }

        let data = vec![0x51u8; 1000];
        download.receive_chunk(0, data[..200].to_vec()).unwrap();
        download
            .receive_chunk(
                num_chunks - 1,
                data[(num_chunks as usize - 1) * 200..].to_vec(),
            )
            .unwrap();

        let missing = download.missing_chunks();
        assert_eq!(missing.len(), (num_chunks - 2) as usize);
        assert!(!missing.contains(&0));
        assert!(!missing.contains(&(num_chunks - 1)));
    }

    #[test]
    fn test_receive_chunk_rejects_wrong_size() {
        let manifest = create_test_manifest();
        let mut download = ResumableDownload::new(manifest);

        let wrong_data = vec![0x52u8; 10];
        let err = download.receive_chunk(0, wrong_data).unwrap_err();
        assert!(err.to_string().contains("size mismatch"));
    }

    #[test]
    fn test_receive_chunk_rejects_corrupted_data() {
        let manifest = create_test_manifest();
        let mut download = ResumableDownload::new(manifest);

        let corrupted = vec![0xFFu8; 200];
        let err = download.receive_chunk(0, corrupted).unwrap_err();
        assert!(err.to_string().contains("hash verification failed"));
    }

    #[test]
    fn test_receive_chunk_rejects_out_of_range_index() {
        let manifest = create_test_manifest();
        let mut download = ResumableDownload::new(manifest);

        let err = download.receive_chunk(999, vec![0x51u8; 100]).unwrap_err();
        assert!(err.to_string().contains("out of range"));
    }

    #[test]
    fn test_progress_empty_manifest() {
        let manifest = Manifest::new(0, vec![], vec![], None, None);
        let download = ResumableDownload::new(manifest);
        assert_eq!(download.progress(), 1.0);
        assert_eq!(download.missing_chunks().len(), 0);
    }
}
