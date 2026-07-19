use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectId(pub String);

impl ObjectId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for ObjectId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ObjectId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<String> for ObjectId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for ObjectId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl NodeId {
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4().to_string())
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub type Timestamp = DateTime<Utc>;

pub fn now() -> Timestamp {
    Utc::now()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    FileObject,
    FolderObject,
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::FileObject => write!(f, "FileObject"),
            ObjectType::FolderObject => write!(f, "FolderObject"),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("object not found: {0}")]
    NotFound(ObjectId),

    #[error("database error: {0}")]
    Database(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("event error: {0}")]
    Event(String),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("manifest error: {0}")]
    Manifest(String),

    #[error("chunk error: {0}")]
    Chunk(String),

    #[error("verification error: {0}")]
    Verification(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("transport error: {0}")]
    Transport(String),
}

#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ContentHash {
    pub algorithm: String,
    pub hash: String,
}

impl ContentHash {
    pub fn new_sha256(data: &[u8]) -> Self {
        use sha2::Digest;
        let hash = sha2::Sha256::digest(data);
        Self {
            algorithm: "sha256".to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn new_blake3(data: &[u8]) -> Self {
        let hash = blake3::hash(data);
        Self {
            algorithm: "blake3".to_string(),
            hash: hex::encode(hash.as_bytes()),
        }
    }

    pub fn from_blake3(hash: &[u8; 32]) -> Self {
        Self {
            algorithm: "blake3".to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.hash
    }
}

impl fmt::Display for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.algorithm, self.hash)
    }
}

impl fmt::Debug for ContentHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ContentHash({})", self)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChunkInfo {
    pub index: u64,
    pub offset: u64,
    pub size: u64,
    pub hash: ContentHash,
}

pub type ManifestId = ContentHash;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_object_id_creation() {
        let id = ObjectId::new();
        assert!(!id.0.is_empty());
    }

    #[test]
    fn test_object_id_equality() {
        let id1 = ObjectId::from("abc");
        let id2 = ObjectId::from("abc");
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_object_id_display() {
        let id = ObjectId::from("test-id");
        assert_eq!(id.to_string(), "test-id");
    }

    #[test]
    fn test_object_id_serialization() {
        let id = ObjectId::from("test-id");
        let json = serde_json::to_string(&id).unwrap();
        let deserialized: ObjectId = serde_json::from_str(&json).unwrap();
        assert_eq!(id, deserialized);
    }

    #[test]
    fn test_node_id_creation() {
        let id = NodeId::new();
        assert!(!id.0.is_empty());
    }

    #[test]
    fn test_now_is_utc() {
        let ts = now();
        assert!(!ts.to_string().is_empty());
    }

    #[test]
    fn test_content_hash_blake3() {
        let data = b"hello world";
        let ch = ContentHash::new_blake3(data);
        assert_eq!(ch.algorithm, "blake3");
        assert!(!ch.hash.is_empty());
    }

    #[test]
    fn test_content_hash_display() {
        let data = b"test";
        let ch = ContentHash::new_blake3(data);
        let display = ch.to_string();
        assert!(display.starts_with("blake3:"));
        assert!(display.len() > "blake3:".len());
    }

    #[test]
    fn test_content_hash_serialization_roundtrip() {
        let data = b"roundtrip test";
        let ch = ContentHash::new_blake3(data);
        let json = serde_json::to_string(&ch).unwrap();
        let deserialized: ContentHash = serde_json::from_str(&json).unwrap();
        assert_eq!(ch, deserialized);
        assert_eq!(ch.algorithm, deserialized.algorithm);
        assert_eq!(ch.hash, deserialized.hash);
    }

    #[test]
    fn test_chunk_info_serialization() {
        let data = b"chunk data";
        let ch = ChunkInfo {
            index: 0,
            offset: 1024,
            size: 4096,
            hash: ContentHash::new_blake3(data),
        };
        let json = serde_json::to_string(&ch).unwrap();
        let deserialized: ChunkInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(ch.index, deserialized.index);
        assert_eq!(ch.offset, deserialized.offset);
        assert_eq!(ch.size, deserialized.size);
        assert_eq!(ch.hash, deserialized.hash);
    }

    #[test]
    fn test_content_hash_from_blake3() {
        let data = b"raw hash test";
        let raw = blake3::hash(data);
        let ch = ContentHash::from_blake3(raw.as_bytes());
        assert_eq!(ch.algorithm, "blake3");
        assert_eq!(ch.hash, hex::encode(raw.as_bytes()));
    }

    #[test]
    fn test_content_hash_sha256() {
        let data = b"sha256 test";
        let ch = ContentHash::new_sha256(data);
        assert_eq!(ch.algorithm, "sha256");
        assert!(!ch.hash.is_empty());
    }

    #[test]
    fn test_content_hash_equality() {
        let data = b"same data";
        let ch1 = ContentHash::new_blake3(data);
        let ch2 = ContentHash::new_blake3(data);
        assert_eq!(ch1, ch2);
    }

    #[test]
    fn test_manifest_id_alias() {
        let data = b"manifest";
        let mid: ManifestId = ContentHash::new_blake3(data);
        assert_eq!(mid.algorithm, "blake3");
    }
}
