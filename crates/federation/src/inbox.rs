use crate::activities::ActivityPubActivity;
use crate::FederationError;
use async_trait::async_trait;

#[async_trait]
pub trait ActivityHandler: Send + Sync {
    async fn handle_create(&self, actor: &str, object: &str) -> Result<(), FederationError>;
    async fn handle_update(&self, actor: &str, object: &str) -> Result<(), FederationError>;
    async fn handle_delete(&self, actor: &str, object: &str) -> Result<(), FederationError>;
    async fn handle_follow(&self, actor: &str, object: &str) -> Result<(), FederationError>;
    async fn handle_accept(&self, actor: &str, object: &str) -> Result<(), FederationError>;
    async fn handle_announce(&self, actor: &str, object: &str) -> Result<(), FederationError>;
    async fn handle_like(&self, actor: &str, object: &str) -> Result<(), FederationError>;
    async fn handle_undo(&self, actor: &str, object: &str) -> Result<(), FederationError>;
}

pub struct Inbox {
    handler: Box<dyn ActivityHandler>,
}

impl Inbox {
    pub fn new(handler: Box<dyn ActivityHandler>) -> Self {
        Self { handler }
    }

    pub async fn process_activity(
        &self,
        activity: &ActivityPubActivity,
    ) -> Result<(), FederationError> {
        let actor = activity.actor();
        let object = activity.object();

        match activity {
            ActivityPubActivity::Create(_) => self.handler.handle_create(actor, object).await,
            ActivityPubActivity::Update(_) => self.handler.handle_update(actor, object).await,
            ActivityPubActivity::Delete(_) => self.handler.handle_delete(actor, object).await,
            ActivityPubActivity::Follow(_) => self.handler.handle_follow(actor, object).await,
            ActivityPubActivity::Accept(_) => self.handler.handle_accept(actor, object).await,
            ActivityPubActivity::Announce(_) => self.handler.handle_announce(actor, object).await,
            ActivityPubActivity::Like(_) => self.handler.handle_like(actor, object).await,
            ActivityPubActivity::Undo(_) => self.handler.handle_undo(actor, object).await,
        }
    }

    pub async fn process_json(&self, json: &serde_json::Value) -> Result<(), FederationError> {
        let activity = crate::activities::parse_activity(json)?;
        self.process_activity(&activity).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct TestHandler {
        received: Mutex<Vec<(String, String, String)>>,
    }

    impl TestHandler {
        fn new() -> Self {
            Self {
                received: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl ActivityHandler for TestHandler {
        async fn handle_create(&self, actor: &str, object: &str) -> Result<(), FederationError> {
            self.received
                .lock()
                .unwrap()
                .push(("Create".into(), actor.into(), object.into()));
            Ok(())
        }
        async fn handle_update(&self, actor: &str, object: &str) -> Result<(), FederationError> {
            self.received
                .lock()
                .unwrap()
                .push(("Update".into(), actor.into(), object.into()));
            Ok(())
        }
        async fn handle_delete(&self, actor: &str, object: &str) -> Result<(), FederationError> {
            self.received
                .lock()
                .unwrap()
                .push(("Delete".into(), actor.into(), object.into()));
            Ok(())
        }
        async fn handle_follow(&self, actor: &str, object: &str) -> Result<(), FederationError> {
            self.received
                .lock()
                .unwrap()
                .push(("Follow".into(), actor.into(), object.into()));
            Ok(())
        }
        async fn handle_accept(&self, actor: &str, object: &str) -> Result<(), FederationError> {
            self.received
                .lock()
                .unwrap()
                .push(("Accept".into(), actor.into(), object.into()));
            Ok(())
        }
        async fn handle_announce(&self, actor: &str, object: &str) -> Result<(), FederationError> {
            self.received
                .lock()
                .unwrap()
                .push(("Announce".into(), actor.into(), object.into()));
            Ok(())
        }
        async fn handle_like(&self, actor: &str, object: &str) -> Result<(), FederationError> {
            self.received
                .lock()
                .unwrap()
                .push(("Like".into(), actor.into(), object.into()));
            Ok(())
        }
        async fn handle_undo(&self, actor: &str, object: &str) -> Result<(), FederationError> {
            self.received
                .lock()
                .unwrap()
                .push(("Undo".into(), actor.into(), object.into()));
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_inbox_process_create_activity() {
        let handler = TestHandler::new();
        let inbox = Inbox::new(Box::new(handler));

        let activity = crate::activities::CreateActivity::new(
            "https://remote.example/actor",
            "https://remote.example/obj/1",
        );
        inbox.process_activity(&activity.into()).await.unwrap();
    }

    #[tokio::test]
    async fn test_inbox_process_json() {
        let handler = TestHandler::new();
        let inbox = Inbox::new(Box::new(handler));

        let json = serde_json::json!({
            "@context": "https://www.w3.org/ns/activitystreams",
            "type": "Create",
            "actor": "https://remote.example/actor",
            "object": "https://remote.example/obj/1",
            "published": "2024-01-01T00:00:00Z"
        });

        let result = inbox.process_json(&json).await;
        assert!(result.is_ok());
    }
}
