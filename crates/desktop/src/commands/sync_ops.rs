use common::NodeId;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::views::sync_dashboard::{
    IntegrityStatus, SyncDashboard, SyncPeerInfo, TransferDirection, TransferInfo,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncStatus {
    Idle,
    Syncing,
    Error,
    Connected,
}

impl SyncStatus {
    pub fn name(&self) -> &str {
        match self {
            SyncStatus::Idle => "Idle",
            SyncStatus::Syncing => "Syncing",
            SyncStatus::Error => "Error",
            SyncStatus::Connected => "Connected",
        }
    }
}

pub struct SyncOps {
    dashboard: Arc<RwLock<SyncDashboard>>,
    sync_status: Arc<RwLock<SyncStatus>>,
    sync_active: Arc<RwLock<bool>>,
}

impl SyncOps {
    pub fn new() -> Self {
        Self {
            dashboard: Arc::new(RwLock::new(SyncDashboard::new())),
            sync_status: Arc::new(RwLock::new(SyncStatus::Idle)),
            sync_active: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn get_peers(&self) -> Vec<SyncPeerInfo> {
        self.dashboard.read().await.connected_peers.clone()
    }

    pub async fn add_peer(&self, peer: SyncPeerInfo) {
        let mut dashboard = self.dashboard.write().await;
        dashboard.connected_peers.push(peer);
    }

    pub async fn remove_peer(&self, node_id: &NodeId) {
        let mut dashboard = self.dashboard.write().await;
        dashboard.connected_peers.retain(|p| p.node_id != *node_id);
    }

    pub async fn get_sync_status(&self) -> SyncStatus {
        *self.sync_status.read().await
    }

    pub async fn sync_with_peer(&self, node_id: &NodeId) -> anyhow::Result<()> {
        *self.sync_status.write().await = SyncStatus::Syncing;
        *self.sync_active.write().await = true;

        let dashboard = self.dashboard.read().await;
        let peer = dashboard
            .connected_peers
            .iter()
            .find(|p| p.node_id == *node_id);
        if peer.is_none() {
            *self.sync_status.write().await = SyncStatus::Error;
            *self.sync_active.write().await = false;
            return Err(anyhow::anyhow!("peer not found: {}", node_id));
        }

        drop(dashboard);

        let transfer = TransferInfo {
            peer: node_id.0.clone(),
            direction: TransferDirection::Download,
            progress: 0.0,
            speed_bytes_per_sec: 0,
            file_name: "sync_target".to_string(),
        };

        {
            let mut dashboard = self.dashboard.write().await;
            dashboard.active_transfers.push(transfer);
        }

        for p in [25u8, 50, 75, 100] {
            if !*self.sync_active.read().await {
                *self.sync_status.write().await = SyncStatus::Idle;
                return Err(anyhow::anyhow!("sync cancelled"));
            }
            {
                let mut dashboard = self.dashboard.write().await;
                if let Some(t) = dashboard.active_transfers.last_mut() {
                    t.progress = p as f64;
                    t.speed_bytes_per_sec = 1024 * 1024;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        {
            let mut dashboard = self.dashboard.write().await;
            if let Some(t) = dashboard.active_transfers.last_mut() {
                t.progress = 100.0;
            }
        }

        *self.sync_status.write().await = SyncStatus::Connected;
        *self.sync_active.write().await = false;
        Ok(())
    }

    pub async fn stop_sync(&self) {
        *self.sync_active.write().await = false;
        *self.sync_status.write().await = SyncStatus::Idle;
    }

    pub async fn get_dashboard_summary(
        &self,
    ) -> anyhow::Result<crate::views::sync_dashboard::SyncDashboardSummary> {
        Ok(self.dashboard.read().await.get_summary())
    }

    pub async fn get_storage_usage(&self) -> u64 {
        self.dashboard.read().await.storage_usage_bytes
    }

    pub async fn get_active_transfers(&self) -> Vec<TransferInfo> {
        self.dashboard.read().await.active_transfers.clone()
    }

    pub async fn check_integrity(&self) -> anyhow::Result<IntegrityStatus> {
        let status = self.dashboard.read().await.integrity_status;
        Ok(status)
    }

    pub async fn update_storage_usage(&self, bytes: u64) {
        self.dashboard.write().await.storage_usage_bytes = bytes;
    }

    pub async fn update_integrity(&self, status: IntegrityStatus) {
        self.dashboard.write().await.integrity_status = status;
    }
}

impl Default for SyncOps {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::sync_dashboard::TransferStats;
    use chrono::Utc;

    fn make_peer(id: &str) -> SyncPeerInfo {
        SyncPeerInfo {
            node_id: NodeId::new(),
            address: format!("{}:8080", id),
            connected_since: Utc::now(),
            transfer_stats: TransferStats::default(),
        }
    }

    #[tokio::test]
    async fn test_add_and_get_peers() {
        let ops = SyncOps::new();
        let peer = make_peer("node1");
        ops.add_peer(peer.clone()).await;

        let peers = ops.get_peers().await;
        assert_eq!(peers.len(), 1);
        assert_eq!(peers[0].node_id, peer.node_id);
    }

    #[tokio::test]
    async fn test_remove_peer() {
        let ops = SyncOps::new();
        let peer = make_peer("node1");
        let id = peer.node_id.clone();
        ops.add_peer(peer).await;
        assert_eq!(ops.get_peers().await.len(), 1);

        ops.remove_peer(&id).await;
        assert_eq!(ops.get_peers().await.len(), 0);
    }

    #[tokio::test]
    async fn test_sync_status_defaults() {
        let ops = SyncOps::new();
        assert_eq!(ops.get_sync_status().await, SyncStatus::Idle);
    }

    #[tokio::test]
    async fn test_sync_with_peer_updates_status() {
        let ops = SyncOps::new();
        let peer = make_peer("node1");
        let id = peer.node_id.clone();
        ops.add_peer(peer).await;

        let result = ops.sync_with_peer(&id).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_sync_with_unknown_peer_fails() {
        let ops = SyncOps::new();
        let result = ops.sync_with_peer(&NodeId::new()).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stop_sync() {
        let ops = SyncOps::new();
        ops.stop_sync().await;
        assert_eq!(ops.get_sync_status().await, SyncStatus::Idle);
    }

    #[tokio::test]
    async fn test_storage_usage() {
        let ops = SyncOps::new();
        assert_eq!(ops.get_storage_usage().await, 0);

        ops.update_storage_usage(12345).await;
        assert_eq!(ops.get_storage_usage().await, 12345);
    }

    #[tokio::test]
    async fn test_integrity_check() {
        let ops = SyncOps::new();
        let status = ops.check_integrity().await.unwrap();
        assert_eq!(status, IntegrityStatus::Valid);

        ops.update_integrity(IntegrityStatus::Corrupted).await;
        let status = ops.check_integrity().await.unwrap();
        assert_eq!(status, IntegrityStatus::Corrupted);
    }

    #[tokio::test]
    async fn test_dashboard_summary() {
        let ops = SyncOps::new();
        let summary = ops.get_dashboard_summary().await.unwrap();
        assert_eq!(summary.peer_count, 0);
        assert_eq!(summary.storage_usage_bytes, 0);
    }
}
