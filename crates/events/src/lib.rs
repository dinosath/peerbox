use common::ObjectId;
use std::future::Future;
use tracing::info;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    ObjectCreated { id: ObjectId },
    ObjectUpdated { id: ObjectId },
    ObjectDeleted { id: ObjectId },
}

impl Event {
    pub fn event_type(&self) -> &str {
        match self {
            Event::ObjectCreated { .. } => "ObjectCreated",
            Event::ObjectUpdated { .. } => "ObjectUpdated",
            Event::ObjectDeleted { .. } => "ObjectDeleted",
        }
    }
}

pub struct EventBus {
    sender: tokio::sync::broadcast::Sender<Event>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (sender, _) = tokio::sync::broadcast::channel(capacity);
        Self { sender }
    }

    pub async fn publish(&self, event: Event) -> anyhow::Result<()> {
        info!("event published type={}", event.event_type());
        let _ = self.sender.send(event);
        Ok(())
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Event> {
        self.sender.subscribe()
    }

    pub async fn subscribe_handler<F, Fut>(&self, handler: F)
    where
        F: Fn(Event) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut rx = self.sender.subscribe();
        tokio::spawn(async move {
            while let Ok(event) = rx.recv().await {
                handler(event).await;
            }
        });
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new(256)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[tokio::test]
    async fn test_publish_single_subscriber() {
        let bus = EventBus::new(16);
        let mut rx = bus.subscribe();

        bus.publish(Event::ObjectCreated {
            id: ObjectId::from("test"),
        })
        .await
        .unwrap();

        let event = rx.recv().await.unwrap();
        assert_eq!(event.event_type(), "ObjectCreated");
    }

    #[tokio::test]
    async fn test_publish_multiple_subscribers() {
        let bus = EventBus::new(16);
        let mut rx1 = bus.subscribe();
        let mut rx2 = bus.subscribe();

        bus.publish(Event::ObjectDeleted {
            id: ObjectId::from("test"),
        })
        .await
        .unwrap();

        let e1 = rx1.recv().await.unwrap();
        let e2 = rx2.recv().await.unwrap();
        assert_eq!(e1.event_type(), "ObjectDeleted");
        assert_eq!(e2.event_type(), "ObjectDeleted");
    }

    #[tokio::test]
    async fn test_handler_subscription() {
        let bus = Arc::new(EventBus::new(16));
        let received = Arc::new(Mutex::new(Vec::new()));
        let received_clone = received.clone();

        bus.subscribe_handler(move |event| {
            let received = received_clone.clone();
            async move {
                received.lock().await.push(event.event_type().to_string());
            }
        })
        .await;

        bus.publish(Event::ObjectUpdated {
            id: ObjectId::from("test"),
        })
        .await
        .unwrap();

        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        let events = received.lock().await;
        assert_eq!(events.len(), 1);
        assert_eq!(events[0], "ObjectUpdated");
    }

    #[tokio::test]
    async fn test_event_types() {
        assert_eq!(
            Event::ObjectCreated {
                id: ObjectId::from("x")
            }
            .event_type(),
            "ObjectCreated"
        );
        assert_eq!(
            Event::ObjectUpdated {
                id: ObjectId::from("x")
            }
            .event_type(),
            "ObjectUpdated"
        );
        assert_eq!(
            Event::ObjectDeleted {
                id: ObjectId::from("x")
            }
            .event_type(),
            "ObjectDeleted"
        );
    }
}
