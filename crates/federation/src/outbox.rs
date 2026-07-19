use crate::activities::ActivityPubActivity;
use crate::FederationError;
use std::collections::VecDeque;
use tokio::sync::Mutex;

pub struct Outbox {
    queue: Mutex<VecDeque<(ActivityPubActivity, Vec<String>)>>,
    max_queue_size: usize,
}

impl Outbox {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(VecDeque::new()),
            max_queue_size: 1000,
        }
    }

    pub fn with_max_queue_size(mut self, size: usize) -> Self {
        self.max_queue_size = size;
        self
    }

    pub async fn send_activity(
        &self,
        activity: ActivityPubActivity,
        recipients: Vec<String>,
    ) -> Result<(), FederationError> {
        let mut queue = self.queue.lock().await;
        if queue.len() >= self.max_queue_size {
            return Err(FederationError::DeliveryFailed("outbox queue full".into()));
        }
        queue.push_back((activity, recipients));
        Ok(())
    }

    pub async fn drain_pending(&self) -> Vec<(ActivityPubActivity, Vec<String>)> {
        let mut queue = self.queue.lock().await;
        let pending: Vec<_> = queue.drain(..).collect();
        pending
    }

    pub async fn pending_count(&self) -> usize {
        self.queue.lock().await.len()
    }
}

impl Default for Outbox {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::activities::CreateActivity;

    #[tokio::test]
    async fn test_outbox_send_and_drain() {
        let outbox = Outbox::new();
        let activity = ActivityPubActivity::from(CreateActivity::new(
            "https://a.example/actor",
            "https://a.example/obj/1",
        ));
        let recipients = vec!["https://b.example/inbox".into()];

        outbox
            .send_activity(activity.clone(), recipients)
            .await
            .unwrap();
        assert_eq!(outbox.pending_count().await, 1);

        let pending = outbox.drain_pending().await;
        assert_eq!(pending.len(), 1);
        assert_eq!(outbox.pending_count().await, 0);
    }

    #[tokio::test]
    async fn test_outbox_queue_full() {
        let outbox = Outbox::new().with_max_queue_size(2);
        let activity = ActivityPubActivity::from(CreateActivity::new(
            "https://a.example/actor",
            "https://a.example/obj/1",
        ));
        let recipients = vec!["https://b.example/inbox".into()];

        outbox
            .send_activity(activity.clone(), recipients.clone())
            .await
            .unwrap();
        outbox
            .send_activity(activity.clone(), recipients.clone())
            .await
            .unwrap();
        let result = outbox.send_activity(activity, recipients).await;
        assert!(result.is_err());
    }
}
