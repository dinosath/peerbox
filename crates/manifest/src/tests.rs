use super::*;
use crate::download::ResumableDownload;

#[test]
fn test_create_manifest_from_data() {
    let data = vec![0x61u8; 4096];
    let manifest = Manifest::from_data(
        &data,
        1024,
        vec![Transport::Local],
        Some("text/plain".into()),
        Some("test.txt".into()),
    );

    assert_eq!(manifest.size, 4096);
    assert_eq!(manifest.chunks.len(), 4);
    assert_eq!(manifest.filename, Some("test.txt".to_string()));
    assert_eq!(manifest.mime_type, Some("text/plain".to_string()));
    assert!(matches!(manifest.transports[0], Transport::Local));
}

#[test]
fn test_json_roundtrip() {
    let data = vec![0x62u8; 2048];
    let manifest = Manifest::from_data(
        &data,
        512,
        vec![
            Transport::Iroh,
            Transport::Https("https://example.com".into()),
        ],
        Some("application/octet-stream".into()),
        Some("binary.bin".into()),
    );

    let json = manifest.to_json().unwrap();
    let restored = Manifest::from_json(&json).unwrap();

    assert_eq!(manifest.size, restored.size);
    assert_eq!(manifest.chunks.len(), restored.chunks.len());
    assert_eq!(manifest.filename, restored.filename);
    assert_eq!(manifest.mime_type, restored.mime_type);
    assert_eq!(manifest.id, restored.id);

    for (a, b) in manifest.chunks.iter().zip(restored.chunks.iter()) {
        assert_eq!(a.offset, b.offset);
        assert_eq!(a.size, b.size);
        assert_eq!(a.hash, b.hash);
    }
}

#[test]
fn test_json_roundtrip_recomputes_id() {
    let data = vec![0x63u8; 1024];
    let manifest = Manifest::from_data(&data, 256, vec![], None, None);
    let mut json: serde_json::Value = serde_json::from_str(&manifest.to_json().unwrap()).unwrap();
    json["id"] = serde_json::json!({"algorithm": "blake3", "hash": "0000000000000000000000000000000000000000000000000000000000000000"});
    let tampered_json = serde_json::to_string(&json).unwrap();
    let restored = Manifest::from_json(&tampered_json).unwrap();
    assert_eq!(
        restored.id, manifest.id,
        "from_json should recompute the id"
    );
}

#[test]
fn test_verification_succeeds_with_correct_data() {
    let data = vec![0x64u8; 2048];
    let manifest = Manifest::from_data(&data, 512, vec![], None, None);
    assert!(manifest.verify(&data));
}

#[test]
fn test_verification_fails_corrupted_data() {
    let data = vec![0x65u8; 2048];
    let manifest = Manifest::from_data(&data, 512, vec![], None, None);

    let mut corrupted = data.clone();
    corrupted[1024] = 0xFF;
    assert!(!manifest.verify(&corrupted));
}

#[test]
fn test_verification_fails_truncated_data() {
    let data = vec![0x66u8; 2048];
    let manifest = Manifest::from_data(&data, 512, vec![], None, None);

    let truncated = vec![0x66u8; 1000];
    assert!(!manifest.verify(&truncated));
}

#[test]
fn test_verification_fails_extra_data() {
    let data = vec![0x67u8; 2048];
    let manifest = Manifest::from_data(&data, 512, vec![], None, None);

    let extra = vec![0x67u8; 3000];
    assert!(!manifest.verify(&extra));
}

#[test]
fn test_resumable_download_out_of_order_chunks() {
    let data = vec![0x68u8; 2000];
    let manifest = Manifest::from_data(&data, 500, vec![], None, None);
    let mut download = ResumableDownload::new(manifest);

    let num_chunks = download.manifest.chunks.len() as u64;
    let mut indices: Vec<u64> = (0..num_chunks).collect();
    indices.sort_by(|a, b| {
        let order = [1, 3, 0, 2];
        let a_pos = order
            .iter()
            .position(|&x| x == *a as usize)
            .unwrap_or(*a as usize);
        let b_pos = order
            .iter()
            .position(|&x| x == *b as usize)
            .unwrap_or(*b as usize);
        a_pos.cmp(&b_pos)
    });

    for idx in &indices {
        let start = *idx as usize * 500;
        let end = std::cmp::min(start + 500, data.len());
        download
            .receive_chunk(*idx, data[start..end].to_vec())
            .unwrap();
    }

    let assembled = download.assemble().unwrap();
    assert_eq!(assembled.len(), 2000);
    assert_eq!(assembled, data);
}

