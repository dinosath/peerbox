use chrono::{DateTime, Utc};
use common::NodeId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPeerInfo {
    pub node_id: NodeId,
    pub address: String,
    pub connected_since: DateTime<Utc>,
    pub transfer_stats: TransferStats,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TransferStats {
    pub uploaded_bytes: u64,
    pub downloaded_bytes: u64,
    pub upload_count: u64,
    pub download_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferInfo {
    pub peer: String,
    pub direction: TransferDirection,
    pub progress: f64,
    pub speed_bytes_per_sec: u64,
    pub file_name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransferDirection {
    Upload,
    Download,
}

impl TransferDirection {
    pub fn name(&self) -> &str {
        match self {
            TransferDirection::Upload => "Upload",
            TransferDirection::Download => "Download",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegrityStatus {
    Valid,
    Corrupted,
    Repairing,
}

impl IntegrityStatus {
    pub fn name(&self) -> &str {
        match self {
            IntegrityStatus::Valid => "Valid",
            IntegrityStatus::Corrupted => "Corrupted",
            IntegrityStatus::Repairing => "Repairing",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SyncDashboard {
    pub connected_peers: Vec<SyncPeerInfo>,
    pub active_transfers: Vec<TransferInfo>,
    pub storage_usage_bytes: u64,
    pub integrity_status: IntegrityStatus,
}

impl SyncDashboard {
    pub fn new() -> Self {
        Self {
            connected_peers: Vec::new(),
            active_transfers: Vec::new(),
            storage_usage_bytes: 0,
            integrity_status: IntegrityStatus::Valid,
        }
    }

    pub fn refresh(
        &mut self,
        peers: Vec<SyncPeerInfo>,
        transfers: Vec<TransferInfo>,
        storage_usage: u64,
        integrity: IntegrityStatus,
    ) {
        self.connected_peers = peers;
        self.active_transfers = transfers;
        self.storage_usage_bytes = storage_usage;
        self.integrity_status = integrity;
    }

    pub fn get_summary(&self) -> SyncDashboardSummary {
        let total_uploaded: u64 = self
            .connected_peers
            .iter()
            .map(|p| p.transfer_stats.uploaded_bytes)
            .sum();
        let total_downloaded: u64 = self
            .connected_peers
            .iter()
            .map(|p| p.transfer_stats.downloaded_bytes)
            .sum();
        SyncDashboardSummary {
            peer_count: self.connected_peers.len(),
            active_transfer_count: self.active_transfers.len(),
            total_uploaded,
            total_downloaded,
            storage_usage_bytes: self.storage_usage_bytes,
            integrity_status: self.integrity_status,
        }
    }
}

impl Default for SyncDashboard {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct SyncDashboardSummary {
    pub peer_count: usize,
    pub active_transfer_count: usize,
    pub total_uploaded: u64,
    pub total_downloaded: u64,
    pub storage_usage_bytes: u64,
    pub integrity_status: IntegrityStatus,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_peer(node_id: &str, uploaded: u64, downloaded: u64) -> SyncPeerInfo {
        SyncPeerInfo {
            node_id: NodeId::new(),
            address: format!("{}:8080", node_id),
            connected_since: Utc::now(),
            transfer_stats: TransferStats {
                uploaded_bytes: uploaded,
                downloaded_bytes: downloaded,
                upload_count: 5,
                download_count: 3,
            },
        }
    }

    #[test]
    fn test_sync_dashboard_summary() {
        let mut dash = SyncDashboard::new();
        dash.refresh(
            vec![make_peer("a", 1000, 500), make_peer("b", 2000, 1500)],
            vec![TransferInfo {
                peer: "node1".into(),
                direction: TransferDirection::Download,
                progress: 45.0,
                speed_bytes_per_sec: 1024,
                file_name: "file_a.txt".into(),
            }],
            4096,
            IntegrityStatus::Valid,
        );

        let summary = dash.get_summary();
        assert_eq!(summary.peer_count, 2);
        assert_eq!(summary.active_transfer_count, 1);
        assert_eq!(summary.total_uploaded, 3000);
        assert_eq!(summary.total_downloaded, 2000);
        assert_eq!(summary.storage_usage_bytes, 4096);
        assert_eq!(summary.integrity_status, IntegrityStatus::Valid);
    }

    #[test]
    fn test_sync_dashboard_empty() {
        let dash = SyncDashboard::new();
        let summary = dash.get_summary();
        assert_eq!(summary.peer_count, 0);
        assert_eq!(summary.active_transfer_count, 0);
        assert_eq!(summary.total_uploaded, 0);
        assert_eq!(summary.total_downloaded, 0);
        assert_eq!(summary.storage_usage_bytes, 0);
        assert_eq!(summary.integrity_status, IntegrityStatus::Valid);
    }

    #[test]
    fn test_sync_dashboard_refresh_updates_integrity() {
        let mut dash = SyncDashboard::new();
        assert_eq!(dash.integrity_status, IntegrityStatus::Valid);

        dash.refresh(vec![], vec![], 0, IntegrityStatus::Corrupted);
        assert_eq!(dash.integrity_status, IntegrityStatus::Corrupted);

        dash.refresh(vec![], vec![], 0, IntegrityStatus::Repairing);
        assert_eq!(dash.integrity_status, IntegrityStatus::Repairing);
    }

    #[test]
    fn test_integrity_status_names() {
        assert_eq!(IntegrityStatus::Valid.name(), "Valid");
        assert_eq!(IntegrityStatus::Corrupted.name(), "Corrupted");
        assert_eq!(IntegrityStatus::Repairing.name(), "Repairing");
    }

    #[test]
    fn test_transfer_direction_names() {
        assert_eq!(TransferDirection::Upload.name(), "Upload");
        assert_eq!(TransferDirection::Download.name(), "Download");
    }

    #[test]
    fn test_transfer_stats_default() {
        let stats = TransferStats::default();
        assert_eq!(stats.uploaded_bytes, 0);
        assert_eq!(stats.downloaded_bytes, 0);
        assert_eq!(stats.upload_count, 0);
        assert_eq!(stats.download_count, 0);
    }
}
