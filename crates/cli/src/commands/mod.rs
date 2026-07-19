mod download;
mod identity;
mod init;
mod peers;
mod status;
mod sync;
mod upload;

use std::path::PathBuf;
use std::sync::Arc;

use clap::{Parser, Subcommand};
use common::NodeId;
use config::PeerBoxConfig;
use crypto::{Ed25519CryptoProvider, KeyPair};
use database::{EventRepository, ObjectRepository, SqliteEventRepository, SqliteObjectRepository};
use peerbox_core::Application;
use events::EventBus;
use p2p::simulated::{SimulatedNetwork, SimulatedTransport};
use p2p::types::PeerId;
use storage::FileSystemStorageProvider;

#[derive(Parser)]
#[command(name = "peerbox", about = "Peerbox CLI", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    #[command(about = "Initialize a new node")]
    Init,
    #[command(about = "Manage node identity")]
    Identity {
        #[command(subcommand)]
        action: IdentityCommand,
    },
    #[command(about = "Upload a file")]
    Upload {
        #[arg(help = "Path to the file to upload")]
        file: PathBuf,
    },
    #[command(about = "Download a file by manifest ID")]
    Download {
        #[arg(help = "Manifest ID of the file to download")]
        manifest_id: String,
        #[arg(help = "Output path (defaults to original filename)")]
        output: Option<PathBuf>,
    },
    #[command(about = "Manage peers")]
    Peers {
        #[command(subcommand)]
        action: Option<PeersCommand>,
    },
    #[command(about = "Show sync status")]
    Sync,
    #[command(about = "Show node status")]
    Status,
}

#[derive(Subcommand)]
pub enum IdentityCommand {
    #[command(about = "Generate a new identity keypair")]
    Create,
    #[command(about = "Show current node ID and public key")]
    Show,
    #[command(about = "Export public key")]
    Export,
}

#[derive(Subcommand)]
pub enum PeersCommand {
    #[command(about = "List all known peers")]
    List,
    #[command(about = "Show peer details")]
    Info {
        #[arg(help = "Peer ID")]
        peer_id: String,
    },
}

fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("peerbox")
}

fn keypair_path() -> PathBuf {
    data_dir().join("keypair")
}

fn load_keypair() -> anyhow::Result<KeyPair> {
    let path = keypair_path();
    if !path.exists() {
        anyhow::bail!("No keypair found. Run 'peerbox init' to initialize a new node.");
    }
    let seed_bytes: Vec<u8> = std::fs::read(&path)?;
    let seed: [u8; 32] = seed_bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid keypair file"))?;
    Ok(KeyPair::from_seed(&seed))
}

fn save_keypair(keypair: &KeyPair) -> anyhow::Result<()> {
    std::fs::create_dir_all(data_dir())?;
    std::fs::write(keypair_path(), keypair.to_bytes())?;
    Ok(())
}

fn node_id_from_keypair(keypair: &KeyPair) -> NodeId {
    keypair.node_id()
}

pub async fn run(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Init => init::run().await,
        Commands::Identity { action } => identity::run(action).await,
        Commands::Upload { file } => upload::run(file).await,
        Commands::Download {
            manifest_id,
            output,
        } => download::run(manifest_id, output).await,
        Commands::Peers { action } => peers::run(action).await,
        Commands::Sync => sync::run().await,
        Commands::Status => status::run().await,
    }
}

async fn build_application() -> anyhow::Result<(Application, SimulatedTransport)> {
    let config = PeerBoxConfig::load()?;
    let keypair = load_keypair()?;
    let node_id = node_id_from_keypair(&keypair);

    let object_repo: Arc<dyn ObjectRepository> =
        Arc::new(SqliteObjectRepository::new(&config.database_url).await?);
    let event_repo: Arc<dyn EventRepository> =
        Arc::new(SqliteEventRepository::new(&config.database_url).await?);
    let event_bus = Arc::new(EventBus::new(256));
    let storage_provider = Arc::new(FileSystemStorageProvider::new(config.storage_dir.clone()));
    let crypto_provider: Arc<dyn crypto::CryptoProvider> =
        Arc::new(Ed25519CryptoProvider::new(keypair));

    let app = Application::new(
        object_repo,
        event_bus,
        storage_provider,
        crypto_provider,
        event_repo,
    );

    let peer_id = PeerId(node_id.0.clone());
    let network = Arc::new(tokio::sync::RwLock::new(SimulatedNetwork::new()));
    let transport = SimulatedTransport::new(network, peer_id);

    Ok((app, transport))
}
