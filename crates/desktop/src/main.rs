#![allow(dead_code)]

mod app;
mod commands;
mod platform;
mod state;
mod views;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use common::ObjectId;
use crypto::{CryptoProvider, DefaultCryptoProvider};
use database::{EventRepository, ObjectRepository, StoredEvent, StoredObject};
use peerbox_core::Application;
use events::EventBus;
use storage::{MemoryStorageProvider, StorageProvider};

use app::DesktopApplication;
use commands::identity_ops::IdentityOps;

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
            obj.updated_at = chrono::Utc::now();
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    tracing::info!("starting peerbox desktop application");

    let config = config::PeerBoxConfig::load()?;
    tracing::info!("loaded config: node_name={}", config.node_name);

    let obj_repo: Arc<dyn ObjectRepository> = Arc::new(MemoryObjectRepository::new());
    let event_bus = Arc::new(EventBus::new(256));
    let storage: Arc<dyn StorageProvider> = Arc::new(MemoryStorageProvider::new());
    let crypto: Arc<dyn CryptoProvider> = Arc::new(DefaultCryptoProvider::new());
    let event_repo: Arc<dyn EventRepository> = Arc::new(MemoryEventRepository::new());

    let app = Arc::new(Application::new(
        obj_repo, event_bus, storage, crypto, event_repo,
    ));

    let identity = IdentityOps::generate();
    tracing::info!("node identity: {}", identity.get_node_id());

    let desktop = DesktopApplication::new(app, identity);

    tracing::info!("peerbox desktop running");

    desktop.run().await?;

    Ok(())
}
