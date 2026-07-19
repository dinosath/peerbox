# Development Guide

## Prerequisites

- **Rust**: Latest stable toolchain (see `rust-toolchain.toml`)
  ```bash
  rustup update stable
  ```
- **Cargo tools**:
  ```bash
  rustup component add rustfmt clippy llvm-tools-preview
  cargo install cargo-deny cargo-audit
  ```
- **SQLite**: Development libraries (libsqlite3-dev on Debian/Ubuntu)

## Getting Started

```bash
git clone <repository-url>
cd peerbox
cargo build
```

## Building All Targets

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Build specific crate
cargo build -p peerbox-server
cargo build -p peerbox-cli

# Cross-compile for ARM64 (requires cross or target toolchain)
rustup target add aarch64-unknown-linux-gnu
cargo build --release --target aarch64-unknown-linux-gnu
```

## Running the Server

```bash
cargo run -p peerbox-server

# With custom address
LISTEN_ADDR=0.0.0.0:8080 cargo run -p peerbox-server
```

## Running the CLI

```bash
# Initialize a node
cargo run -p peerbox-cli -- init

# Show node status
cargo run -p peerbox-cli -- status

# Show node identity
cargo run -p peerbox-cli -- identity

# Upload a file
cargo run -p peerbox-cli -- upload /path/to/file

# Download by object ID
cargo run -p peerbox-cli -- download <object-id>

# Sync with peers
cargo run -p peerbox-cli -- sync

# List peers
cargo run -p peerbox-cli -- peers
```

## Running Tests

```bash
# Run all workspace tests
cargo test --workspace

# Run with output
RUST_LOG=info cargo test --workspace -- --nocapture

# Run specific crate tests
cargo test -p common
cargo test -p objects
cargo test -p database
cargo test -p events
cargo test -p storage
cargo test -p crypto
cargo test -p core
cargo test -p config
cargo test -p peerbox-server
cargo test -p peerbox-cli
cargo test -p chunking
cargo test -p manifest
cargo test -p verification
cargo test -p federation
cargo test -p p2p

# Run integration tests
cargo test -p integration-tests

# Run with coverage
cargo llvm-cov --workspace --all-features --lcov --output-path lcov.info
```

## Linting and Formatting

```bash
# Format code
cargo fmt --all -- --check

# Lint with clippy
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Check for dependency vulnerabilities
cargo audit

# Check license compliance
cargo deny check
```

## Adding New Crates

1. Create the crate directory:
   ```bash
   mkdir crates/new-crate
   ```

2. Create `crates/new-crate/Cargo.toml`:
   ```toml
   [package]
   name = "new-crate"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   common = { workspace = true }
   tokio = { workspace = true }
   ```

3. Create `crates/new-crate/src/lib.rs` with your module code.

4. Register in workspace `Cargo.toml`:
   ```toml
   [workspace]
   members = [
       # ... existing members
       "crates/new-crate",
   ]

   [workspace.dependencies]
   new-crate = { path = "crates/new-crate" }
   ```

5. Update `deny.toml` if new licenses are introduced.

## Code Style Guidelines

- Follow standard Rust conventions (`cargo fmt`).
- Use `anyhow::Result` for fallible functions; `thiserror` for library error types.
- Use `async_trait` for async trait methods.
- Prefer `Arc<dyn Trait>` for dependency injection.
- Use `#[cfg(test)] mod tests;` in every crate for unit tests.
- Keep core crate free of network, UI, and database implementation details.
- Use the repository pattern for database access through traits.

## Testing Guidelines

- Every public function should have at least one test.
- Use `MemoryStorageProvider` and in-memory implementations for unit tests.
- Use `tempfile` for filesystem-backed tests.
- Use `tokio::test` for async test functions.
- Integration tests live in the `tests/` workspace member.
- Aim for >80% line coverage.

## CI/CD Pipeline

The CI/CD pipeline is defined in `.github/workflows/`:

| Workflow | Triggers | Description |
|----------|----------|-------------|
| `ci.yml` | Push, PR to main | Build, test, lint, audit |
| `docker.yml` | Push to main, PR, Release | Multi-arch Docker build + push to GHCR |
| `release.yml` | Tag `v*.*.*` | Cross-compile binaries, Docker, Helm, GitHub Release |
| `helm.yml` | Push to main (charts/), Tag, Release | Helm chart lint, test, package, push to GHCR |

### CI Steps
1. Build workspace
2. Run all tests
3. Run clippy
4. Check formatting
5. Run cargo-audit
6. Run cargo-deny

### Release Steps
1. Run tests
2. Build for linux/amd64 (native)
3. Build for linux/arm64 (cross)
4. Build for macOS (x86_64 + arm64)
5. Build for Windows (x86_64)
6. Build multi-arch Docker image
7. Package and push Helm chart
8. Create GitHub Release with artifacts

## Git Workflow

- **Main branch**: `main`
- **Feature branches**: `feature/<description>`
- **Bug fix branches**: `fix/<description>`
- **Release tags**: `v<major>.<minor>.<patch>`

```bash
# Start a feature
git checkout -b feature/my-feature main

# Commit changes
git add -A
git commit -m "feat: description"

# Push and create PR
git push -u origin feature/my-feature
# Open PR against main
```
