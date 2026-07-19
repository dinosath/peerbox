use chrono::{DateTime, Utc};
use common::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub name: String,
    pub size: u64,
    pub created_at: DateTime<Utc>,
    pub object_id: ObjectId,
    pub mime_type: Option<String>,
    pub hash: Option<String>,
    pub is_folder: bool,
    pub parent_id: Option<ObjectId>,
}

#[derive(Debug, Clone)]
pub struct FileBrowser {
    pub files: Vec<FileItem>,
    pub current_parent: Option<ObjectId>,
}

impl FileBrowser {
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            current_parent: None,
        }
    }

    pub fn list_files(&self) -> &[FileItem] {
        &self.files
    }

    pub fn set_files(&mut self, files: Vec<FileItem>) {
        self.files = files;
    }

    pub fn filter_by_type(&self, mime_type: &str) -> Vec<&FileItem> {
        self.files
            .iter()
            .filter(|f| f.mime_type.as_deref() == Some(mime_type))
            .collect()
    }

    pub fn sort_by_name(&mut self, ascending: bool) {
        if ascending {
            self.files.sort_by(|a, b| a.name.cmp(&b.name));
        } else {
            self.files.sort_by(|a, b| b.name.cmp(&a.name));
        }
    }

    pub fn sort_by_date(&mut self, ascending: bool) {
        if ascending {
            self.files.sort_by_key(|a| a.created_at);
        } else {
            self.files.sort_by_key(|b| std::cmp::Reverse(b.created_at));
        }
    }

    pub fn sort_by_size(&mut self, ascending: bool) {
        if ascending {
            self.files.sort_by_key(|a| a.size);
        } else {
            self.files.sort_by_key(|b| std::cmp::Reverse(b.size));
        }
    }

    pub fn search(&self, query: &str) -> Vec<&FileItem> {
        let query = query.to_lowercase();
        self.files
            .iter()
            .filter(|f| f.name.to_lowercase().contains(&query))
            .collect()
    }
}

impl Default for FileBrowser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_item(name: &str, size: u64, is_folder: bool) -> FileItem {
        FileItem {
            name: name.to_string(),
            size,
            created_at: Utc::now(),
            object_id: ObjectId::new(),
            mime_type: if is_folder {
                None
            } else {
                Some("text/plain".to_string())
            },
            hash: None,
            is_folder,
            parent_id: None,
        }
    }

    #[test]
    fn test_file_browser_listing() {
        let mut browser = FileBrowser::new();
        let items = vec![
            make_item("alpha.txt", 100, false),
            make_item("beta.txt", 200, false),
        ];
        browser.set_files(items);
        assert_eq!(browser.list_files().len(), 2);
    }

    #[test]
    fn test_file_browser_sort_by_name() {
        let mut browser = FileBrowser::new();
        browser.set_files(vec![
            make_item("c.txt", 100, false),
            make_item("a.txt", 200, false),
            make_item("b.txt", 150, false),
        ]);

        browser.sort_by_name(true);
        let names: Vec<&str> = browser
            .list_files()
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, vec!["a.txt", "b.txt", "c.txt"]);

        browser.sort_by_name(false);
        let names: Vec<&str> = browser
            .list_files()
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, vec!["c.txt", "b.txt", "a.txt"]);
    }

    #[test]
    fn test_file_browser_sort_by_size() {
        let mut browser = FileBrowser::new();
        browser.set_files(vec![
            make_item("small.txt", 10, false),
            make_item("large.txt", 300, false),
            make_item("medium.txt", 150, false),
        ]);

        browser.sort_by_size(true);
        let sizes: Vec<u64> = browser.list_files().iter().map(|f| f.size).collect();
        assert_eq!(sizes, vec![10, 150, 300]);

        browser.sort_by_size(false);
        let sizes: Vec<u64> = browser.list_files().iter().map(|f| f.size).collect();
        assert_eq!(sizes, vec![300, 150, 10]);
    }

    #[test]
    fn test_file_browser_filter_by_type() {
        let browser = FileBrowser {
            files: vec![
                FileItem {
                    name: "doc.txt".into(),
                    size: 100,
                    created_at: Utc::now(),
                    object_id: ObjectId::new(),
                    mime_type: Some("text/plain".into()),
                    hash: None,
                    is_folder: false,
                    parent_id: None,
                },
                FileItem {
                    name: "image.png".into(),
                    size: 500,
                    created_at: Utc::now(),
                    object_id: ObjectId::new(),
                    mime_type: Some("image/png".into()),
                    hash: None,
                    is_folder: false,
                    parent_id: None,
                },
            ],
            current_parent: None,
        };

        let text_files = browser.filter_by_type("text/plain");
        assert_eq!(text_files.len(), 1);
        assert_eq!(text_files[0].name, "doc.txt");

        let image_files = browser.filter_by_type("image/png");
        assert_eq!(image_files.len(), 1);
        assert_eq!(image_files[0].name, "image.png");
    }

    #[test]
    fn test_file_browser_search() {
        let browser = FileBrowser {
            files: vec![
                make_item("hello.txt", 100, false),
                make_item("world.txt", 200, false),
                make_item("HelloWorld.md", 300, false),
            ],
            current_parent: None,
        };

        let results = browser.search("hello");
        assert_eq!(results.len(), 2);

        let results = browser.search("world");
        assert_eq!(results.len(), 2);

        let results = browser.search("xyz");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_file_browser_folder_hierarchy() {
        let parent_id = ObjectId::new();
        let items = vec![
            make_item("folder_a", 0, true),
            FileItem {
                name: "child.txt".into(),
                size: 42,
                created_at: Utc::now(),
                object_id: ObjectId::new(),
                mime_type: Some("text/plain".into()),
                hash: None,
                is_folder: false,
                parent_id: Some(parent_id.clone()),
            },
        ];
        let mut browser = FileBrowser::new();
        browser.set_files(items);
        assert_eq!(browser.list_files().len(), 2);
        assert!(browser.list_files()[1].parent_id.is_some());
    }
}
