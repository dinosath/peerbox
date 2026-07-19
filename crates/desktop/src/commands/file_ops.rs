use std::sync::Arc;

use bytes::Bytes;
use common::ObjectId;
use database::StoredObject;
use dc_core::Application;
use objects::{FileObject, FolderObject};

pub struct FileOps {
    app: Arc<Application>,
}

impl FileOps {
    pub fn new(app: Arc<Application>) -> Self {
        Self { app }
    }

    pub async fn upload_file(
        &self,
        name: String,
        data: Bytes,
        mime_type: Option<String>,
    ) -> anyhow::Result<ObjectId> {
        let size = data.len() as u64;
        let file = FileObject::new(name, size, mime_type);
        self.app
            .object_service()
            .create_file_with_data(file, data)
            .await
    }

    pub async fn download_file(&self, id: &ObjectId) -> anyhow::Result<Vec<u8>> {
        if self.app.get_stored_object(id).await?.is_none() {
            return Err(anyhow::anyhow!("file not found: {}", id));
        }
        match self.app.object_service().get_file_data(id).await? {
            Some(data) => Ok(data.to_vec()),
            None => Err(anyhow::anyhow!("file data not found: {}", id)),
        }
    }

    pub async fn delete_file(&self, id: &ObjectId) -> anyhow::Result<()> {
        self.app.object_service().delete_object(id).await
    }

    pub async fn get_file_metadata(&self, id: &ObjectId) -> anyhow::Result<Option<StoredObject>> {
        self.app.get_stored_object(id).await
    }

    pub async fn list_files(&self) -> anyhow::Result<Vec<StoredObject>> {
        self.app.list_objects().await
    }

    pub async fn create_folder(
        &self,
        name: String,
        parent: Option<ObjectId>,
    ) -> anyhow::Result<ObjectId> {
        let folder = FolderObject::new(name, parent);
        self.app.object_service().create_folder(folder).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    use common::ObjectId;
    use crypto::{CryptoProvider, DefaultCryptoProvider};
    use database::{EventRepository, ObjectRepository, StoredEvent, StoredObject};
    use events::EventBus;
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

    async fn make_app() -> Arc<Application> {
        let obj_repo: Arc<dyn ObjectRepository> = Arc::new(MemoryObjectRepository::new());
        let event_bus = Arc::new(EventBus::new(256));
        let storage: Arc<dyn StorageProvider> = Arc::new(MemoryStorageProvider::new());
        let crypto: Arc<dyn CryptoProvider> = Arc::new(DefaultCryptoProvider::new());
        let event_repo: Arc<dyn EventRepository> = Arc::new(MemoryEventRepository::new());
        Arc::new(Application::new(
            obj_repo, event_bus, storage, crypto, event_repo,
        ))
    }

    #[tokio::test]
    async fn test_upload_file() {
        let app = make_app().await;
        let ops = FileOps::new(app);
        let data = Bytes::from("hello peerbox");
        let id = ops
            .upload_file("test.txt".into(), data.clone(), Some("text/plain".into()))
            .await
            .unwrap();
        assert!(!id.0.is_empty());
    }

    #[tokio::test]
    async fn test_upload_and_download_cycle() {
        let app = make_app().await;
        let ops = FileOps::new(app);
        let data = Bytes::from("upload download test data");

        let id = ops
            .upload_file(
                "cycle_test.bin".into(),
                data.clone(),
                Some("application/octet-stream".into()),
            )
            .await
            .unwrap();

        let downloaded = ops.download_file(&id).await.unwrap();
        assert_eq!(downloaded, data.to_vec());
    }

    #[tokio::test]
    async fn test_delete_file() {
        let app = make_app().await;
        let ops = FileOps::new(app);
        let data = Bytes::from("delete me");
        let id = ops
            .upload_file("delete.txt".into(), data, None)
            .await
            .unwrap();

        ops.delete_file(&id).await.unwrap();

        let result = ops.download_file(&id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_file_metadata() {
        let app = make_app().await;
        let ops = FileOps::new(app);
        let data = Bytes::from("metadata test");
        let id = ops
            .upload_file("meta.txt".into(), data, Some("text/plain".into()))
            .await
            .unwrap();

        let meta = ops.get_file_metadata(&id).await.unwrap();
        assert!(meta.is_some());
        assert_eq!(meta.unwrap().object_type, "FileObject");
    }

    #[tokio::test]
    async fn test_list_files() {
        let app = make_app().await;
        let ops = FileOps::new(app);
        ops.upload_file("a.txt".into(), Bytes::from("a"), None)
            .await
            .unwrap();
        ops.upload_file("b.txt".into(), Bytes::from("b"), None)
            .await
            .unwrap();

        let files = ops.list_files().await.unwrap();
        assert_eq!(files.len(), 2);
    }

    #[tokio::test]
    async fn test_create_folder() {
        let app = make_app().await;
        let ops = FileOps::new(app);
        let id = ops.create_folder("documents".into(), None).await.unwrap();
        assert!(!id.0.is_empty());
    }

    #[tokio::test]
    async fn test_create_folder_with_parent() {
        let app = make_app().await;
        let ops = FileOps::new(app);
        let parent_id = ops.create_folder("root".into(), None).await.unwrap();
        let child_id = ops
            .create_folder("sub".into(), Some(parent_id.clone()))
            .await
            .unwrap();
        assert!(!child_id.0.is_empty());
    }

    #[tokio::test]
    async fn test_download_nonexistent_file() {
        let app = make_app().await;
        let ops = FileOps::new(app);
        let result = ops.download_file(&ObjectId::new()).await;
        assert!(result.is_err());
    }
}
