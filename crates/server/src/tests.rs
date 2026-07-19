#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use async_trait::async_trait;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
    };
    use common::ObjectId;
    use database::StoredEvent;
    use database::{ObjectRepository, StoredObject};
    use events::EventBus;
    use tower::util::ServiceExt;

    use crate::state::AppState;

    use std::collections::HashMap;
    use tokio::sync::RwLock;

    struct TestObjectRepository {
        objects: RwLock<HashMap<String, StoredObject>>,
    }

    impl TestObjectRepository {
        fn new() -> Self {
            Self {
                objects: RwLock::new(HashMap::new()),
            }
        }
    }

    #[async_trait]
    impl ObjectRepository for TestObjectRepository {
        async fn create(&self, object: StoredObject) -> anyhow::Result<()> {
            self.objects.write().await.insert(object.id.clone(), object);
            Ok(())
        }

        async fn get(&self, id: &ObjectId) -> anyhow::Result<Option<StoredObject>> {
            Ok(self.objects.read().await.get(&id.0).cloned())
        }

        async fn list(&self) -> anyhow::Result<Vec<StoredObject>> {
            Ok(self.objects.read().await.values().cloned().collect())
        }

        async fn update(&self, id: &ObjectId, data: serde_json::Value) -> anyhow::Result<()> {
            if let Some(obj) = self.objects.write().await.get_mut(&id.0) {
                obj.data = data;
            }
            Ok(())
        }

        async fn delete(&self, id: &ObjectId) -> anyhow::Result<()> {
            self.objects.write().await.remove(&id.0);
            Ok(())
        }
    }

    struct TestEventRepository {
        events: RwLock<Vec<StoredEvent>>,
    }

    impl TestEventRepository {
        fn new() -> Self {
            Self {
                events: RwLock::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl database::EventRepository for TestEventRepository {
        async fn store(&self, event: StoredEvent) -> anyhow::Result<()> {
            self.events.write().await.push(event);
            Ok(())
        }

        async fn list(&self) -> anyhow::Result<Vec<StoredEvent>> {
            Ok(self.events.read().await.clone())
        }
    }

    struct TestCryptoProvider;

    #[async_trait]
    impl crypto::CryptoProvider for TestCryptoProvider {
        async fn sign(&self, _data: &[u8]) -> anyhow::Result<Vec<u8>> {
            Ok(vec![0u8; 64])
        }

        async fn verify(&self, _data: &[u8], _signature: &[u8]) -> anyhow::Result<bool> {
            Ok(true)
        }
    }

    fn test_state() -> Arc<AppState> {
        let object_repo = Arc::new(TestObjectRepository::new());
        let event_bus = Arc::new(EventBus::new(16));
        let storage = Arc::new(storage::MemoryStorageProvider::new());
        let crypto: Arc<dyn crypto::CryptoProvider> = Arc::new(TestCryptoProvider);
        let event_repo = Arc::new(TestEventRepository::new());

        let application = Arc::new(dc_core::Application::new(
            object_repo,
            event_bus,
            storage,
            crypto,
            event_repo,
        ));

        Arc::new(AppState {
            application,
            node_id: "test-node-1234".to_string(),
        })
    }

    #[tokio::test]
    async fn test_health() {
        let state = test_state();
        let app = crate::create_router(state);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body["status"], "ok");
        assert_eq!(body["node_id"], "test-node-1234");
        assert!(!body["version"].as_str().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_object_not_found() {
        let state = test_state();
        let app = crate::create_router(state);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/objects/nonexistent-id")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_objects_empty() {
        let state = test_state();
        let app = crate::create_router(state);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/objects")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert!(body["objects"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_activitypub_inbox_valid_activity() {
        let state = test_state();
        let app = crate::create_router(state);

        let activity = serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "type": "Create",
            "actor": "https://remote.example/actor",
            "object": "https://remote.example/obj/1",
            "published": "2024-01-01T00:00:00Z"
        });

        let request = Request::builder()
            .method(Method::POST)
            .uri("/activitypub/inbox")
            .header("Content-Type", "application/json")
            .body(Body::from(serde_json::to_vec(&activity).unwrap()))
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::ACCEPTED);
    }

    #[tokio::test]
    async fn test_webfinger() {
        let state = test_state();
        let app = crate::create_router(state);

        let request = Request::builder()
            .method(Method::GET)
            .uri("/.well-known/webfinger?resource=acct:user@example.com")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body_bytes = axum::body::to_bytes(response.into_body(), 1024 * 1024)
            .await
            .unwrap();
        let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(body["subject"], "acct:user@example.com");
        assert!(body["links"].as_array().unwrap().len() > 0);
        assert_eq!(body["links"][0]["rel"], "self");
        assert_eq!(body["links"][0]["type"], "application/activity+json");
    }
}
