use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tokio::sync::Mutex;

use common::ObjectId;
use crypto::{CryptoProvider, DefaultCryptoProvider};
use database::{EventRepository, ObjectRepository, StoredEvent, StoredObject};
use events::EventBus;
use peerbox_core::Application;
use storage::{MemoryStorageProvider, StorageProvider};

struct MemoryObjectRepository {
    data: Mutex<HashMap<String, StoredObject>>,
}

impl MemoryObjectRepository {
    fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl ObjectRepository for MemoryObjectRepository {
    async fn create(&self, object: StoredObject) -> anyhow::Result<()> {
        self.data.lock().await.insert(object.id.clone(), object);
        Ok(())
    }
    async fn get(&self, id: &ObjectId) -> anyhow::Result<Option<StoredObject>> {
        Ok(self.data.lock().await.get(&id.0).cloned())
    }
    async fn list(&self) -> anyhow::Result<Vec<StoredObject>> {
        Ok(self.data.lock().await.values().cloned().collect())
    }
    async fn update(&self, id: &ObjectId, data: serde_json::Value) -> anyhow::Result<()> {
        if let Some(obj) = self.data.lock().await.get_mut(&id.0) {
            obj.data = data;
            obj.updated_at = Utc::now();
        }
        Ok(())
    }
    async fn delete(&self, id: &ObjectId) -> anyhow::Result<()> {
        self.data.lock().await.remove(&id.0);
        Ok(())
    }
}

struct MemoryEventRepository {
    events: Mutex<Vec<StoredEvent>>,
}

impl MemoryEventRepository {
    fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }
}

#[async_trait::async_trait]
impl EventRepository for MemoryEventRepository {
    async fn store(&self, event: StoredEvent) -> anyhow::Result<()> {
        self.events.lock().await.push(event);
        Ok(())
    }
    async fn list(&self) -> anyhow::Result<Vec<StoredEvent>> {
        Ok(self.events.lock().await.clone())
    }
}

pub fn build_peerbox_app() -> Arc<Application> {
    let obj_repo: Arc<dyn ObjectRepository> = Arc::new(MemoryObjectRepository::new());
    let event_bus = Arc::new(EventBus::new(256));
    let storage: Arc<dyn StorageProvider> = Arc::new(MemoryStorageProvider::new());
    let crypto: Arc<dyn CryptoProvider> = Arc::new(DefaultCryptoProvider::new());
    let event_repo: Arc<dyn EventRepository> = Arc::new(MemoryEventRepository::new());

    Arc::new(Application::new(
        obj_repo, event_bus, storage, crypto, event_repo,
    ))
}
