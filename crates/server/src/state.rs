use std::sync::Arc;

pub struct AppState {
    pub application: Arc<dc_core::Application>,
    pub node_id: String,
}
