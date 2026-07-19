use crate::activities::{AcceptActivity, ActivityPubActivity, FollowActivity};
use crate::actors::NodeActor;
use crate::inbox::Inbox;
use crate::objects::ObjectRef;
use crate::outbox::Outbox;
use crate::FederationError;
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct FederationManager {
    known_nodes: RwLock<HashMap<String, NodeActor>>,
    followed_actors: RwLock<Vec<String>>,
    inbox: Inbox,
    outbox: Outbox,
    local_node_id: String,
}

impl FederationManager {
    pub fn new(inbox: Inbox, outbox: Outbox, local_node_id: String) -> Self {
        Self {
            known_nodes: RwLock::new(HashMap::new()),
            followed_actors: RwLock::new(Vec::new()),
            inbox,
            outbox,
            local_node_id,
        }
    }

    pub async fn track_node(&self, node: NodeActor) {
        self.known_nodes.write().await.insert(node.id.clone(), node);
    }

    pub async fn is_known_node(&self, node_id: &str) -> bool {
        self.known_nodes.read().await.contains_key(node_id)
    }

    pub async fn get_known_nodes(&self) -> Vec<NodeActor> {
        self.known_nodes.read().await.values().cloned().collect()
    }

    pub async fn follow(&self, actor_url: &str) -> Result<(), FederationError> {
        if self
            .followed_actors
            .read()
            .await
            .contains(&actor_url.to_string())
        {
            return Ok(());
        }

        let follow = FollowActivity::new(&self.local_node_id, actor_url);
        let activity: ActivityPubActivity = follow.into();

        self.outbox
            .send_activity(activity, vec![actor_url.to_string()])
            .await?;

        self.followed_actors
            .write()
            .await
            .push(actor_url.to_string());
        Ok(())
    }

    pub async fn unfollow(&self, actor_url: &str) -> Result<(), FederationError> {
        let mut followed = self.followed_actors.write().await;
        followed.retain(|a| a != actor_url);
        Ok(())
    }

    pub async fn is_following(&self, actor_url: &str) -> bool {
        self.followed_actors
            .read()
            .await
            .contains(&actor_url.to_string())
    }

    pub async fn propagate_object_metadata(
        &self,
        object_ref: &ObjectRef,
    ) -> Result<(), FederationError> {
        let create = crate::activities::CreateActivity::new(&self.local_node_id, &object_ref.id);
        let activity: ActivityPubActivity = create.into();

        let recipients: Vec<String> = self
            .known_nodes
            .read()
            .await
            .values()
            .map(|n| n.inbox_url.clone())
            .collect();

        if recipients.is_empty() {
            return Ok(());
        }

        self.outbox.send_activity(activity, recipients).await?;
        Ok(())
    }

    pub async fn accept_follow(&self, follower_url: &str) -> Result<(), FederationError> {
        let accept = AcceptActivity::new(&self.local_node_id, follower_url);
        let activity: ActivityPubActivity = accept.into();

        self.outbox
            .send_activity(activity, vec![follower_url.to_string()])
            .await?;
        Ok(())
    }

    pub async fn process_incoming(
        &self,
        activity: &ActivityPubActivity,
    ) -> Result<(), FederationError> {
        self.inbox.process_activity(activity).await
    }

    pub fn inbox(&self) -> &Inbox {
        &self.inbox
    }

    pub fn outbox(&self) -> &Outbox {
        &self.outbox
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activities::CreateActivity;
    use crate::actors::NodeActor;
    use crate::inbox::ActivityHandler;
    use async_trait::async_trait;

    struct NoopHandler;

    #[async_trait]
    impl ActivityHandler for NoopHandler {
        async fn handle_create(&self, _: &str, _: &str) -> Result<(), FederationError> {
            Ok(())
        }
        async fn handle_update(&self, _: &str, _: &str) -> Result<(), FederationError> {
            Ok(())
        }
        async fn handle_delete(&self, _: &str, _: &str) -> Result<(), FederationError> {
            Ok(())
        }
        async fn handle_follow(&self, _: &str, _: &str) -> Result<(), FederationError> {
            Ok(())
        }
        async fn handle_accept(&self, _: &str, _: &str) -> Result<(), FederationError> {
            Ok(())
        }
        async fn handle_announce(&self, _: &str, _: &str) -> Result<(), FederationError> {
            Ok(())
        }
        async fn handle_like(&self, _: &str, _: &str) -> Result<(), FederationError> {
            Ok(())
        }
        async fn handle_undo(&self, _: &str, _: &str) -> Result<(), FederationError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_track_node() {
        let inbox = Inbox::new(Box::new(NoopHandler));
        let outbox = Outbox::new();
        let manager = FederationManager::new(inbox, outbox, "https://local.example".into());

        let node = NodeActor::new(
            "https://remote.example".into(),
            "Remote Node".into(),
            "pem".into(),
        );
        manager.track_node(node).await;

        assert!(manager.is_known_node("https://remote.example").await);
        assert_eq!(manager.get_known_nodes().await.len(), 1);
    }

    #[tokio::test]
    async fn test_follow_unfollow() {
        let inbox = Inbox::new(Box::new(NoopHandler));
        let outbox = Outbox::new();
        let manager = FederationManager::new(inbox, outbox, "https://local.example".into());

        let target = "https://remote.example/actor";
        manager.follow(target).await.unwrap();
        assert!(manager.is_following(target).await);

        manager.unfollow(target).await.unwrap();
        assert!(!manager.is_following(target).await);
    }

    #[tokio::test]
    async fn test_propagate_metadata() {
        let inbox = Inbox::new(Box::new(NoopHandler));
        let outbox = Outbox::new();
        let manager = FederationManager::new(inbox, outbox, "https://local.example".into());

        let obj_ref = ObjectRef::new("https://local.example/objects/test".into(), "abc123".into());

        let node = NodeActor::new(
            "https://remote.example".into(),
            "Remote".into(),
            "pem".into(),
        );
        manager.track_node(node).await;

        let result = manager.propagate_object_metadata(&obj_ref).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_process_incoming_activity() {
        let inbox = Inbox::new(Box::new(NoopHandler));
        let outbox = Outbox::new();
        let manager = FederationManager::new(inbox, outbox, "https://local.example".into());

        let activity = ActivityPubActivity::from(CreateActivity::new(
            "https://remote.example/actor",
            "https://remote.example/obj/1",
        ));
        let result = manager.process_incoming(&activity).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_accept_follow() {
        let inbox = Inbox::new(Box::new(NoopHandler));
        let outbox = Outbox::new();
        let manager = FederationManager::new(inbox, outbox, "https://local.example".into());

        let result = manager.accept_follow("https://remote.example/actor").await;
        assert!(result.is_ok());
    }
}
