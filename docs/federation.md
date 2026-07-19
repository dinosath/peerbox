# Federation

## ActivityPub Overview

Peerbox uses the [ActivityPub](https://www.w3.org/TR/activitypub/) protocol for decentralized node-to-node communication. Federation operates at the **metadata** level only: activities describe what objects exist, who owns them, and how they relate. Raw data transfer happens through the P2P (Iroh) layer, never through ActivityPub.

## Actor Types

### Node Actor

Represents a Peerbox node in the network:

```json
{
  "@context": ["https://www.w3.org/ns/activitystreams"],
  "type": "Application",
  "id": "https://node.example/actor",
  "name": "node-name",
  "inbox": "https://node.example/activitypub/inbox",
  "outbox": "https://node.example/activitypub/outbox",
  "publicKey": {
    "id": "https://node.example/actor#main-key",
    "owner": "https://node.example/actor",
    "publicKeyPem": "-----BEGIN PUBLIC KEY-----..."
  }
}
```

### User Actor

Represents a human user:

```json
{
  "type": "Person",
  "id": "https://node.example/users/alice",
  "preferredUsername": "alice",
  "name": "Alice",
  "inbox": "https://node.example/users/alice/inbox",
  "outbox": "https://node.example/users/alice/outbox"
}
```

### Organization Actor

Represents a group or organization:

```json
{
  "type": "Organization",
  "id": "https://node.example/orgs/my-org",
  "name": "My Organization",
  "inbox": "https://node.example/orgs/my-org/inbox",
  "outbox": "https://node.example/orgs/my-org/outbox"
}
```

## Activity Types

All activities follow the ActivityStreams 2.0 vocabulary and are JSON-LD signed.

### Create Activity

Announces a new object was created:

```json
{
  "type": "Create",
  "id": "https://node.example/activities/123",
  "actor": "https://node.example/actor",
  "object": {
    "type": "Document",
    "id": "https://node.example/objects/abc",
    "name": "file.txt",
    "manifest": "..."
  }
}
```

### Update Activity

Notifies followers that an object was modified:

```json
{
  "type": "Update",
  "actor": "https://node.example/actor",
  "object": {
    "type": "Document",
    "id": "https://node.example/objects/abc",
    "updated": "2024-01-01T00:00:00Z"
  }
}
```

### Delete Activity

Notifies followers that an object was removed:

```json
{
  "type": "Delete",
  "actor": "https://node.example/actor",
  "object": "https://node.example/objects/abc"
}
```

### Follow Activity

Requests to follow another actor:

```json
{
  "type": "Follow",
  "actor": "https://node-a.example/actor",
  "object": "https://node-b.example/actor"
}
```

### Accept Activity

Accepts a follow request:

```json
{
  "type": "Accept",
  "actor": "https://node-b.example/actor",
  "object": {
    "type": "Follow",
    "actor": "https://node-a.example/actor",
    "object": "https://node-b.example/actor"
  }
}
```

### Announce Activity

Shares/reposts an object to followers:

```json
{
  "type": "Announce",
  "actor": "https://node-a.example/actor",
  "object": "https://node-b.example/objects/xyz"
}
```

## Federation Flow

### Metadata Only, Never Bytes

```
Node A creates file.txt
    |
    v
Manifest generated (chunk hashes + transport info)
    |
    v
ActivityPub: Node A announces "Create Document" to followers
    |
    v
Node B receives Create activity in its inbox
    |
    v
Node B decides to replicate the file
    |
    v
P2P Layer (Iroh): Node B requests chunks from Node A
    |
    v
Node B verifies chunks against manifest hashes
    |
    v
Node B stores file locally
```

### Activity Delivery

1. Node creates an activity in its local outbox
2. FederationManager resolves recipient inbox URLs
3. HTTP POST with signed JSON-LD payload to each recipient
4. Recipient verifies signature using sender's public key
5. Recipient processes activity via ActivityHandler

## Inbox/Outbox Pattern

### Inbox (`POST /activitypub/inbox`)

Receives activities from other nodes:

```rust
pub trait ActivityHandler: Send + Sync {
    async fn handle_activity(&self, activity: &ActivityPubActivity) -> Result<()>;
}
```

- Activities are validated and verified
- Unknown activity types are logged and ignored
- Duplicate activities are deduplicated by ID

### Outbox

Queues activities for delivery:

```rust
pub trait Outbox: Send + Sync {
    async fn enqueue(&self, activity: ActivityPubActivity) -> Result<()>;
    async fn deliver_pending(&self) -> Result<()>;
}
```

- Failed deliveries are retried with exponential backoff
- Activities are stored persistently for crash safety
- Delivery status is tracked per recipient

## WebFinger Integration

### Discovery

```bash
GET /.well-known/webfinger?resource=acct:alice@node.example

Response:
{
  "subject": "acct:alice@node.example",
  "links": [
    {
      "rel": "self",
      "type": "application/activity+json",
      "href": "https://node.example/users/alice"
    }
  ]
}
```

### FederationManager

The `FederationManager` orchestrates all federation operations:
- Actor discovery via WebFinger
- Activity routing based on actor relationships
- Signature management
- Delivery scheduling

## Permission Model

### Access Control

```rust
pub enum Permission {
    Read,
    Write,
    Admin,
    Owner,
}

pub struct PermissionEntry {
    pub actor_id: String,
    pub object_id: String,
    pub permission: Permission,
}
```

### Default Permissions

| Action | Default | Requires |
|--------|---------|----------|
| Read public object | Allowed | No permission |
| Read private object | Denied | Read permission |
| Create object | Allowed | Write on parent |
| Update object | Denied | Write on object |
| Delete object | Denied | Admin or Owner |
| Follow actor | Allowed | No permission |
| Accept follow | Node decision | - |

## Node-to-Node Discovery

### Discovery Methods

1. **Bootstrap Nodes**: Configured in `config.bootstrap_nodes`
2. **mDNS**: Automatic LAN discovery
3. **WebFinger**: Resolve `acct:node@domain` queries
4. **Federation Graph**: Discover followers/following

### Bootstrap Configuration

```json
{
  "bootstrap_nodes": [
    "node1.peerbox.example:8080",
    "node2.peerbox.example:8080"
  ]
}
```

### Federation Graph Expansion

```
Node A follows Node B
    |
    v
Node A's outbox delivers to Node B's inbox
    |
    v
Node C follows Node A
    |
    v
Node C discovers Node B through Node A's following list
    |
    v
Node C can now follow Node B directly
```

## Error Handling

```rust
pub enum FederationError {
    ActorNotFound(String),
    InvalidActivity(String),
    DeliveryFailed(String),
    SerializationError(String),
    SignatureVerificationFailed(String),
    WebfingerError(String),
    Internal(String),
}
```

- All errors are logged with context
- Delivery failures are retried
- Invalid activities are rejected with HTTP 400
- Unknown actors return HTTP 404
- Internal errors return HTTP 500
