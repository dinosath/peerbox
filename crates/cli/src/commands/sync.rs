use super::build_application;

pub async fn run() -> anyhow::Result<()> {
    let (app, _transport) = build_application().await?;

    app.start().await?;

    let objects = app.list_objects().await?;
    let total_size: usize = objects.iter().map(|o| o.data.to_string().len()).sum();

    println!("=== Sync Status ===");
    println!();
    println!("Stored objects: {}", objects.len());
    println!("Total data size: {} bytes", total_size);
    println!("Pending transfers: 0 (simulated transport)");
    println!();

    app.shutdown().await?;

    Ok(())
}
