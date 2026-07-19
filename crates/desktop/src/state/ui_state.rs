use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    FileBrowser,
    Upload,
    SyncDashboard,
    Settings,
}

impl View {
    pub fn name(&self) -> &str {
        match self {
            View::FileBrowser => "FileBrowser",
            View::Upload => "Upload",
            View::SyncDashboard => "SyncDashboard",
            View::Settings => "Settings",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortOrder {
    #[default]
    NameAsc,
    NameDesc,
    DateAsc,
    DateDesc,
    SizeAsc,
    SizeDesc,
}

#[derive(Debug, Clone)]
pub struct UiState {
    pub current_view: View,
    pub selected_files: Vec<String>,
    pub search_query: String,
    pub sort_order: SortOrder,
    pub is_loading: bool,
    pub status_message: Option<String>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            current_view: View::FileBrowser,
            selected_files: Vec::new(),
            search_query: String::new(),
            sort_order: SortOrder::default(),
            is_loading: false,
            status_message: None,
        }
    }
}

pub type SharedUiState = Arc<RwLock<UiState>>;

pub fn new_shared_ui_state() -> SharedUiState {
    Arc::new(RwLock::new(UiState::default()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ui_state_defaults() {
        let state = UiState::default();
        assert_eq!(state.current_view, View::FileBrowser);
        assert!(state.selected_files.is_empty());
        assert!(state.search_query.is_empty());
        assert_eq!(state.sort_order, SortOrder::NameAsc);
        assert!(!state.is_loading);
        assert!(state.status_message.is_none());
    }

    #[tokio::test]
    async fn test_shared_ui_state_transitions() {
        let ui = new_shared_ui_state();

        {
            let mut state = ui.write().await;
            state.current_view = View::Upload;
            state.search_query = "test.txt".to_string();
            state.is_loading = true;
        }

        {
            let state = ui.read().await;
            assert_eq!(state.current_view, View::Upload);
            assert_eq!(state.search_query, "test.txt");
            assert!(state.is_loading);
        }

        {
            let mut state = ui.write().await;
            state.current_view = View::SyncDashboard;
            state.is_loading = false;
            state.status_message = Some("sync complete".to_string());
        }

        {
            let state = ui.read().await;
            assert_eq!(state.current_view, View::SyncDashboard);
            assert!(!state.is_loading);
            assert_eq!(state.status_message.as_deref(), Some("sync complete"));
        }
    }

    #[test]
    fn test_view_names() {
        assert_eq!(View::FileBrowser.name(), "FileBrowser");
        assert_eq!(View::Upload.name(), "Upload");
        assert_eq!(View::SyncDashboard.name(), "SyncDashboard");
        assert_eq!(View::Settings.name(), "Settings");
    }

    #[test]
    fn test_sort_order_default() {
        assert_eq!(SortOrder::default(), SortOrder::NameAsc);
    }
}
