use std::path::PathBuf;

use bytes::Bytes;
use manifest::chunking::Chunker;
use manifest::{Manifest, Transport};
use objects::FileObject;

use super::build_application;

pub async fn run(file: PathBuf) -> anyhow::Result<()> {
    let (app, _transport) = build_application().await?;

    let file_name = file
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let mime_type = file
        .extension()
        .and_then(|ext| ext.to_str())
        .and_then(guess_mime);

    let data = std::fs::read(&file)?;
    let data_size = data.len() as u64;
    let data_bytes = Bytes::from(data.clone());

    println!("Uploading: {} ({} bytes)", file.display(), data_size);

    let chunker = Chunker::default();
    let chunks = chunker.chunk(&data);
    println!("Chunked into {} chunks", chunks.len());

    let manifest = Manifest::new(
        data_size,
        chunks.clone(),
        vec![Transport::Local],
        mime_type.clone(),
        Some(file_name.clone()),
    );
    let manifest_id = manifest.id.to_string();

    let file_object = FileObject::new(file_name.clone(), data_size, mime_type);

    let object_id = app
        .object_service()
        .create_file_with_data(file_object, data_bytes.clone())
        .await?;

    let manifest_json = manifest.to_json()?;
    let manifest_obj = objects::FileObject::new(
        format!("manifest-{}", manifest_id),
        manifest_json.len() as u64,
        Some("application/json".to_string()),
    );

    app.object_service()
        .create_file_with_data(manifest_obj, Bytes::from(manifest_json))
        .await?;

    println!();
    println!("=== Upload complete ===");
    println!("File:        {}", file_name);
    println!("Size:        {} bytes", data_size);
    println!("Chunks:      {}", chunks.len());
    println!("Object ID:   {}", object_id);
    println!("Manifest ID: {}", manifest_id);
    println!();

    Ok(())
}

fn guess_mime(ext: &str) -> Option<String> {
    match ext {
        "txt" => Some("text/plain".into()),
        "md" => Some("text/markdown".into()),
        "html" => Some("text/html".into()),
        "json" => Some("application/json".into()),
        "png" => Some("image/png".into()),
        "jpg" | "jpeg" => Some("image/jpeg".into()),
        "gif" => Some("image/gif".into()),
        "svg" => Some("image/svg+xml".into()),
        "pdf" => Some("application/pdf".into()),
        "zip" => Some("application/zip".into()),
        "mp4" => Some("video/mp4".into()),
        "mp3" => Some("audio/mpeg".into()),
        _ => None,
    }
}
