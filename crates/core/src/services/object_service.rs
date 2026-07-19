use std::sync::Arc;

use bytes::Bytes;
use chrono::Utc;
use common::{ContentHash, ObjectId};
use crypto::CryptoProvider;
use database::{ObjectRepository, StoredObject};
use events::{Event, EventBus};
use objects::Object;
use storage::StorageProvider;
use tracing::info;

use serde::Serialize;

pub struct ObjectService {
    repository: Arc<dyn ObjectRepository>,
    event_bus: Arc<EventBus>,
    storage: Arc<dyn StorageProvider>,
    crypto: Arc<dyn CryptoProvider>,
}

impl ObjectService {
    pub fn new(
        repository: Arc<dyn ObjectRepository>,
        event_bus: Arc<EventBus>,
        storage: Arc<dyn StorageProvider>,
        crypto: Arc<dyn CryptoProvider>,
    ) -> Self {
        Self {
            repository,
            event_bus,
            storage,
            crypto,
        }
    }

    pub async fn create_file(&self, file: objects::FileObject) -> anyhow::Result<ObjectId> {
        let id = file.id().clone();
        let file_bytes = serde_json::to_vec(&file)?;
        let data = serde_json::to_value(&file)?;

        let signature = self.crypto.sign(&file_bytes).await?;

        let mut stored_data = data;
        if let serde_json::Value::Object(ref mut map) = stored_data {
            map.insert(
                "signature".to_string(),
                serde_json::json!(hex::encode(&signature)),
            );
        }

        let stored = StoredObject {
            id: id.0.clone(),
            object_type: "FileObject".to_string(),
            data: stored_data,
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

    pub async fn create_file_with_data(
        &self,
        file: objects::FileObject,
        data: Bytes,
    ) -> anyhow::Result<ObjectId> {
        let id = file.id().clone();

        self.storage.put(&id, data.clone()).await?;

        let content_hash = ContentHash::new_blake3(&data);
        let file_bytes = serde_json::to_vec(&file)?;
        let signature = self.crypto.sign(&file_bytes).await?;

        let mut file_value = serde_json::to_value(&file)?;
        if let serde_json::Value::Object(ref mut map) = file_value {
            map.insert(
                "content_hash".to_string(),
                serde_json::to_value(&content_hash)?,
            );
            map.insert(
                "signature".to_string(),
                serde_json::json!(hex::encode(&signature)),
            );
        }

        let stored = StoredObject {
            id: id.0.clone(),
            object_type: "FileObject".to_string(),
            data: file_value,
            created_at: file.created_at(),
            updated_at: Utc::now(),
        };

        self.repository.create(stored).await?;

        self.event_bus
            .publish(Event::ObjectCreated { id: id.clone() })
            .await?;

        info!("object created with data id={}", id);
        Ok(id)
    }

    pub async fn get_file_data(&self, id: &ObjectId) -> anyhow::Result<Option<Bytes>> {
        self.storage.get(id).await
    }

    pub async fn create_folder(&self, folder: objects::FolderObject) -> anyhow::Result<ObjectId> {
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

    pub async fn list_objects(&self) -> anyhow::Result<Vec<database::StoredObject>> {
        self.repository.list().await
    }

    pub async fn get_stored_object(
        &self,
        id: &ObjectId,
    ) -> anyhow::Result<Option<database::StoredObject>> {
        self.repository.get(id).await
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
