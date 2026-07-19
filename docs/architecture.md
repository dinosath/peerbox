# Peerbox Architecture

## Overview

Peerbox is a decentralized, local-first platform built in Rust. It provides a local application runtime for managing objects, storing metadata, emitting events, and persisting data locally. All components (server, CLI, federation, P2P transport) are built on a shared core.

## Architecture Diagram

```
+------------------------------------------------------------------+
|                        APPLICATIONS                               |
|  +----------+  +----------+  +----------+  +-----------+          |
|  | peerbox-server|  |  peerbox |  | Desktop  |  |  Future   |          |
|  | (HTTP)   |  |   (CLI)  |  |  (GUI)   |  |  Clients  |          |
|  +----+-----+  +----+-----+  +----+-----+  +-----+-----+          |
|       |             |             |               |                |
+------------------------------------------------------------------+
|       |             |             |               |                |
|  +----v-------------v-------------v---------------v-----------+   |
|  |                     CORE RUNTIME                          |   |
|  |  +------------+  +----------+  +------------+             |   |
|  |  | ObjectSvc  |  | EventBus |  | Application|             |   |
|  |  +------------+  +----------+  +------------+             |   |
|  +----+---------+-----+-------+------+-------+---------------+   |
|       |         |     |       |      |       |                   |
+------------------------------------------------------------------+
|       |         |     |       |      |       |                   |
|  +----v---+ +---v--+ +v------+ +---v--+ +---v------+            |
|  |Objects | |Crypto| |Events | |Manifest| |Database |            |
|  +--------+ +------+ +-------+ +--------+ +---------+            |
|       |         |     |       |      |       |                   |
+------------------------------------------------------------------+
|       |         |     |       |      |       |                   |
|  +----v---------v-----v-------v------v-------v---------------+   |
|  |              ACTIVITYPUB / IROH / STORAGE                 |   |
|  |  +-------------+  +----------+  +----------------+        |   |
|  |  | Federation  |  |   P2P    |  | Storage (FS/Mem)|       |   |
|  |  | (ActivityPub|  | (Iroh)   |  +----------------+        |   |
|  |  |  actor/act) |  +----------+                            |   |
|  |  +-------------+                                          |   |
|  +----+-------------------+----------------------------------+   |
|       |                   |                                      |
+------------------------------------------------------------------+
|       |                   |                                      |
|  +----v--------+  +------v--------+                              |
|  | Content     |  | Local /       |                              |
|  | Manifests   |  | Cloud Storage |                              |
|  | (Blake3)    |  | (SQLite, FS)  |                              |
|  +-------------+  +---------------+                              |
+------------------------------------------------------------------+
```

## Layer Descriptions

### Applications
- **peerbox-server**: Axum-based HTTP server exposing REST API and ActivityPub endpoints for federation.
- **peerbox (CLI)**: Command-line tool for node management, file upload/download, sync operations, and identity management.
- **Desktop** (future): GUI application for end-user interaction.
- **Future Clients**: Mobile, web, or other platform clients.

### Core Runtime
The core runtime is completely independent from HTTP, UI frameworks, ActivityPub, Iroh, and any network protocols. It follows a clean layered architecture:
- **Application**: Orchestrates services and dependency injection.
- **ObjectService**: CRUD operations on objects with event emission.
- **EventBus**: Asynchronous publish/subscribe using tokio broadcast channels.

### ActivityPub / Iroh / Storage
- **Federation**: Implements ActivityPub actors, activities, inbox/outbox, and WebFinger for decentralized discovery.
- **P2P**: Iroh-based peer-to-peer networking for direct data transfer between nodes.
- **Storage**: Pluggable storage abstraction with in-memory and filesystem providers.

### Content Manifests
- **Chunking**: Splits files into fixed-size chunks with Blake3 hashing.
- **Manifest**: Describes file structure, chunk layout, transport options, and verification info.
- **Verification**: Progressive verification of chunks during transfer with corruption detection.

### Local / Cloud Storage
- **SQLite**: Persistent metadata storage via repository pattern.
- **Filesystem**: Local file storage for chunk data.

## Crate Dependency Graph

```
applications
     |
     v
   core  <-- server API
     |
     v
  common
     ^
     |
objects  database  events  storage  crypto
  ^        ^        ^        ^        ^
  |        |        |        |        |
  +--------+--------+--------+--------+
                    |
            federation  p2p  chunking  manifest  verification
```

### Crate Descriptions

| Crate | Description |
|-------|-------------|
| `common` | Shared types: ObjectId, NodeId, ContentHash, ChunkInfo, Timestamp, Error |
| `objects` | Domain objects: FileObject, FolderObject with Object trait |
| `database` | SQLite persistence via ObjectRepository and EventRepository traits |
| `events` | Async event bus with Event enum (Created, Updated, Deleted) |
| `storage` | Pluggable storage: MemoryStorageProvider, FileSystemStorageProvider |
| `crypto` | Ed25519 identity: KeyPair, CryptoProvider trait, DefaultCryptoProvider |
| `core` | Application runtime, ObjectService, dependency injection |
| `config` | Configuration loading/saving: PeerBoxConfig with serde JSON |
| `server` | Axum HTTP server with REST routes and ActivityPub endpoints |
| `cli` | Command-line interface via clap with init, upload, download, sync, identity commands |
| `manifest` | Content manifests with chunk info and transport options |
| `chunking` | File chunking/assembly with Blake3 verification |
| `verification` | Full and progressive chunk verification with corruption detection |
| `federation` | ActivityPub actors, activities, inbox/outbox, WebFinger, permissions |
| `p2p` | Peer-to-peer networking: identity, transport, transfer, simulated mode |

## Data Flow

### Upload
```
Client -> peerbox-server HTTP POST /objects
  -> ObjectService.create()
  -> Chunker splits data
  -> Manifest generated (Blake3 hashes)
  -> ObjectRepository stores metadata
  -> StorageProvider stores chunks
  -> EventBus emits ObjectCreated
  -> Federation Outbox sends Create activity (if enabled)
```

### Download
```
Client -> peerbox-server HTTP GET /objects/{id}
  -> ObjectService.get()
  -> Manifest retrieved
  -> ChunkAssembler collects chunks from StorageProvider
  -> Verifier checks chunk integrity
  -> Assembled data returned to client
```

### Sync
```
peerbox sync command
  -> P2P transport discovers peers
  -> Manifest exchange between nodes
  -> ProgressiveVerifier validates incoming chunks
  -> Missed chunks requested from peers
  -> Local storage updated
```

## Design Decisions

1. **Repository Pattern**: Database access through trait interfaces. Core crate never depends on SQLx directly.
2. **Async-First**: All I/O operations use tokio async runtime.
3. **Content-Addressed**: Blake3 hashing for all content identifiers.
4. **Pluggable Transport**: Manifest includes transport list (Iroh, HTTPS, IPFS, Local).
5. **Metadata Federation Only**: ActivityPub federation exchanges metadata, never raw bytes. Data transfer happens via P2P (Iroh) layer.
6. **Ed25519 Identity**: Node identity derived from Ed25519 public key (hex-encoded).
7. **Configuration as Code**: JSON configuration files with environment variable overrides.
8. **Zero Downtime Migrations**: SQLite schema managed via sqlx migrations.
