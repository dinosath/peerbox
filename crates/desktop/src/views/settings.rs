use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub node_name: String,
    pub data_directory: String,
    pub listen_port: u16,
    pub bootstrap_nodes: Vec<String>,
    pub federation_enabled: bool,
    pub log_level: String,
    pub auto_sync: bool,
    pub max_upload_speed_kbps: u64,
    pub max_download_speed_kbps: u64,
}

impl Default for UserSettings {
    fn default() -> Self {
        Self {
            node_name: "peerbox-desktop".to_string(),
            data_directory: "./peerbox-data".to_string(),
            listen_port: 0,
            bootstrap_nodes: Vec::new(),
            federation_enabled: false,
            log_level: "info".to_string(),
            auto_sync: true,
            max_upload_speed_kbps: 0,
            max_download_speed_kbps: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SettingsView {
    pub settings: UserSettings,
    pub modified: bool,
}

impl SettingsView {
    pub fn new() -> Self {
        Self {
            settings: UserSettings::default(),
            modified: false,
        }
    }

    pub fn display_settings(&self) -> &UserSettings {
        &self.settings
    }

    pub fn update_setting<K, V>(&mut self, key: K, value: V)
    where
        K: AsRef<str>,
        V: Into<String>,
    {
        let key = key.as_ref();
        let val = value.into();
        match key {
            "node_name" => self.settings.node_name = val,
            "data_directory" => self.settings.data_directory = val,
            "log_level" => self.settings.log_level = val,
            _ => {}
        }
        self.modified = true;
    }

    pub fn set_listen_port(&mut self, port: u16) {
        self.settings.listen_port = port;
        self.modified = true;
    }

    pub fn set_federation_enabled(&mut self, enabled: bool) {
        self.settings.federation_enabled = enabled;
        self.modified = true;
    }

    pub fn set_auto_sync(&mut self, enabled: bool) {
        self.settings.auto_sync = enabled;
        self.modified = true;
    }

    pub fn save_settings(&mut self) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(&self.settings)?;
        tracing::info!("settings saved: {}", json);
        self.modified = false;
        Ok(())
    }

    pub fn is_modified(&self) -> bool {
        self.modified
    }
}

impl Default for SettingsView {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_settings() {
        let view = SettingsView::new();
        let s = view.display_settings();
        assert_eq!(s.node_name, "peerbox-desktop");
        assert_eq!(s.listen_port, 0);
        assert_eq!(s.log_level, "info");
        assert!(s.auto_sync);
        assert!(!s.federation_enabled);
    }

    #[test]
    fn test_update_string_settings() {
        let mut view = SettingsView::new();
        view.update_setting("node_name", "my-node");
        view.update_setting("log_level", "debug");
        view.update_setting("data_directory", "/custom/path");

        assert_eq!(view.settings.node_name, "my-node");
        assert_eq!(view.settings.log_level, "debug");
        assert_eq!(view.settings.data_directory, "/custom/path");
        assert!(view.is_modified());
    }

    #[test]
    fn test_set_listen_port() {
        let mut view = SettingsView::new();
        view.set_listen_port(9090);
        assert_eq!(view.settings.listen_port, 9090);
        assert!(view.is_modified());
    }

    #[test]
    fn test_set_federation_enabled() {
        let mut view = SettingsView::new();
        assert!(!view.settings.federation_enabled);

        view.set_federation_enabled(true);
        assert!(view.settings.federation_enabled);
        assert!(view.is_modified());
    }

    #[test]
    fn test_set_auto_sync() {
        let mut view = SettingsView::new();
        assert!(view.settings.auto_sync);

        view.set_auto_sync(false);
        assert!(!view.settings.auto_sync);
        assert!(view.is_modified());
    }

    #[test]
    fn test_save_settings_resets_modified() {
        let mut view = SettingsView::new();
        view.set_listen_port(8080);
        assert!(view.is_modified());

        view.save_settings().unwrap();
        assert!(!view.is_modified());
    }

    #[test]
    fn test_unknown_setting_key_does_not_panic() {
        let mut view = SettingsView::new();
        view.update_setting("unknown_key", "value");
        assert!(view.is_modified());
    }

    #[test]
    fn test_settings_serialization() {
        let settings = UserSettings {
            node_name: "test".into(),
            data_directory: "/data".into(),
            listen_port: 9000,
            bootstrap_nodes: vec!["peer1:8080".into()],
            federation_enabled: true,
            log_level: "trace".into(),
            auto_sync: false,
            max_upload_speed_kbps: 1024,
            max_download_speed_kbps: 2048,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: UserSettings = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.node_name, "test");
        assert_eq!(deserialized.listen_port, 9000);
        assert_eq!(deserialized.max_upload_speed_kbps, 1024);
    }
}
