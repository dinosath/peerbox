# Peerbox

A decentralized, local-first platform built in Rust. The system provides a local application
runtime capable of managing objects, storing metadata, emitting events, and persisting data
locally. All future components (desktop, server, CLI, federation, transport) will be built on
this foundation.

## Architecture

The core runtime is completely independent from HTTP, UI frameworks, ActivityPub, Iroh,
and any network protocols. It follows a clean layered architecture:

```
common    — shared types (ObjectId, NodeId, Timestamp)
objects   — domain objects (FileObject, FolderObject)
database  — SQLite persistence via repository pattern
events    — async event bus (tokio broadcast channels)
storage   — pluggable storage abstraction
crypto    — identity primitives (placeholder)
core      — application runtime, object service, DI
```

### Dependency Flow

```
objects
   |
   v
database
   |
   v
 core
   |
   v
applications (CLI, desktop, server, federation)
```

The core crate never depends on SQLx, Axum, Makepad, or ActivityPub directly.
Database access goes through repository interfaces only.

## Workspace Structure

```
peerbox/
├── Cargo.toml          # Workspace root
├── crates/
│   ├── common/         # Shared types
│   ├── objects/        # Domain objects
│   ├── database/       # SQLite persistence
│   ├── events/         # Event bus
│   ├── storage/        # Storage abstraction
│   ├── crypto/         # Identity placeholders
│   └── core/           # Application runtime
├── tests/              # Integration tests
├── data/               # Local data (SQLite DB)
└── README.md
```

## Database Setup

SQLite is used for local persistence. The database file is created at `data/database.sqlite`
by default. Tables are auto-created on first connection.

### Tables

- **objects** — stores all object metadata as JSON
- **events** — persists event history

## Running Tests

```bash
# Run all unit tests
cargo test

# Run with logging
RUST_LOG=info cargo test -- --nocapture

# Run specific crate tests
cargo test -p common
cargo test -p objects
cargo test -p database
cargo test -p events
cargo test -p storage
cargo test -p core

# Run integration tests
cargo test -p integration-tests
```

## Building

```bash
cargo build
cargo build --release
```

## Key Components

### Object Model

Everything in the system is an Object. Each object has a unique ID and creation timestamp.
Built-in types include `FileObject` and `FolderObject`.

### Repository Pattern

Database access uses the repository pattern. The `ObjectRepository` trait defines the
interface, and `SqliteObjectRepository` provides the SQLite implementation. This keeps
SQLx isolated in the database crate.

### Event System

An asynchronous event bus built on tokio broadcast channels. Supports publishing events
and subscribing with either raw receivers or handler closures. Events include
`ObjectCreated`, `ObjectUpdated`, and `ObjectDeleted`.

### Storage Abstraction

The `StorageProvider` trait defines a pluggable storage interface with `put`, `get`, and
`delete` operations. `MemoryStorageProvider` is provided for testing.

## Milestone 1 Completion Criteria

- [x] Cargo workspace builds
- [x] Core runs without network
- [x] SQLite persistence works
- [x] Objects can be created/read
- [x] Event system works asynchronously
- [x] Storage abstraction exists
- [x] Tests pass
- [x] No UI/network dependencies in core
