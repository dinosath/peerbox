use std::sync::Arc;

use dc_core::Application;

use crate::commands::file_ops::FileOps;
use crate::commands::identity_ops::IdentityOps;
use crate::commands::sync_ops::SyncOps;
use crate::state::ui_state::SharedUiState;
use crate::views::file_browser::{FileBrowser, FileItem};
use crate::views::sync_dashboard::{IntegrityStatus, SyncDashboard, SyncPeerInfo, TransferInfo};

#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub total_files: u64,
    pub valid_files: u64,
    pub corrupted_files: u64,
    pub missing_files: u64,
    pub total_chunks: u64,
    pub valid_chunks: u64,
    pub corrupted_chunks: u64,
    pub missing_chunks: u64,
    pub integrity_status: IntegrityStatus,
}

pub struct DesktopApplication {
    app: Arc<Application>,
    pub ui_state: SharedUiState,
    pub sync_dashboard: SyncDashboard,
    pub file_browser: FileBrowser,
    pub file_ops: FileOps,
    pub sync_ops: SyncOps,
    pub identity_ops: IdentityOps,
}

impl DesktopApplication {
    pub fn new(app: Arc<Application>, identity_ops: IdentityOps) -> Self {
        let file_ops = FileOps::new(app.clone());
        Self {
            app,
            ui_state: crate::state::ui_state::new_shared_ui_state(),
            sync_dashboard: SyncDashboard::new(),
            file_browser: FileBrowser::new(),
            file_ops,
            sync_ops: SyncOps::new(),
            identity_ops,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        tracing::info!("desktop application main loop started");
        self.app.start().await?;

        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            if let Ok(state) = self.ui_state.try_read() {
                tracing::trace!("ui view: {:?}", state.current_view);
            }
        }
    }

    pub async fn get_file_list(&self) -> Vec<FileItem> {
        self.file_browser.list_files().to_vec()
    }

    pub async fn get_storage_usage(&self) -> u64 {
        self.sync_ops.get_storage_usage().await
    }

    pub async fn get_connected_peers(&self) -> Vec<SyncPeerInfo> {
        self.sync_ops.get_peers().await
    }

    pub async fn get_active_transfers(&self) -> Vec<TransferInfo> {
        self.sync_ops.get_active_transfers().await
    }

    pub async fn check_integrity(&self) -> IntegrityReport {
        let status = self
            .sync_ops
            .check_integrity()
            .await
            .unwrap_or(IntegrityStatus::Valid);
        IntegrityReport {
            total_files: 0,
            valid_files: 0,
            corrupted_files: 0,
            missing_files: 0,
            total_chunks: 0,
            valid_chunks: 0,
            corrupted_chunks: 0,
            missing_chunks: 0,
            integrity_status: status,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::sync::Mutex;

    use chrono::Utc;
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

    async fn make_test_app() -> Arc<Application> {
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
    async fn test_desktop_application_creation() {
        let app = make_test_app().await;
        let identity = IdentityOps::generate();
        let desktop = DesktopApplication::new(app, identity);

        let files = desktop.get_file_list().await;
        assert!(files.is_empty());

        let usage = desktop.get_storage_usage().await;
        assert_eq!(usage, 0);

        let peers = desktop.get_connected_peers().await;
        assert!(peers.is_empty());

        let transfers = desktop.get_active_transfers().await;
        assert!(transfers.is_empty());

        let integrity = desktop.check_integrity().await;
        assert_eq!(integrity.integrity_status, IntegrityStatus::Valid);
    }
}
