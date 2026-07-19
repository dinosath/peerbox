use config::PeerBoxConfig;
use crypto::KeyPair;

use super::{data_dir, load_keypair, node_id_from_keypair, save_keypair};

pub async fn run() -> anyhow::Result<()> {
    let dir = data_dir();
    std::fs::create_dir_all(&dir)?;
    println!("Created data directory: {}", dir.display());

    let keypair = match std::fs::metadata(super::keypair_path()) {
        Ok(_) => {
            println!("Keypair already exists, loading existing...");
            load_keypair()?
        }
        Err(_) => {
            let kp = KeyPair::generate();
            save_keypair(&kp)?;
            println!("Generated new keypair");
            kp
        }
    };

    let config = PeerBoxConfig::load()?;
    config.save()?;
    println!(
        "Config saved to: {}",
        PeerBoxConfig::default_config_path().display()
    );

    let _repo = database::SqliteObjectRepository::new(&config.database_url).await?;
    println!("Database initialized at: {}", config.database_url);

    let node_id = node_id_from_keypair(&keypair);
    println!();
    println!("=== Node initialized ===");
    println!("Node ID: {}", node_id);
    println!(
        "Public key: {}",
        hex::encode(keypair.public_key().to_bytes())
    );
    println!("Data directory: {}", dir.display());
    println!("Storage directory: {}", config.storage_dir.display());
    println!();

    Ok(())
}
