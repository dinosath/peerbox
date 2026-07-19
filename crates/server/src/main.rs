use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use crypto::DefaultCryptoProvider;
use database::StoredEvent;
use events::EventBus;
use peerbox_server::state::AppState;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let addr = std::env::var("LISTEN_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
        .parse::<SocketAddr>()?;

    let crypto = Arc::new(DefaultCryptoProvider::new());
    let node_id = hex::encode(crypto.keypair.public_key().to_bytes());

    let object_repo = Arc::new(InMemoryObjectRepository::new());
    let event_bus = Arc::new(EventBus::new(256));
    let storage = Arc::new(storage::MemoryStorageProvider::new());
    let event_repo = Arc::new(InMemoryEventRepository::new());

    let application = Arc::new(peerbox_core::Application::new(
        object_repo,
        event_bus,
        storage,
        crypto,
        event_repo,
    ));

    let state = Arc::new(AppState {
        application,
        node_id,
    });

    peerbox_server::run_server(state, addr).await?;
    Ok(())
}

use async_trait::async_trait;
use common::ObjectId;
use database::{ObjectRepository, StoredObject};
use std::collections::HashMap;
use tokio::sync::RwLock;

struct InMemoryObjectRepository {
    objects: RwLock<HashMap<String, StoredObject>>,
}

impl InMemoryObjectRepository {
    fn new() -> Self {
        Self {
            objects: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl ObjectRepository for InMemoryObjectRepository {
    async fn create(&self, object: StoredObject) -> anyhow::Result<()> {
        self.objects.write().await.insert(object.id.clone(), object);
        Ok(())
    }

    async fn get(&self, id: &ObjectId) -> anyhow::Result<Option<StoredObject>> {
        Ok(self.objects.read().await.get(&id.0).cloned())
    }

    async fn list(&self) -> anyhow::Result<Vec<StoredObject>> {
        Ok(self.objects.read().await.values().cloned().collect())
    }

    async fn update(&self, id: &ObjectId, data: serde_json::Value) -> anyhow::Result<()> {
        if let Some(obj) = self.objects.write().await.get_mut(&id.0) {
            obj.data = data;
        }
        Ok(())
    }

    async fn delete(&self, id: &ObjectId) -> anyhow::Result<()> {
        self.objects.write().await.remove(&id.0);
        Ok(())
    }
}

struct InMemoryEventRepository {
    events: RwLock<Vec<StoredEvent>>,
}

impl InMemoryEventRepository {
    fn new() -> Self {
        Self {
            events: RwLock::new(Vec::new()),
        }
    }
}

#[async_trait]
impl database::EventRepository for InMemoryEventRepository {
    async fn store(&self, event: StoredEvent) -> anyhow::Result<()> {
        self.events.write().await.push(event);
        Ok(())
    }

    async fn list(&self) -> anyhow::Result<Vec<StoredEvent>> {
        Ok(self.events.read().await.clone())
    }
}
