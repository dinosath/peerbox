use std::sync::Arc;

use common::ObjectId;
use database::{ObjectRepository, StoredObject};
use events::{Event, EventBus};
use objects::Object;
use chrono::Utc;
use storage::StorageProvider;
use tracing::info;

use serde::Serialize;

pub struct ObjectService {
    repository: Arc<dyn ObjectRepository>,
    event_bus: Arc<EventBus>,
    #[allow(dead_code)]
    storage: Arc<dyn StorageProvider>,
}

impl ObjectService {
    pub fn new(
        repository: Arc<dyn ObjectRepository>,
        event_bus: Arc<EventBus>,
        storage: Arc<dyn StorageProvider>,
    ) -> Self {
        Self {
            repository,
            event_bus,
            storage,
        }
    }

    pub async fn create_file(
        &self,
        file: objects::FileObject,
    ) -> anyhow::Result<ObjectId> {
        let id = file.id().clone();
        let data = serde_json::to_value(&file)?;

        let stored = StoredObject {
            id: id.0.clone(),
            object_type: "FileObject".to_string(),
            data,
            created_at: file.created_at(),
            updated_at: Utc::now(),
        };

        self.repository.create(stored).await?;

        self.event_bus
            .publish(Event::ObjectCreated { id: id.clone() })
            .await?;

        info!("object created id={}", id);
        Ok(id)
    }

    pub async fn create_folder(
        &self,
        folder: objects::FolderObject,
    ) -> anyhow::Result<ObjectId> {
        let id = folder.id().clone();
        let data = serde_json::to_value(&folder)?;

        let stored = StoredObject {
            id: id.0.clone(),
            object_type: "FolderObject".to_string(),
            data,
            created_at: folder.created_at(),
            updated_at: Utc::now(),
        };

        self.repository.create(stored).await?;

        self.event_bus
            .publish(Event::ObjectCreated { id: id.clone() })
            .await?;

        info!("object created id={}", id);
        Ok(id)
    }

    pub async fn get_object<T: Serialize + serde::de::DeserializeOwned>(
        &self,
        id: &ObjectId,
    ) -> anyhow::Result<Option<T>> {
        let stored = self.repository.get(id).await?;
        match stored {
            Some(s) => {
                let obj: T = serde_json::from_value(s.data)?;
                Ok(Some(obj))
            }
            None => Ok(None),
        }
    }

    pub async fn delete_object(&self, id: &ObjectId) -> anyhow::Result<()> {
        self.repository.delete(id).await?;

        self.event_bus
            .publish(Event::ObjectDeleted { id: id.clone() })
            .await?;

        info!("object deleted id={}", id);
        Ok(())
    }

    pub async fn update_object_data(
        &self,
        id: &ObjectId,
        data: serde_json::Value,
    ) -> anyhow::Result<()> {
        self.repository.update(id, data).await?;

        self.event_bus
            .publish(Event::ObjectUpdated { id: id.clone() })
            .await?;

        info!("object updated id={}", id);
        Ok(())
    }
}
