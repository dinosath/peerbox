use common::ObjectId;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UploadStage {
    Idle,
    Reading,
    Chunking,
    Hashing,
    CreatingManifest,
    Storing,
    Complete,
    Cancelled,
    Error,
}

impl UploadStage {
    pub fn name(&self) -> &str {
        match self {
            UploadStage::Idle => "Idle",
            UploadStage::Reading => "Reading",
            UploadStage::Chunking => "Chunking",
            UploadStage::Hashing => "Hashing",
            UploadStage::CreatingManifest => "CreatingManifest",
            UploadStage::Storing => "Storing",
            UploadStage::Complete => "Complete",
            UploadStage::Cancelled => "Cancelled",
            UploadStage::Error => "Error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UploadProgress {
    pub stage: UploadStage,
    pub percent: f64,
    pub bytes_processed: u64,
    pub total_bytes: u64,
    pub file_name: String,
    pub result_object_id: Option<ObjectId>,
}

impl UploadProgress {
    pub fn for_file(file_name: String, total_bytes: u64) -> Self {
        Self {
            stage: UploadStage::Idle,
            percent: 0.0,
            bytes_processed: 0,
            total_bytes,
            file_name,
            result_object_id: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct UploadFlow {
    progress: Arc<RwLock<UploadProgress>>,
    cancelled: Arc<RwLock<bool>>,
}

impl UploadFlow {
    pub fn new(file_name: String, total_bytes: u64) -> Self {
        Self {
            progress: Arc::new(RwLock::new(UploadProgress::for_file(
                file_name,
                total_bytes,
            ))),
            cancelled: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn get_progress(&self) -> UploadProgress {
        self.progress.read().await.clone()
    }

    pub async fn set_stage(&self, stage: UploadStage) {
        self.progress.write().await.stage = stage;
    }

    pub async fn update_progress(&self, bytes_processed: u64) {
        let mut p = self.progress.write().await;
        p.bytes_processed = bytes_processed;
        if p.total_bytes > 0 {
            p.percent = (bytes_processed as f64 / p.total_bytes as f64) * 100.0;
        }
    }

    pub async fn complete(&self, object_id: ObjectId) {
        let mut p = self.progress.write().await;
        p.stage = UploadStage::Complete;
        p.percent = 100.0;
        p.bytes_processed = p.total_bytes;
        p.result_object_id = Some(object_id);
    }

    pub async fn cancel(&self) {
        *self.cancelled.write().await = true;
        self.progress.write().await.stage = UploadStage::Cancelled;
    }

    pub async fn is_cancelled(&self) -> bool {
        *self.cancelled.read().await
    }

    pub fn file_name(&self) -> &str {
        "UploadFlow"
    }

    pub async fn start_upload(&self, _file_path: &str) -> anyhow::Result<ObjectId> {
        self.set_stage(UploadStage::Reading).await;
        if self.is_cancelled().await {
            return Err(anyhow::anyhow!("upload cancelled"));
        }

        self.set_stage(UploadStage::Chunking).await;
        if self.is_cancelled().await {
            return Err(anyhow::anyhow!("upload cancelled"));
        }

        self.set_stage(UploadStage::Hashing).await;
        if self.is_cancelled().await {
            return Err(anyhow::anyhow!("upload cancelled"));
        }

        self.set_stage(UploadStage::CreatingManifest).await;
        if self.is_cancelled().await {
            return Err(anyhow::anyhow!("upload cancelled"));
        }

        self.set_stage(UploadStage::Storing).await;
        if self.is_cancelled().await {
            return Err(anyhow::anyhow!("upload cancelled"));
        }

        let id = ObjectId::new();
        self.complete(id.clone()).await;
        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_upload_flow_stages() {
        let flow = UploadFlow::new("test.txt".to_string(), 1024);
        assert_eq!(flow.get_progress().await.stage, UploadStage::Idle);
        assert_eq!(flow.get_progress().await.percent, 0.0);

        flow.set_stage(UploadStage::Reading).await;
        assert_eq!(flow.get_progress().await.stage, UploadStage::Reading);

        flow.update_progress(512).await;
        let p = flow.get_progress().await;
        assert_eq!(p.bytes_processed, 512);
        assert_eq!(p.percent, 50.0);

        flow.update_progress(1024).await;
        let p = flow.get_progress().await;
        assert_eq!(p.bytes_processed, 1024);
        assert_eq!(p.percent, 100.0);

        let id = ObjectId::new();
        flow.complete(id.clone()).await;
        let p = flow.get_progress().await;
        assert_eq!(p.stage, UploadStage::Complete);
        assert_eq!(p.result_object_id, Some(id));
    }

    #[tokio::test]
    async fn test_upload_flow_cancel() {
        let flow = UploadFlow::new("test.txt".to_string(), 1024);
        flow.cancel().await;
        assert!(flow.is_cancelled().await);
        assert_eq!(flow.get_progress().await.stage, UploadStage::Cancelled);
    }

    #[tokio::test]
    async fn test_upload_flow_start_upload() {
        let flow = UploadFlow::new("test.txt".to_string(), 1024);
        let result = flow.start_upload("test.txt").await;
        assert!(result.is_ok());
        let p = flow.get_progress().await;
        assert_eq!(p.stage, UploadStage::Complete);
        assert_eq!(p.percent, 100.0);
        assert!(p.result_object_id.is_some());
    }

    #[tokio::test]
    async fn test_upload_flow_cancel_during_upload() {
        let flow = UploadFlow::new("test.txt".to_string(), 1024);
        flow.cancel().await;
        let result = flow.start_upload("test.txt").await;
        assert!(result.is_err());
    }

    #[test]
    fn test_upload_stage_names() {
        assert_eq!(UploadStage::Idle.name(), "Idle");
        assert_eq!(UploadStage::Reading.name(), "Reading");
        assert_eq!(UploadStage::Chunking.name(), "Chunking");
        assert_eq!(UploadStage::Hashing.name(), "Hashing");
        assert_eq!(UploadStage::CreatingManifest.name(), "CreatingManifest");
        assert_eq!(UploadStage::Storing.name(), "Storing");
        assert_eq!(UploadStage::Complete.name(), "Complete");
        assert_eq!(UploadStage::Cancelled.name(), "Cancelled");
        assert_eq!(UploadStage::Error.name(), "Error");
    }

    #[test]
    fn test_upload_progress_for_file() {
        let p = UploadProgress::for_file("data.bin".to_string(), 2048);
        assert_eq!(p.file_name, "data.bin");
        assert_eq!(p.total_bytes, 2048);
        assert_eq!(p.stage, UploadStage::Idle);
        assert_eq!(p.percent, 0.0);
        assert_eq!(p.bytes_processed, 0);
    }
}
