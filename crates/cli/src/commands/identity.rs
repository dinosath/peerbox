use crypto::KeyPair;

use super::{load_keypair, node_id_from_keypair, save_keypair, IdentityCommand};

pub async fn run(action: IdentityCommand) -> anyhow::Result<()> {
    match action {
        IdentityCommand::Create => create().await,
        IdentityCommand::Show => show().await,
        IdentityCommand::Export => export().await,
    }
}

async fn create() -> anyhow::Result<()> {
    let keypair = KeyPair::generate();
    save_keypair(&keypair)?;
    let node_id = node_id_from_keypair(&keypair);

    println!("New identity created:");
    println!("  Node ID: {}", node_id);
    println!(
        "  Public key: {}",
        hex::encode(keypair.public_key().to_bytes())
    );
    println!("  Saved to: {}", super::keypair_path().display());

    Ok(())
}

async fn show() -> anyhow::Result<()> {
    let keypair = load_keypair()?;
    let node_id = node_id_from_keypair(&keypair);

    println!("Node ID:    {}", node_id);
    println!(
        "Public key: {}",
        hex::encode(keypair.public_key().to_bytes())
    );

    Ok(())
}

async fn export() -> anyhow::Result<()> {
    let keypair = load_keypair()?;
    println!("{}", hex::encode(keypair.public_key().to_bytes()));
    Ok(())
}
