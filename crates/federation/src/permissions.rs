use std::collections::HashMap;

use common::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Permission {
    Read,
    Write,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionEntry {
    pub actor_uri: String,
    pub permission: Permission,
    pub granted_at: Timestamp,
}

#[derive(Debug, Clone, Default)]
pub struct PermissionManager {
    entries: HashMap<String, Vec<PermissionEntry>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant(&mut self, object_id: &str, actor_uri: &str, permission: Permission) {
        let entry = PermissionEntry {
            actor_uri: actor_uri.to_string(),
            permission,
            granted_at: common::now(),
        };
        self.entries
            .entry(object_id.to_string())
            .or_default()
            .push(entry);
    }

    pub fn revoke(&mut self, object_id: &str, actor_uri: &str) {
        if let Some(entries) = self.entries.get_mut(object_id) {
            entries.retain(|e| e.actor_uri != actor_uri);
        }
    }

    pub fn check(&self, object_id: &str, actor_uri: &str, required: Permission) -> bool {
        if let Some(entries) = self.entries.get(object_id) {
            entries
                .iter()
                .any(|e| e.actor_uri == actor_uri && e.permission >= required)
        } else {
            false
        }
    }

    pub fn get_permissions(&self, object_id: &str) -> Vec<&PermissionEntry> {
        self.entries
            .get(object_id)
            .map(|v| v.iter().collect())
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grant_and_check() {
        let mut pm = PermissionManager::new();
        pm.grant(
            "obj-1",
            "https://node.example.com/users/alice",
            Permission::Read,
        );
        assert!(pm.check(
            "obj-1",
            "https://node.example.com/users/alice",
            Permission::Read
        ));
    }

    #[test]
    fn test_revoke() {
        let mut pm = PermissionManager::new();
        pm.grant(
            "obj-1",
            "https://node.example.com/users/alice",
            Permission::Read,
        );
        assert!(pm.check(
            "obj-1",
            "https://node.example.com/users/alice",
            Permission::Read
        ));

        pm.revoke("obj-1", "https://node.example.com/users/alice");
        assert!(!pm.check(
            "obj-1",
            "https://node.example.com/users/alice",
            Permission::Read
        ));
    }

    #[test]
    fn test_admin_implies_read_and_write() {
        let mut pm = PermissionManager::new();
        pm.grant(
            "obj-1",
            "https://node.example.com/users/admin",
            Permission::Admin,
        );
        assert!(pm.check(
            "obj-1",
            "https://node.example.com/users/admin",
            Permission::Read
        ));
        assert!(pm.check(
            "obj-1",
            "https://node.example.com/users/admin",
            Permission::Write
        ));
        assert!(pm.check(
            "obj-1",
            "https://node.example.com/users/admin",
            Permission::Admin
        ));
    }

    #[test]
    fn test_write_does_not_imply_admin() {
        let mut pm = PermissionManager::new();
        pm.grant(
            "obj-1",
            "https://node.example.com/users/writer",
            Permission::Write,
        );
        assert!(pm.check(
            "obj-1",
            "https://node.example.com/users/writer",
            Permission::Write
        ));
        assert!(!pm.check(
            "obj-1",
            "https://node.example.com/users/writer",
            Permission::Admin
        ));
    }

    #[test]
    fn test_different_objects_isolated() {
        let mut pm = PermissionManager::new();
        pm.grant(
            "obj-1",
            "https://node.example.com/users/alice",
            Permission::Read,
        );
        assert!(pm.check(
            "obj-1",
            "https://node.example.com/users/alice",
            Permission::Read
        ));
        assert!(!pm.check(
            "obj-2",
            "https://node.example.com/users/alice",
            Permission::Read
        ));
    }

    #[test]
    fn test_get_permissions() {
        let mut pm = PermissionManager::new();
        pm.grant(
            "obj-1",
            "https://node.example.com/users/alice",
            Permission::Read,
        );
        pm.grant(
            "obj-1",
            "https://node.example.com/users/bob",
            Permission::Write,
        );

        let perms = pm.get_permissions("obj-1");
        assert_eq!(perms.len(), 2);
        let actors: Vec<&str> = perms.iter().map(|e| e.actor_uri.as_str()).collect();
        assert!(actors.contains(&"https://node.example.com/users/alice"));
        assert!(actors.contains(&"https://node.example.com/users/bob"));
    }

    #[test]
    fn test_get_permissions_empty() {
        let pm = PermissionManager::new();
        let perms = pm.get_permissions("nonexistent");
        assert!(perms.is_empty());
    }

    #[test]
    fn test_permission_ordering() {
        assert!(Permission::Admin > Permission::Write);
        assert!(Permission::Write > Permission::Read);
        assert!(Permission::Admin > Permission::Read);
    }
}
