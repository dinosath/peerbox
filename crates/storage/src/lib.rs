use bytes::Bytes;
use common::ObjectId;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::fs;
use tokio::sync::RwLock;

#[async_trait::async_trait]
pub trait StorageProvider: Send + Sync {
    async fn put(&self, id: &ObjectId, data: Bytes) -> anyhow::Result<()>;
    async fn get(&self, id: &ObjectId) -> anyhow::Result<Option<Bytes>>;
    async fn delete(&self, id: &ObjectId) -> anyhow::Result<()>;
}

pub struct MemoryStorageProvider {
    data: RwLock<HashMap<String, Bytes>>,
}

impl MemoryStorageProvider {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait::async_trait]
impl StorageProvider for MemoryStorageProvider {
    async fn put(&self, id: &ObjectId, data: Bytes) -> anyhow::Result<()> {
        self.data.write().await.insert(id.0.clone(), data);
        Ok(())
    }

    async fn get(&self, id: &ObjectId) -> anyhow::Result<Option<Bytes>> {
        Ok(self.data.read().await.get(&id.0).cloned())
    }

    async fn delete(&self, id: &ObjectId) -> anyhow::Result<()> {
        self.data.write().await.remove(&id.0);
        Ok(())
    }
}

impl Default for MemoryStorageProvider {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FileSystemStorageProvider {
    base_path: PathBuf,
}

impl FileSystemStorageProvider {
    pub fn new(path: PathBuf) -> Self {
        Self { base_path: path }
    }

    fn file_path(&self, id: &ObjectId) -> PathBuf {
        let filename = hex::encode(id.0.as_bytes());
        self.base_path.join(filename)
    }
}

#[async_trait::async_trait]
impl StorageProvider for FileSystemStorageProvider {
    async fn put(&self, id: &ObjectId, data: Bytes) -> anyhow::Result<()> {
        fs::create_dir_all(&self.base_path).await?;
        let path = self.file_path(id);
        fs::write(&path, &data).await?;
        Ok(())
    }

    async fn get(&self, id: &ObjectId) -> anyhow::Result<Option<Bytes>> {
        let path = self.file_path(id);
        match fs::read(&path).await {
            Ok(data) => Ok(Some(Bytes::from(data))),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn delete(&self, id: &ObjectId) -> anyhow::Result<()> {
        let path = self.file_path(id);
        match fs::remove_file(&path).await {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_storage_put_get() {
        let storage = MemoryStorageProvider::new();
        let id = ObjectId::new();
        let data = Bytes::from("hello world");

        storage.put(&id, data.clone()).await.unwrap();
        let result = storage.get(&id).await.unwrap();
        assert_eq!(result, Some(data));
    }

    #[tokio::test]
    async fn test_memory_storage_delete() {
        let storage = MemoryStorageProvider::new();
        let id = ObjectId::new();
        let data = Bytes::from("hello");

        storage.put(&id, data).await.unwrap();
        storage.delete(&id).await.unwrap();
        let result = storage.get(&id).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_memory_storage_get_nonexistent() {
        let storage = MemoryStorageProvider::new();
        let result = storage.get(&ObjectId::new()).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_filesystem_storage_put_get() {
        let dir = tempfile::tempdir().unwrap();
        let storage = FileSystemStorageProvider::new(dir.path().to_path_buf());
        let id = ObjectId::new();
        let data = Bytes::from("hello world");

        storage.put(&id, data.clone()).await.unwrap();
        let result = storage.get(&id).await.unwrap();
        assert_eq!(result, Some(data));
    }

    #[tokio::test]
    async fn test_filesystem_storage_delete() {
        let dir = tempfile::tempdir().unwrap();
        let storage = FileSystemStorageProvider::new(dir.path().to_path_buf());
        let id = ObjectId::new();
        let data = Bytes::from("hello");

        storage.put(&id, data).await.unwrap();
        storage.delete(&id).await.unwrap();
        let result = storage.get(&id).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_filesystem_storage_get_nonexistent() {
        let dir = tempfile::tempdir().unwrap();
        let storage = FileSystemStorageProvider::new(dir.path().to_path_buf());
        let result = storage.get(&ObjectId::new()).await.unwrap();
        assert_eq!(result, None);
    }

    #[tokio::test]
    async fn test_filesystem_storage_large_data() {
        let dir = tempfile::tempdir().unwrap();
        let storage = FileSystemStorageProvider::new(dir.path().to_path_buf());
        let id = ObjectId::new();
        let data = Bytes::from(vec![0x42u8; 4096]);

        storage.put(&id, data.clone()).await.unwrap();
        let result = storage.get(&id).await.unwrap();
        assert_eq!(result, Some(data));
    }
}
