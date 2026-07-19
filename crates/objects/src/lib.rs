use chrono::{DateTime, Utc};
use common::ObjectId;
use serde::{Deserialize, Serialize};

pub trait Object {
    fn id(&self) -> &ObjectId;
    fn created_at(&self) -> DateTime<Utc>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileObject {
    pub id: ObjectId,
    pub name: String,
    pub mime_type: Option<String>,
    pub size: u64,
    pub created_at: DateTime<Utc>,
}

impl FileObject {
    pub fn new(name: String, size: u64, mime_type: Option<String>) -> Self {
        Self {
            id: ObjectId::new(),
            name,
            mime_type,
            size,
            created_at: common::now(),
        }
    }
}

impl Object for FileObject {
    fn id(&self) -> &ObjectId {
        &self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderObject {
    pub id: ObjectId,
    pub name: String,
    pub parent: Option<ObjectId>,
    pub created_at: DateTime<Utc>,
}

impl FolderObject {
    pub fn new(name: String, parent: Option<ObjectId>) -> Self {
        Self {
            id: ObjectId::new(),
            name,
            parent,
            created_at: common::now(),
        }
    }
}

impl Object for FolderObject {
    fn id(&self) -> &ObjectId {
        &self.id
    }

    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_object_creation() {
        let file = FileObject::new("test.txt".into(), 1024, Some("text/plain".into()));
        assert_eq!(file.name, "test.txt");
        assert_eq!(file.size, 1024);
        assert_eq!(file.mime_type.as_deref(), Some("text/plain"));
        assert!(!file.id().0.is_empty());
    }

    #[test]
    fn test_file_object_serialization() {
        let file = FileObject::new("test.txt".into(), 1024, Some("text/plain".into()));
        let json = serde_json::to_string(&file).unwrap();
        let deserialized: FileObject = serde_json::from_str(&json).unwrap();
        assert_eq!(file.id, deserialized.id);
        assert_eq!(file.name, deserialized.name);
    }

    #[test]
    fn test_folder_object_creation() {
        let folder = FolderObject::new("docs".into(), None);
        assert_eq!(folder.name, "docs");
        assert!(folder.parent.is_none());
        assert!(!folder.id().0.is_empty());
    }

    #[test]
    fn test_folder_object_with_parent() {
        let parent_id = ObjectId::new();
        let folder = FolderObject::new("subfolder".into(), Some(parent_id.clone()));
        assert_eq!(folder.parent.as_ref().unwrap(), &parent_id);
    }

    #[test]
    fn test_folder_object_serialization() {
        let folder = FolderObject::new("docs".into(), None);
        let json = serde_json::to_string(&folder).unwrap();
        let deserialized: FolderObject = serde_json::from_str(&json).unwrap();
        assert_eq!(folder.id, deserialized.id);
        assert_eq!(folder.name, deserialized.name);
    }
}
