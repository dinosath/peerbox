use std::path::PathBuf;

use bytes::Bytes;

use super::build_application;

pub async fn run(manifest_id: String, output: Option<PathBuf>) -> anyhow::Result<()> {
    let (app, _transport) = build_application().await?;

    let manifest_obj_id = common::ObjectId::from(format!("manifest-{}", manifest_id));
    let manifest_data = app
        .object_service()
        .get_file_data(&manifest_obj_id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("Manifest not found: {}", manifest_id))?;

    let manifest: manifest::Manifest = serde_json::from_slice(&manifest_data)?;

    println!(
        "Downloading: {}  ({} bytes, {} chunks)",
        manifest.filename.as_deref().unwrap_or("unknown"),
        manifest.size,
        manifest.chunks.len()
    );

    let mut assembled = Vec::with_capacity(manifest.size as usize);

    for chunk in &manifest.chunks {
        let chunk_id = common::ObjectId::from(format!("chunk-{}-{}", manifest_id, chunk.index));
        let chunk_data = app
            .object_service()
            .get_file_data(&chunk_id)
            .await?
            .unwrap_or_else(|| Bytes::copy_from_slice(&vec![0u8; chunk.size as usize]));

        let actual_hash = common::ContentHash::new_blake3(&chunk_data);
        if actual_hash != chunk.hash {
            anyhow::bail!(
                "Chunk {} verification failed: expected {}, got {}",
                chunk.index,
                chunk.hash,
                actual_hash
            );
        }

        assembled.extend_from_slice(&chunk_data);
    }

    if assembled.len() as u64 != manifest.size {
        anyhow::bail!(
            "Size mismatch: expected {}, got {}",
            manifest.size,
            assembled.len()
        );
    }

    let output_path =
        output.unwrap_or_else(|| PathBuf::from(manifest.filename.as_deref().unwrap_or("download")));

    std::fs::write(&output_path, &assembled)?;

    println!();
    println!("=== Download complete ===");
    println!("File:        {}", output_path.display());
    println!("Size:        {} bytes", assembled.len());
    println!("Manifest ID: {}", manifest_id);
    println!();

    Ok(())
}
