use p2p::transport::Transport;

use super::build_application;

pub async fn run() -> anyhow::Result<()> {
    let (app, transport) = build_application().await?;

    app.start().await?;

    let config = config::PeerBoxConfig::load()?;
    let keypair = super::load_keypair()?;
    let node_id = keypair.node_id();

    let objects = app.list_objects().await?;
    let peers = transport.discover_peers().await?;
    let connected_peers: Vec<_> = peers.iter().filter(|p| p.connected).collect();

    let storage_usage: u64 = objects.iter().filter_map(|o| o.data["size"].as_u64()).sum();

    println!("=== Node Status ===");
    println!();
    println!("Node ID:      {}", node_id);
    println!("Node name:    {}", config.node_name);
    println!("Data dir:     {}", config.data_dir.display());
    println!();
    println!("Objects:      {}", objects.len());
    println!("Storage used: {} bytes", storage_usage);
    println!(
        "Peers:        {} ({} connected)",
        peers.len(),
        connected_peers.len()
    );
    println!();
    println!("Database:     {}", config.database_url);
    println!("Storage dir:  {}", config.storage_dir.display());
    println!("Log level:    {}", config.log_level);
    println!();

    app.shutdown().await?;

    Ok(())
}
