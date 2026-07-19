use std::sync::Arc;

use chrono::Utc;
use common::ObjectId;
use crypto::CryptoProvider;
use database::{EventRepository, ObjectRepository, StoredEvent, StoredObject};
use events::{Event, EventBus};
use storage::StorageProvider;
use tracing::info;

use super::ObjectService;

pub struct Subscriber {
    event_bus: Arc<EventBus>,
    event_repository: Arc<dyn EventRepository>,
}

impl Subscriber {
    pub fn new(event_bus: Arc<EventBus>, event_repository: Arc<dyn EventRepository>) -> Self {
        Self {
            event_bus,
            event_repository,
        }
    }

    pub async fn start(&self) {
        let mut rx = self.event_bus.subscribe();
        let repo = self.event_repository.clone();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                let payload = match &event {
                    Event::ObjectCreated { id } => {
                        serde_json::json!({
                            "type": "ObjectCreated",
                            "id": id.0
                        })
                    }
                    Event::ObjectUpdated { id } => {
                        serde_json::json!({
                            "type": "ObjectUpdated",
                            "id": id.0
                        })
                    }
                    Event::ObjectDeleted { id } => {
                        serde_json::json!({
                            "type": "ObjectDeleted",
                            "id": id.0
                        })
                    }
                };

                let stored = StoredEvent {
                    id: ObjectId::new().0,
                    event_type: event.event_type().to_string(),
                    payload,
                    created_at: Utc::now(),
                };

                if let Err(e) = repo.store(stored).await {
                    tracing::error!("failed to persist event: {}", e);
                }
            }
        });
    }
}

pub struct Application {
    object_service: ObjectService,
    event_bus: Arc<EventBus>,
    crypto_provider: Arc<dyn CryptoProvider>,
    event_repository: Arc<dyn EventRepository>,
}

impl Application {
    pub fn new(
        object_repository: Arc<dyn ObjectRepository>,
        event_bus: Arc<EventBus>,
        storage_provider: Arc<dyn StorageProvider>,
        crypto_provider: Arc<dyn CryptoProvider>,
        event_repository: Arc<dyn EventRepository>,
    ) -> Self {
        let object_service = ObjectService::new(
            object_repository,
            event_bus.clone(),
            storage_provider,
            crypto_provider.clone(),
        );
        Self {
            object_service,
            event_bus,
            crypto_provider,
            event_repository,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        info!("application started");

        let subscriber = Subscriber::new(self.event_bus.clone(), self.event_repository.clone());
        subscriber.start().await;

        Ok(())
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        info!("application shutting down");
        Ok(())
    }

    pub fn object_service(&self) -> &ObjectService {
        &self.object_service
    }

    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }

    pub fn crypto_provider(&self) -> &Arc<dyn CryptoProvider> {
        &self.crypto_provider
    }

    pub async fn list_objects(&self) -> anyhow::Result<Vec<StoredObject>> {
        self.object_service.list_objects().await
    }

    pub async fn get_stored_object(&self, id: &ObjectId) -> anyhow::Result<Option<StoredObject>> {
        self.object_service.get_stored_object(id).await
    }
}
