use async_trait::async_trait;
use chrono::{DateTime, Utc};
use common::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredObject {
    pub id: String,
    pub object_type: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredEvent {
    pub id: String,
    pub event_type: String,
    pub payload: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[async_trait]
pub trait ObjectRepository: Send + Sync {
    async fn create(&self, object: StoredObject) -> anyhow::Result<()>;
    async fn get(&self, id: &ObjectId) -> anyhow::Result<Option<StoredObject>>;
    async fn list(&self) -> anyhow::Result<Vec<StoredObject>>;
    async fn update(&self, id: &ObjectId, data: serde_json::Value) -> anyhow::Result<()>;
    async fn delete(&self, id: &ObjectId) -> anyhow::Result<()>;
}

#[async_trait]
pub trait EventRepository: Send + Sync {
    async fn store(&self, event: StoredEvent) -> anyhow::Result<()>;
    async fn list(&self) -> anyhow::Result<Vec<StoredEvent>>;
}

mod sqlite;
pub use sqlite::SqliteEventRepository;
pub use sqlite::SqliteObjectRepository;
