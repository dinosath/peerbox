use std::sync::Arc;

pub struct AppState {
    pub application: Arc<peerbox_core::Application>,
    pub node_id: String,
}
