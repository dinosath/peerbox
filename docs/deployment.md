# Deployment Guide

## Binary Deployment

### Server

```bash
# Build
cargo build --release -p dc-server

# Run
LISTEN_ADDR=0.0.0.0:8080 ./target/release/dc-server
```

### CLI

```bash
# Build
cargo build --release -p dcc

# Initialize a node
./target/release/dcc init

# Start syncing
./target/release/dcc sync
```

## Docker Deployment

### Building the Image

```bash
# Default build
docker build -t peerbox:latest .

# With custom version
docker build --build-arg APP_VERSION=0.2.0 -t peerbox:0.2.0 .

# Multi-platform build (requires buildx)
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t ghcr.io/peerbox/peerbox:latest \
  --push .
```

### Running the Container

```bash
# Server mode (default entrypoint)
docker run -d \
  -p 8080:8080 \
  -v peerbox-data:/home/peerbox/.local/share/peerbox \
  -v peerbox-config:/home/peerbox/.config/peerbox \
  -e PEERBOX_NODE_NAME=my-node \
  -e RUST_LOG=info \
  peerbox:latest

# CLI mode
docker run --rm \
  -v peerbox-data:/home/peerbox/.local/share/peerbox \
  peerbox:latest dcc status
```

## Docker Compose

```bash
# Start all services
docker compose up -d

# View logs
docker compose logs -f peerbox-server

# Check node status
docker compose exec peerbox-server dcc status

# Stop
docker compose down
```

### Services

| Service | Role | Port | Notes |
|---------|------|------|-------|
| `peerbox-server` | HTTP API server | 8080 | REST API + federation |
| `peerbox-storage` | Storage node | - | `dcc sync` only |
| `peerbox-federation` | Federation node | - | ActivityPub enabled |

## Kubernetes Deployment via Helm

### Prerequisites

- Kubernetes cluster 1.25+
- Helm 3.15+
- Container registry access (GHCR)

### Installation

```bash
# OCI registry install
helm install peerbox oci://ghcr.io/peerbox/peerbox/charts/peerbox \
  --namespace peerbox \
  --create-namespace

# From local chart
helm install peerbox ./charts/peerbox \
  --namespace peerbox \
  --create-namespace

# With custom values
helm install peerbox ./charts/peerbox \
  --namespace peerbox \
  --set config.nodeName=prod-node-1 \
  --set config.federationEnabled=true \
  --set persistence.size=50Gi
```

### Upgrade

```bash
helm upgrade peerbox ./charts/peerbox \
  --namespace peerbox \
  --set image.tag=0.2.0
```

### Uninstall

```bash
helm uninstall peerbox --namespace peerbox
```

### Key Values

| Parameter | Default | Description |
|-----------|---------|-------------|
| `replicaCount` | `1` | Number of replicas |
| `image.repository` | `ghcr.io/peerbox/peerbox` | Image repository |
| `image.tag` | `""` (Chart appVersion) | Image tag |
| `config.nodeName` | `peerbox-node` | Node name |
| `config.logLevel` | `info` | Log level (info, debug, warn, error) |
| `config.federationEnabled` | `false` | Enable ActivityPub federation |
| `config.listenPort` | `8080` | HTTP listen port |
| `config.bootstrapNodes` | `[]` | Bootstrap peer addresses |
| `service.type` | `ClusterIP` | Service type |
| `service.port` | `8080` | Service port |
| `persistence.enabled` | `true` | Enable persistent storage |
| `persistence.size` | `10Gi` | PVC size |
| `ingress.enabled` | `false` | Enable ingress |
| `monitoring.serviceMonitor.enabled` | `false` | Enable Prometheus monitoring |
| `autoscaling.enabled` | `false` | Enable HPA |
| `resources.limits.cpu` | `500m` | CPU limit |
| `resources.limits.memory` | `512Mi` | Memory limit |
| `resources.requests.cpu` | `100m` | CPU request |
| `resources.requests.memory` | `128Mi` | Memory request |

## Configuration Reference

### Configuration File (`~/.config/peerbox/config.json`)

```json
{
  "node_name": "peerbox-node",
  "data_dir": "/home/peerbox/.local/share/peerbox",
  "database_url": "sqlite:///home/peerbox/.local/share/peerbox/database.sqlite?mode=rwc",
  "storage_dir": "/home/peerbox/.local/share/peerbox/storage",
  "listen_port": 8080,
  "bootstrap_nodes": [],
  "federation_enabled": false,
  "log_level": "info"
}
```

### Environment Variables

| Variable | Config Key | Description |
|----------|-----------|-------------|
| `PEERBOX_NODE_NAME` | `node_name` | Node display name |
| `PEERBOX_LISTEN_PORT` | `listen_port` | HTTP server port |
| `LISTEN_ADDR` | - | Listen address (format: `ip:port`) |
| `RUST_LOG` | `log_level` | Log filter (e.g., `info`, `debug`) |
| `PEERBOX_FEDERATION_ENABLED` | `federation_enabled` | Enable federation |
| `PEERBOX_BOOTSTRAP_NODES` | `bootstrap_nodes` | Comma-separated node addresses |

### Volume Mounts

| Path | Purpose |
|------|---------|
| `/home/peerbox/.local/share/peerbox` | Data directory (SQLite DB, chunks) |
| `/home/peerbox/.config/peerbox` | Configuration (config.json) |

## Network Requirements

| Port | Protocol | Purpose |
|------|----------|---------|
| 8080 | TCP | HTTP API server |
| Variable | TCP/UDP | P2P Iroh transport (NAT traversal) |

For federation, the node must be reachable at a public HTTPS URL configured via ActivityPub actor endpoints.
