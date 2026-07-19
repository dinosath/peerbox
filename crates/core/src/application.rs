use std::sync::Arc;

use database::ObjectRepository;
use events::EventBus;
use storage::StorageProvider;
use tracing::info;

use super::ObjectService;

pub struct Application {
    object_service: ObjectService,
    event_bus: Arc<EventBus>,
}

impl Application {
    pub fn new(
        object_repository: Arc<dyn ObjectRepository>,
        event_bus: Arc<EventBus>,
        storage_provider: Arc<dyn StorageProvider>,
    ) -> Self {
        let object_service = ObjectService::new(object_repository, event_bus.clone(), storage_provider);
        Self {
            object_service,
            event_bus,
        }
    }

    pub async fn start(&self) -> anyhow::Result<()> {
        info!("application started");
        Ok(())
    }

    pub async fn shutdown(&self) -> anyhow::Result<()> {
        info!("application shutdown");
        Ok(())
    }

    pub fn object_service(&self) -> &ObjectService {
        &self.object_service
    }

    pub fn event_bus(&self) -> &Arc<EventBus> {
        &self.event_bus
    }
}
