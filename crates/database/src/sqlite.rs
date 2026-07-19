use async_trait::async_trait;
use chrono::Utc;
use common::ObjectId;
use sqlx::sqlite::SqlitePool;

use super::{EventRepository, ObjectRepository, StoredEvent, StoredObject};

pub struct SqliteObjectRepository {
    pool: SqlitePool,
}

impl SqliteObjectRepository {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        let repo = Self { pool };
        repo.run_migrations().await?;
        Ok(repo)
    }

    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS objects (
                id TEXT PRIMARY KEY,
                object_type TEXT NOT NULL,
                data JSON NOT NULL,
                created_at TIMESTAMP NOT NULL,
                updated_at TIMESTAMP NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                payload JSON NOT NULL,
                created_at TIMESTAMP NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl ObjectRepository for SqliteObjectRepository {
    async fn create(&self, object: StoredObject) -> anyhow::Result<()> {
        let data_json = serde_json::to_string(&object.data)?;

        sqlx::query(
            "INSERT INTO objects (id, object_type, data, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&object.id)
        .bind(&object.object_type)
        .bind(&data_json)
        .bind(object.created_at)
        .bind(object.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get(&self, id: &ObjectId) -> anyhow::Result<Option<StoredObject>> {
        let row = sqlx::query_as::<_, ObjectRow>(
            "SELECT id, object_type, data, created_at, updated_at FROM objects WHERE id = ?",
        )
        .bind(&id.0)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| r.into_stored_object()))
    }

    async fn list(&self) -> anyhow::Result<Vec<StoredObject>> {
        let rows = sqlx::query_as::<_, ObjectRow>(
            "SELECT id, object_type, data, created_at, updated_at FROM objects ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_stored_object()).collect())
    }

    async fn update(&self, id: &ObjectId, data: serde_json::Value) -> anyhow::Result<()> {
        let data_json = serde_json::to_string(&data)?;
        let now = Utc::now();

        sqlx::query("UPDATE objects SET data = ?, updated_at = ? WHERE id = ?")
            .bind(&data_json)
            .bind(now)
            .bind(&id.0)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn delete(&self, id: &ObjectId) -> anyhow::Result<()> {
        sqlx::query("DELETE FROM objects WHERE id = ?")
            .bind(&id.0)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}

pub struct SqliteEventRepository {
    pool: SqlitePool,
}

impl SqliteEventRepository {
    pub async fn new(database_url: &str) -> anyhow::Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        let repo = Self { pool };
        repo.run_migrations().await?;
        Ok(repo)
    }

    pub fn from_pool(pool: SqlitePool) -> Self {
        Self { pool }
    }

    async fn run_migrations(&self) -> anyhow::Result<()> {
        sqlx::query(
            "CREATE TABLE IF NOT EXISTS events (
                id TEXT PRIMARY KEY,
                event_type TEXT NOT NULL,
                payload JSON NOT NULL,
                created_at TIMESTAMP NOT NULL
            )",
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[async_trait]
impl EventRepository for SqliteEventRepository {
    async fn store(&self, event: StoredEvent) -> anyhow::Result<()> {
        let payload_json = serde_json::to_string(&event.payload)?;

        sqlx::query("INSERT INTO events (id, event_type, payload, created_at) VALUES (?, ?, ?, ?)")
            .bind(&event.id)
            .bind(&event.event_type)
            .bind(&payload_json)
            .bind(event.created_at)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn list(&self) -> anyhow::Result<Vec<StoredEvent>> {
        let rows = sqlx::query_as::<_, EventRow>(
            "SELECT id, event_type, payload, created_at FROM events ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|r| r.into_stored_event()).collect())
    }
}

#[derive(Debug, sqlx::FromRow)]
struct ObjectRow {
    id: String,
    object_type: String,
    data: String,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl ObjectRow {
    fn into_stored_object(self) -> StoredObject {
        let data: serde_json::Value =
            serde_json::from_str(&self.data).unwrap_or(serde_json::Value::Null);
        StoredObject {
            id: self.id,
            object_type: self.object_type,
            data,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
struct EventRow {
    id: String,
    event_type: String,
    payload: String,
    created_at: chrono::DateTime<chrono::Utc>,
}

impl EventRow {
    fn into_stored_event(self) -> StoredEvent {
        let payload: serde_json::Value =
            serde_json::from_str(&self.payload).unwrap_or(serde_json::Value::Null);
        StoredEvent {
            id: self.id,
            event_type: self.event_type,
            payload,
            created_at: self.created_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    async fn setup_repo() -> SqliteObjectRepository {
        SqliteObjectRepository::new("sqlite::memory:")
            .await
            .expect("failed to create repo")
    }

    fn make_object(id: &str, obj_type: &str) -> StoredObject {
        StoredObject {
            id: id.to_string(),
            object_type: obj_type.to_string(),
            data: serde_json::json!({"name": "test"}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    #[tokio::test]
    async fn test_insert_object() {
        let repo = setup_repo().await;
        let obj = make_object("obj-1", "FileObject");

        repo.create(obj).await.expect("insert failed");

        let found = repo
            .get(&ObjectId::from("obj-1"))
            .await
            .expect("get failed")
            .expect("object not found");
        assert_eq!(found.id, "obj-1");
        assert_eq!(found.object_type, "FileObject");
    }

    #[tokio::test]
    async fn test_read_object() {
        let repo = setup_repo().await;
        let obj = make_object("obj-2", "FolderObject");
        repo.create(obj).await.unwrap();

        let found = repo.get(&ObjectId::from("obj-2")).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().object_type, "FolderObject");
    }

    #[tokio::test]
    async fn test_read_nonexistent() {
        let repo = setup_repo().await;
        let found = repo.get(&ObjectId::from("nonexistent")).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_update_object() {
        let repo = setup_repo().await;
        let obj = make_object("obj-3", "FileObject");
        repo.create(obj).await.unwrap();

        let new_data = serde_json::json!({"name": "updated"});
        repo.update(&ObjectId::from("obj-3"), new_data)
            .await
            .unwrap();

        let found = repo.get(&ObjectId::from("obj-3")).await.unwrap().unwrap();
        assert_eq!(found.data["name"], "updated");
    }

    #[tokio::test]
    async fn test_delete_object() {
        let repo = setup_repo().await;
        let obj = make_object("obj-4", "FileObject");
        repo.create(obj).await.unwrap();

        repo.delete(&ObjectId::from("obj-4")).await.unwrap();

        let found = repo.get(&ObjectId::from("obj-4")).await.unwrap();
        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_list_objects() {
        let repo = setup_repo().await;
        let obj1 = make_object("obj-a", "FileObject");
        let obj2 = make_object("obj-b", "FolderObject");

        repo.create(obj1).await.unwrap();
        repo.create(obj2).await.unwrap();

        let list = repo.list().await.unwrap();
        assert_eq!(list.len(), 2);
    }

    #[tokio::test]
    async fn test_insert_event() {
        let event_repo = SqliteEventRepository::new("sqlite::memory:").await.unwrap();
        let event = StoredEvent {
            id: "evt-1".into(),
            event_type: "ObjectCreated".into(),
            payload: serde_json::json!({"id": "obj-x"}),
            created_at: Utc::now(),
        };

        event_repo.store(event).await.unwrap();

        let events = event_repo.list().await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].id, "evt-1");
        assert_eq!(events[0].event_type, "ObjectCreated");
    }
}
