use std::sync::Arc;

use dc_core::Application;
use database::ObjectRepository;
use database::SqliteObjectRepository;
use events::EventBus;
use objects::FileObject;
use storage::MemoryStorageProvider;

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();
}

#[tokio::test]
async fn test_full_integration_flow() {
    init_tracing();

    let repository = Arc::new(
        SqliteObjectRepository::new("sqlite::memory:")
            .await
            .expect("failed to create repository"),
    );

    let event_bus = Arc::new(EventBus::new(256));
    let storage = Arc::new(MemoryStorageProvider::new());

    let app = Application::new(
        repository.clone(),
        event_bus.clone(),
        storage.clone(),
    );

    app.start().await.expect("failed to start application");

    let file = FileObject::new("hello.txt".into(), 13, Some("text/plain".into()));
    let file_id = file.id.clone();

    let created_id = app
        .object_service()
        .create_file(file)
        .await
        .expect("failed to create file");

    assert_eq!(created_id, file_id);

    let stored = repository
        .get(&file_id)
        .await
        .expect("failed to get object");
    assert!(stored.is_some());
    assert_eq!(stored.unwrap().object_type, "FileObject");

    let retrieved: FileObject = app
        .object_service()
        .get_object(&file_id)
        .await
        .expect("failed to deserialize")
        .expect("object not found");
    assert_eq!(retrieved.name, "hello.txt");
    assert_eq!(retrieved.size, 13);

    app.shutdown().await.expect("failed to shutdown");
}

#[tokio::test]
async fn test_create_and_delete_object() {
    init_tracing();

    let repository = Arc::new(
        SqliteObjectRepository::new("sqlite::memory:")
            .await
            .expect("failed to create repository"),
    );

    let event_bus = Arc::new(EventBus::new(256));
    let storage = Arc::new(MemoryStorageProvider::new());

    let app = Application::new(repository.clone(), event_bus.clone(), storage.clone());
    app.start().await.unwrap();

    let file = FileObject::new("delete-me.txt".into(), 0, None);
    let file_id = file.id.clone();

    app.object_service().create_file(file).await.unwrap();

    let exists = repository.get(&file_id).await.unwrap();
    assert!(exists.is_some());

    app.object_service()
        .delete_object(&file_id)
        .await
        .unwrap();

    let gone = repository.get(&file_id).await.unwrap();
    assert!(gone.is_none());

    app.shutdown().await.unwrap();
}

#[tokio::test]
async fn test_event_emission_on_create() {
    init_tracing();

    let repository = Arc::new(
        SqliteObjectRepository::new("sqlite::memory:")
            .await
            .unwrap(),
    );

    let event_bus = Arc::new(EventBus::new(256));
    let storage = Arc::new(MemoryStorageProvider::new());
    let mut rx = event_bus.subscribe();

    let app = Application::new(repository.clone(), event_bus.clone(), storage.clone());
    app.start().await.unwrap();

    let file = FileObject::new("event-test.txt".into(), 5, None);
    app.object_service().create_file(file).await.unwrap();

    let event = tokio::time::timeout(
        std::time::Duration::from_secs(1),
        rx.recv(),
    )
    .await
    .expect("timed out waiting for event")
    .expect("no event received");

    assert_eq!(event.event_type(), "ObjectCreated");

    app.shutdown().await.unwrap();
}
