use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[async_trait]
pub trait ActivityPubActor: Send + Sync {
    fn id(&self) -> &str;
    fn actor_type(&self) -> &str;
    fn inbox_url(&self) -> &str;
    fn outbox_url(&self) -> &str;
    fn name(&self) -> &str;
    fn public_key_pem(&self) -> &str;
    fn to_json_ld(&self) -> serde_json::Value;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ActorPublicKey {
    pub id: String,
    pub owner: String,
    #[serde(rename = "publicKeyPem")]
    pub public_key_pem: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeActor {
    pub id: String,
    pub name: String,
    pub inbox_url: String,
    pub outbox_url: String,
    pub public_key: ActorPublicKey,
}

impl NodeActor {
    pub fn new(id: String, name: String, public_key_pem: String) -> Self {
        let public_key = ActorPublicKey {
            id: format!("{id}#main-key"),
            owner: id.clone(),
            public_key_pem,
        };
        Self {
            inbox_url: format!("{id}/inbox"),
            outbox_url: format!("{id}/outbox"),
            id,
            name,
            public_key,
        }
    }
}

#[async_trait]
impl ActivityPubActor for NodeActor {
    fn id(&self) -> &str {
        &self.id
    }
    fn actor_type(&self) -> &str {
        "Application"
    }
    fn inbox_url(&self) -> &str {
        &self.inbox_url
    }
    fn outbox_url(&self) -> &str {
        &self.outbox_url
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn public_key_pem(&self) -> &str {
        &self.public_key.public_key_pem
    }
    fn to_json_ld(&self) -> serde_json::Value {
        serde_json::json!({
            "@context": ["https://www.w3.org/ns/activitystreams", "https://w3id.org/security/v1"],
            "id": self.id,
            "type": self.actor_type(),
            "name": self.name,
            "inbox": self.inbox_url,
            "outbox": self.outbox_url,
            "publicKey": {
                "id": self.public_key.id,
                "owner": self.public_key.owner,
                "publicKeyPem": self.public_key.public_key_pem
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserActor {
    pub id: String,
    pub name: String,
    pub inbox_url: String,
    pub outbox_url: String,
    pub public_key: ActorPublicKey,
}

impl UserActor {
    pub fn new(id: String, name: String, public_key_pem: String) -> Self {
        let public_key = ActorPublicKey {
            id: format!("{id}#main-key"),
            owner: id.clone(),
            public_key_pem,
        };
        Self {
            inbox_url: format!("{id}/inbox"),
            outbox_url: format!("{id}/outbox"),
            id,
            name,
            public_key,
        }
    }
}

#[async_trait]
impl ActivityPubActor for UserActor {
    fn id(&self) -> &str {
        &self.id
    }
    fn actor_type(&self) -> &str {
        "Person"
    }
    fn inbox_url(&self) -> &str {
        &self.inbox_url
    }
    fn outbox_url(&self) -> &str {
        &self.outbox_url
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn public_key_pem(&self) -> &str {
        &self.public_key.public_key_pem
    }
    fn to_json_ld(&self) -> serde_json::Value {
        serde_json::json!({
            "@context": ["https://www.w3.org/ns/activitystreams", "https://w3id.org/security/v1"],
            "id": self.id,
            "type": self.actor_type(),
            "name": self.name,
            "inbox": self.inbox_url,
            "outbox": self.outbox_url,
            "publicKey": {
                "id": self.public_key.id,
                "owner": self.public_key.owner,
                "publicKeyPem": self.public_key.public_key_pem
            }
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrganizationActor {
    pub id: String,
    pub name: String,
    pub inbox_url: String,
    pub outbox_url: String,
    pub public_key: ActorPublicKey,
}

impl OrganizationActor {
    pub fn new(id: String, name: String, public_key_pem: String) -> Self {
        let public_key = ActorPublicKey {
            id: format!("{id}#main-key"),
            owner: id.clone(),
            public_key_pem,
        };
        Self {
            inbox_url: format!("{id}/inbox"),
            outbox_url: format!("{id}/outbox"),
            id,
            name,
            public_key,
        }
    }
}

#[async_trait]
impl ActivityPubActor for OrganizationActor {
    fn id(&self) -> &str {
        &self.id
    }
    fn actor_type(&self) -> &str {
        "Organization"
    }
    fn inbox_url(&self) -> &str {
        &self.inbox_url
    }
    fn outbox_url(&self) -> &str {
        &self.outbox_url
    }
    fn name(&self) -> &str {
        &self.name
    }
    fn public_key_pem(&self) -> &str {
        &self.public_key.public_key_pem
    }
    fn to_json_ld(&self) -> serde_json::Value {
        serde_json::json!({
            "@context": ["https://www.w3.org/ns/activitystreams", "https://w3id.org/security/v1"],
            "id": self.id,
            "type": self.actor_type(),
            "name": self.name,
            "inbox": self.inbox_url,
            "outbox": self.outbox_url,
            "publicKey": {
                "id": self.public_key.id,
                "owner": self.public_key.owner,
                "publicKeyPem": self.public_key.public_key_pem
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_actor_creation() {
        let node = NodeActor::new(
            "https://node.example".into(),
            "Test Node".into(),
            "pk".into(),
        );
        assert_eq!(node.name, "Test Node");
        assert_eq!(node.actor_type(), "Application");
    }

    #[test]
    fn test_user_actor_creation() {
        let user = UserActor::new(
            "https://node.example/users/alice".into(),
            "Alice".into(),
            "pk".into(),
        );
        assert_eq!(user.actor_type(), "Person");
    }

    #[test]
    fn test_organization_actor_creation() {
        let org = OrganizationActor::new(
            "https://node.example/orgs/team".into(),
            "Team".into(),
            "pk".into(),
        );
        assert_eq!(org.actor_type(), "Organization");
    }

    #[test]
    fn test_actor_json_ld_format() {
        let node = NodeActor::new(
            "https://node.example".into(),
            "Test Node".into(),
            "pk".into(),
        );
        let json = node.to_json_ld();
        assert_eq!(json["type"], "Application");
        assert!(json["@context"].is_array());
    }

    #[test]
    fn test_actor_serialization_roundtrip() {
        let user = UserActor::new(
            "https://node.example/users/alice".into(),
            "Alice".into(),
            "pk".into(),
        );
        let json = serde_json::to_value(&user).unwrap();
        let deserialized: UserActor = serde_json::from_value(json).unwrap();
        assert_eq!(user.id, deserialized.id);
    }

    #[test]
    fn test_actor_types() {
        let node = NodeActor::new("https://n.example".into(), "N".into(), "pem".into());
        let user = UserActor::new("https://u.example".into(), "U".into(), "pem".into());
        let org = OrganizationActor::new("https://o.example".into(), "O".into(), "pem".into());
        assert_eq!(node.actor_type(), "Application");
        assert_eq!(user.actor_type(), "Person");
        assert_eq!(org.actor_type(), "Organization");
    }

    #[test]
    fn test_inbox_outbox_urls() {
        let user = UserActor::new(
            "https://node.example/users/alice".into(),
            "Alice".into(),
            "pk".into(),
        );
        assert!(user.inbox_url.ends_with("/inbox"));
        assert!(user.outbox_url.ends_with("/outbox"));
    }
}
