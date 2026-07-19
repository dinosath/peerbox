use p2p::transport::Transport;

use super::{build_application, PeersCommand};

pub async fn run(action: Option<PeersCommand>) -> anyhow::Result<()> {
    match action {
        Some(PeersCommand::List) | None => list().await,
        Some(PeersCommand::Info { peer_id }) => info(peer_id).await,
    }
}

async fn list() -> anyhow::Result<()> {
    let (_app, transport) = build_application().await?;

    let peers = transport.discover_peers().await?;

    if peers.is_empty() {
        println!("No peers connected. The network is simulated in this version.");
        println!();
        println!("Your node ID: {}", transport.peer_id());
        return Ok(());
    }

    println!("Connected peers:");
    for peer in &peers {
        let status = if peer.connected {
            "connected"
        } else {
            "disconnected"
        };
        println!(
            "  {}  {}  {}",
            peer.id,
            status,
            peer.addresses.first().map(|s| s.as_str()).unwrap_or("-")
        );
    }
    println!();

    Ok(())
}

async fn info(peer_id: String) -> anyhow::Result<()> {
    let (_app, transport) = build_application().await?;

    let peers = transport.discover_peers().await?;
    let peer = peers.iter().find(|p| p.id.0 == peer_id);

    match peer {
        Some(p) => {
            println!("Peer: {}", p.id);
            println!("  Node ID:    {}", p.node_id);
            println!("  Addresses:  {}", p.addresses.join(", "));
            println!(
                "  Status:     {}",
                if p.connected {
                    "connected"
                } else {
                    "disconnected"
                }
            );
            println!("  Last seen:  {}", p.last_seen);
        }
        None => {
            println!("Peer not found: {}", peer_id);
        }
    }
    println!();

    Ok(())
}
