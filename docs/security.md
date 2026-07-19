# Security

## Identity and Key Management

### Ed25519 KeyPais

Every Peerbox node has an Ed25519 keypair generated on first run. The node's identity is derived from the public key:

```rust
let keypair = KeyPair::generate();
let node_id = hex::encode(keypair.public_key().to_bytes());
// node_id is a 64-character hex string
```

- **Key Generation**: Uses `OsRng` for cryptographically secure randomness.
- **Key Storage**: Keys are stored in the node's data directory. Protect this directory with filesystem permissions (600).
- **Key Serialization**: 32-byte seed for deterministic key recovery.

### Public Key Infrastructure

- Node identity = hex-encoded Ed25519 public key (64 hex chars)
- Public keys are embedded in ActivityPub actor objects
- Remote nodes verify signatures using the actor's public key

## Data Signing

All inter-node communications are signed with the node's Ed25519 key:

- ActivityPub activities include `https://w3id.org/security#signature` assertions
- P2P transport messages are signed for authenticity
- Content manifests are hashed (Blake3) but not signed (content-addressed)

### Signature Verification

```rust
let valid = crypto_provider.verify(data, signature).await?;
// Rejects tampered messages
// Rejects messages from unknown nodes
```

## Encrypted Transfers

### Iroh Transport

- Iroh provides end-to-end encrypted data transfer between nodes
- Chunk data is encrypted in transit
- Manifests may be shared in plaintext for discovery

## Permission Model

### Object Permissions

The federation layer implements a permission system:

- **Owner**: Full read/write access
- **Organization**: Shared access within an organization
- **Public**: Read-only for federation peers
- **Private**: No external access

```rust
pub enum Permission {
    Read,
    Write,
    Admin,
    Owner,
}
```

### Activity-Specific Permissions

- **Create**: Requires write permission on parent collection
- **Update**: Requires write permission on the object
- **Delete**: Requires admin permission on the object
- **Follow/Accept**: Unauthenticated (controlled by node policy)
- **Announce**: Requires read permission on the original object

## Secret Handling

### Best Practices

1. **Never commit secrets**: Use `.dockerignore`, `.gitignore` for key material.
2. **Environment variables**: Use `PEERBOX_*` prefix for all configuration.
3. **Kubernetes Secrets**: Store key material in `Secret` resources, never in ConfigMaps.
4. **Volume mounts**: Protect key files with `0600` permissions.

### Helm Secret Template

The Helm chart includes a `Secret` resource at `templates/secret.yaml` for keypair storage:

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: peerbox-keypair
type: Opaque
data:
  keypair-seed: <base64-encoded-32-byte-seed>
```

## Dependency Auditing

### cargo audit

Scans dependencies for known vulnerabilities (CVEs):

```bash
cargo audit
```

Runs in CI on every push.

### cargo deny

Enforces license compliance and bans specific crates:

```bash
cargo deny check
```

Configuration in `deny.toml`:
- **Allowed licenses**: MIT, Apache-2.0, BSD-3-Clause, BSL-1.0, CC0-1.0, ISC, Unicode-3.0, Zlib
- **Vulnerabilities**: Denied
- **Unmaintained crates**: Denied
- **Yanked crates**: Denied

## Container Security

### Multi-Stage Build

The Dockerfile uses a multi-stage build:
1. **Builder stage** (`rust:slim-bookworm`): Full toolchain for compilation
2. **Runtime stage** (`debian:bookworm-slim`): Minimal attack surface

### Rootless Container

```dockerfile
RUN adduser --uid 10001 --disabled-password --gecos "" peerbox
USER peerbox
```

- Non-root user with UID 10001
- No password (disabled)
- No login shell

### Minimal Base Image

- Uses `debian:bookworm-slim` (not full Debian)
- Only `ca-certificates` installed for TLS verification
- No development tools, shells, or package managers in runtime

### Kubernetes Security Context

```yaml
securityContext:
  runAsUser: 10001
  runAsGroup: 10001
  fsGroup: 10001
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop:
      - ALL
```

### Read-Only Root Filesystem

The Helm chart configures `readOnlyRootFilesystem: true`. Only the data directory (`/home/peerbox/.local/share/peerbox`) is writable via persistent volume.

## Secure Defaults

| Setting | Default | Rationale |
|---------|---------|-----------|
| Listen address | `127.0.0.1:3000` | Local-only by default |
| Federation | Disabled | Opt-in federation |
| TLS | Required for prod | Setup via Ingress/nginx |
| Key generation | `OsRng` | Cryptographic randomness |
| Log level | `info` | No sensitive data in logs |
| Root filesystem | Read-only | Prevent runtime modifications |

## Reporting Vulnerabilities

Report security issues to the project maintainers. Do not open public issues for security vulnerabilities.
