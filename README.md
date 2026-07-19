# Peerbox

A decentralized, local-first platform built in Rust. Peerbox provides a local application runtime for managing objects, storing metadata, emitting events, and persisting data locally. Built-in support for P2P data transfer (Iroh), ActivityPub federation, and content-addressed chunking.

## Architecture

```
Applications  -->  peerbox-server (HTTP)  |  peerbox (CLI)  |  Desktop (future)
                      |
Core Runtime  -->  ObjectService  |  EventBus  |  Application
                      |
Domain        -->  Objects  |  Crypto  |  Events  |  Manifest  |  Database
                      |
Infrastructure -->  Federation (ActivityPub)  |  P2P (Iroh)  |  Storage (FS/Mem)
```

The core runtime is completely independent from HTTP, UI frameworks, ActivityPub, Iroh, and any network protocols. Database access goes through repository interfaces only.

## Workspace Structure

```
peerbox/
├── Cargo.toml              # Workspace root
├── Cargo.lock
├── Dockerfile              # Multi-stage Docker build
├── docker-compose.yml      # Docker Compose deployment
├── .dockerignore
├── rust-toolchain.toml     # Rust toolchain config
├── deny.toml               # License/security audit config
├── README.md
├── crates/
│   ├── common/             # Shared types (ObjectId, NodeId, ContentHash, ChunkInfo)
│   ├── objects/            # Domain objects (FileObject, FolderObject)
│   ├── database/           # SQLite persistence via repository pattern
│   ├── events/             # Async event bus (tokio broadcast)
│   ├── storage/            # Pluggable storage (Memory, Filesystem)
│   ├── crypto/             # Ed25519 identity and signing
│   ├── core/               # Application runtime, object service, DI
│   ├── config/             # Configuration loading (JSON)
│   ├── server/             # Axum HTTP server (REST API + ActivityPub)
│   ├── cli/                # CLI tool (peerbox)
│   ├── chunking/           # File chunking/assembly with Blake3
│   ├── manifest/           # Content manifests with transport options
│   ├── verification/       # Progressive chunk verification
│   ├── federation/         # ActivityPub actors, activities, inbox/outbox
│   └── p2p/                # Iroh-based P2P networking
├── tests/                  # Integration tests
├── charts/peerbox/         # Helm chart for Kubernetes
└── docs/
    ├── architecture.md     # Architecture overview
    ├── development.md      # Development guide
    ├── deployment.md       # Deployment guide
    ├── security.md         # Security documentation
    ├── p2p.md              # P2P networking docs
    └── federation.md       # Federation docs
```

## Quick Start

```bash
# Build everything
cargo build

# Run the server
cargo run -p peerbox-server

# Run the CLI
cargo run -p peerbox-cli -- status

# Run tests
cargo test --workspace
```

## Docker

```bash
# Build
docker build -t peerbox:latest .

# Run with Compose
docker compose up -d
```

## Kubernetes (Helm)

```bash
helm install peerbox oci://ghcr.io/peerbox/peerbox/charts/peerbox \
  --namespace peerbox --create-namespace
```

## Key Components

### Object Model
Everything is an Object with a unique ID and creation timestamp. Built-in types: `FileObject`, `FolderObject`.

### Repository Pattern
Database access uses the repository pattern. `ObjectRepository` trait defines the interface; `SqliteObjectRepository` provides SQLite implementation. SQLx is isolated in the database crate.

### Event System
Async event bus on tokio broadcast channels. Events: `ObjectCreated`, `ObjectUpdated`, `ObjectDeleted`. Supports raw receivers and handler closures.

### Storage Abstraction
Pluggable `StorageProvider` trait (put/get/delete). Implementations: `MemoryStorageProvider`, `FileSystemStorageProvider`.

### Content Chunking
Files are split into configurable-size chunks (default 1MB) with Blake3 content hashing. Progressive verification detects corruption during transfer.

### P2P Networking
Iroh-based transport with mDNS discovery, bootstrap nodes, NAT traversal, and encrypted chunk transfer.

### ActivityPub Federation
Metadata-level federation: activities describe objects for discovery; data transfer happens via P2P layer. Supports Create, Update, Delete, Follow, Accept, Announce activities.

### Identity
Ed25519 keypairs. Node identity = hex-encoded public key (64 chars). Self-certifying, no central authority.

## Running Tests

```bash
# All tests
cargo test --workspace

# Specific crate
cargo test -p common
cargo test -p crypto
cargo test -p peerbox-server

# With coverage
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

## Documentation

- [Architecture](docs/architecture.md) - Overall design and crate dependencies
- [Development](docs/development.md) - Setup, building, testing, CI/CD
- [Deployment](docs/deployment.md) - Docker, Compose, Kubernetes, Helm
- [Security](docs/security.md) - Identity, signing, encryption, auditing
- [P2P Networking](docs/p2p.md) - Iroh, peer discovery, chunk transfer
- [Federation](docs/federation.md) - ActivityPub, actors, activities, WebFinger