#[test]
fn test_large_file_manifest_1mb() {
    let size = 1024 * 1024 + 777;
    let data: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
    let chunk_size = 64 * 1024;

    let manifest = Manifest::from_data(
        &data,
        chunk_size,
        vec![Transport::Https(
            "https://peerbox.io/files/large.bin".into(),
        )],
        None,
        Some("large.bin".into()),
    );

    assert_eq!(manifest.size, size as u64);

    let expected_num_chunks = (size as f64 / chunk_size as f64).ceil() as usize;
    assert_eq!(manifest.chunks.len(), expected_num_chunks);

    assert!(manifest.verify(&data));

    let mut corrupted = data.clone();
    corrupted[500_000] ^= 0x01;
    assert!(!manifest.verify(&corrupted));

    let json = manifest.to_json().unwrap();
    let restored = Manifest::from_json(&json).unwrap();
    assert_eq!(manifest.id, restored.id);
    assert_eq!(manifest.chunks.len(), restored.chunks.len());
}

#[test]
fn test_manifest_with_ipfs_and_iroh_transports() {
    let data = vec![0x69u8; 1024];
    let manifest = Manifest::from_data(
        &data,
        256,
        vec![Transport::Ipfs, Transport::Iroh],
        Some("image/png".into()),
        Some("icon.png".into()),
    );

    assert!(matches!(manifest.transports[0], Transport::Ipfs));
    assert!(matches!(manifest.transports[1], Transport::Iroh));

    let json = manifest.to_json().unwrap();
    let restored = Manifest::from_json(&json).unwrap();
    assert_eq!(manifest.transports.len(), restored.transports.len());
}

#[test]
fn test_empty_file_manifest() {
    let data: Vec<u8> = vec![];
    let manifest = Manifest::from_data(&data, 1024, vec![], None, None);

    assert_eq!(manifest.size, 0);
    assert_eq!(manifest.chunks.len(), 0);
    assert!(manifest.verify(&data));
}

#[test]
fn test_single_chunk_file() {
    let data = vec![0x70u8; 500];
    let manifest = Manifest::from_data(&data, 1024, vec![], None, None);

    assert_eq!(manifest.chunks.len(), 1);
    assert_eq!(manifest.chunks[0].offset, 0);
    assert_eq!(manifest.chunks[0].size, 500);
    assert!(manifest.verify(&data));
}

#[test]
fn test_resumable_download_full_lifecycle() {
    let data = vec![0x71u8; 2048];
    let manifest = Manifest::from_data(&data, 512, vec![Transport::Local], None, None);
    let mut download = ResumableDownload::new(manifest.clone());

    assert_eq!(download.missing_chunks().len(), 4);
    assert_eq!(download.progress(), 0.0);

    for i in 0..4u64 {
        let start = i as usize * 512;
        let end = std::cmp::min(start + 512, data.len());
        let done = download
            .receive_chunk(i, data[start..end].to_vec())
            .unwrap();

        let expected_progress = (i + 1) as f64 / 4.0;
        assert!((download.progress() - expected_progress).abs() < 0.001);

        if i == 3 {
            assert!(done);
        } else {
            assert!(!done);
        }
    }

    assert_eq!(download.missing_chunks().len(), 0);

    let assembled = download.assemble().unwrap();
    assert_eq!(assembled.len(), 2048);
    assert!(manifest.verify(&assembled));
}

#[test]
fn test_chunking_consistency() {
    let data = vec![0x72u8; 2048];
    let chunk_size = 256;

    let manifest1 = Manifest::from_data(&data, chunk_size, vec![], None, None);
    let manifest2 = Manifest::from_data(&data, chunk_size, vec![], None, None);

    assert_eq!(manifest1.id, manifest2.id);
    assert_eq!(manifest1.chunks.len(), manifest2.chunks.len());
    for (a, b) in manifest1.chunks.iter().zip(manifest2.chunks.iter()) {
        assert_eq!(a.offset, b.offset);
        assert_eq!(a.size, b.size);
        assert_eq!(a.hash, b.hash);
    }
}

#[test]
fn test_manifest_id_is_unique() {
    let data1 = vec![0x73u8; 1024];
    let data2 = vec![0x74u8; 1024];

    let m1 = Manifest::from_data(&data1, 256, vec![], None, None);
    let m2 = Manifest::from_data(&data2, 256, vec![], None, None);
    assert_ne!(m1.id, m2.id);

    let m3 = Manifest::from_data(&data1, 512, vec![], None, None);
    assert_ne!(m1.id, m3.id);
}

#[test]
fn test_manifest_to_json_pretty() {
    let data = vec![0x75u8; 512];
    let manifest = Manifest::from_data(
        &data,
        128,
        vec![Transport::Local],
        Some("text/plain".into()),
        Some("readme.txt".into()),
    );
    let json = manifest.to_json().unwrap();
    assert!(json.contains("\"id\""));
    assert!(json.contains("\"size\""));
    assert!(json.contains("\"chunks\""));
    assert!(json.contains("\"Local\""));
}
