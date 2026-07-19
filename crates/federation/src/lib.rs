pub mod activities;
pub mod actors;
pub mod federation_manager;
pub mod inbox;
pub mod objects;
pub mod outbox;
pub mod permissions;
pub mod webfinger;

pub use activities::{
    parse_activity, AcceptActivity, ActivityPubActivity, AnnounceActivity, CreateActivity,
    DeleteActivity, FollowActivity, LikeActivity, UndoActivity, UpdateActivity,
};
pub use actors::{ActivityPubActor, ActorPublicKey, NodeActor, OrganizationActor, UserActor};
pub use federation_manager::FederationManager;
pub use inbox::{ActivityHandler, Inbox};
pub use objects::{ActivityPubObject, ObjectRef};
pub use outbox::Outbox;
pub use permissions::{Permission, PermissionEntry, PermissionManager};
pub use webfinger::{parse_webfinger_query, resolve_webfinger, WebfingerResponse};

#[derive(Debug, thiserror::Error)]
pub enum FederationError {
    #[error("actor not found: {0}")]
    ActorNotFound(String),
    #[error("invalid activity: {0}")]
    InvalidActivity(String),
    #[error("delivery failed: {0}")]
    DeliveryFailed(String),
    #[error("serialization error: {0}")]
    SerializationError(String),
    #[error("signature verification failed: {0}")]
    SignatureVerificationFailed(String),
    #[error("webfinger error: {0}")]
    WebfingerError(String),
    #[error("internal error: {0}")]
    Internal(String),
}
