# Peer-to-Peer Networking

## Iroh-Based Networking

Peerbox uses Iroh for peer-to-peer data transfer between nodes. Iroh provides NAT traversal, end-to-end encryption, and efficient chunk-based content transfer.

## Peer Discovery

### mDNS (Multicast DNS)

On local networks, nodes automatically discover each other via mDNS:
- No configuration needed
- Works on LAN only
- Nodes announce their presence on startup

### Bootstrap Nodes

For internet-scale discovery, nodes can be configured with bootstrap peer addresses:

```json
{
  "bootstrap_nodes": [
    "192.168.1.100:12345",
    "node1.peerbox.example:12345"
  ]
}
```

Bootstrap nodes are well-known peers that introduce new nodes to the network. They do not participate in data storage or transfer unless explicitly configured.

### Discovery Flow

```
1. Node starts
2. mDNS broadcast on local network
3. Connect to bootstrap nodes (if configured)
4. Receive peer list from bootstrap nodes
5. Connect to discovered peers
6. Exchange node information
```

## NAT Traversal

Iroh handles NAT traversal automatically:

- **STUN**: Discover public IP and port mapping
- **Relay**: Fallback relay server when direct connection fails
- **Hole punching**: UDP hole punching for NAT traversal

No manual port forwarding is required for most network configurations.

## Node Identity

### Identity Derivation

```rust
let keypair = KeyPair::generate();
let node_id = NodeId(hex::encode(keypair.public_key().to_bytes()));
```

- Identity = Ed25519 public key (32 bytes, hex-encoded to 64 chars)
- Self-certifying: identity can be verified by signature verification
- No central authority needed

### Peer Authentication

When connecting to a peer:
1. Peer presents its public key (identity)
2. Node issues a challenge (random nonce)
3. Peer signs the challenge with its private key
4. Node verifies the signature against the peer's public key

## Transport Interface

The P2P layer defines a transport abstraction:

```rust
pub trait Transport: Send + Sync {
    async fn send(&self, peer: &NodeId, data: Bytes) -> Result<()>;
    async fn recv(&self) -> Result<(NodeId, Bytes)>;
    async fn connect(&self, peer: &NodeId) -> Result<()>;
    async fn disconnect(&self, peer: &NodeId) -> Result<()>;
}
```

Supported transports:
- **Iroh**: Default, NAT-traversing encrypted transport
- **Simulated**: For testing (in-memory channel-based)

## Chunk Transfer Protocol

### Manifest-First Transfer

```
Downloader                          Uploader
    |                                   |
    |--- GET /manifest/{id} ----------->|
    |<-- Manifest (chunks + hashes) ----|
    |                                   |
    |--- Request chunks [0,5,12] ------>|
    |<-- Chunk 0 (Blake3 verified) -----|
    |<-- Chunk 5 (Blake3 verified) -----|
    |<-- Chunk 12 (Blake3 verified) ----|
    |                                   |
    |--- Request missing chunk [3] ---->|
    |<-- Chunk 3 (Blake3 verified) -----|
    |                                   |
    | Assembly + Verification complete  |
```

### Progressive Verification

Chunks are verified as they arrive using `ProgressiveVerifier`:

```rust
let mut verifier = ProgressiveVerifier::new(chunk_infos);
for chunk in incoming_chunks {
    let result = verifier.feed_chunk(chunk.index, chunk.data)?;
    if result == VerificationResult::Corrupted { /* re-request */ }
}
```

### Chunk Selection

Default chunk size: 1 MB (`DEFAULT_CHUNK_SIZE`). Chunks are selected by index:
- Missing chunks are tracked via `ChunkAssembler::missing_chunks()`
- Duplicate chunks are detected and rejected
- Out-of-order delivery is supported

## Connection Management

### Peer States

- **Disconnected**: Initial state, no active connection
- **Connecting**: Handshake in progress
- **Connected**: Active bidirectional channel
- **Degraded**: Connection exists but slow/unreliable

### Reconnection

- Exponential backoff for reconnection attempts
- Maximum retry count before marking peer as unreachable
- Peer list refresh from bootstrap nodes on disconnect

## Multi-Node Replication Scenarios

### Scenario 1: Direct Upload to One Node

```
Client --> Node A
              |
              v
         Node A stores chunk
         Node A announces to federation
         Node B discovers via federation
         Node B requests chunks from Node A
```

### Scenario 2: Client Connected to Multiple Nodes

```
Client --> Node A (chunks 0-4)
       --> Node B (chunks 5-9)
              |
              v
         Nodes exchange bitmaps
         Missing chunks requested peer-to-peer
         Full replication achieved
```

### Scenario 3: Offline First

```
Client creates object locally (Node A)
    |
    v
Event emitted: ObjectCreated
    |
    v
Federation Outbox queues Create activity
    |
    v (Node goes online)
Activity delivered to followers
Peers pull missing chunks
```

## Network Failure Handling

### Chunk Transfer Failures

1. **Corrupted chunk**: Detected by Blake3 hash mismatch, re-requested from same or different peer
2. **Timeouts**: Chunk request times out after 30 seconds, marked as failed
3. **Peer disconnection**: In-progress downloads reassigned to available peers
4. **Partial downloads**: `ResumableDownload` tracks progress, resumes from last verified chunk

### Node Failures

- **Storage node offline**: Data fetched from alternative peers with the same content
- **Federation node offline**: Activities queued in outbox, delivered on reconnection
- **Bootstrap node offline**: Node uses cached peer list until bootstrap comes back

### Consistency

- Eventual consistency model
- No consensus algorithm (no Raft/Paxos)
- Objects are content-addressed; conflicts resolved by content hash
- Federation metadata may have temporary inconsistencies (resolved on next sync)
