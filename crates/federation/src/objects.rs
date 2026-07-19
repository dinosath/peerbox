use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const ACTIVITY_STREAMS_CONTEXT: &str = "https://www.w3.org/ns/activitystreams";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityPubObject {
    #[serde(rename = "@context", default = "default_context")]
    pub context: String,
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(rename = "attributedTo", default)]
    pub attributed_to: Option<String>,
    #[serde(default)]
    pub published: Option<DateTime<Utc>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
}

fn default_context() -> String {
    ACTIVITY_STREAMS_CONTEXT.to_string()
}

impl ActivityPubObject {
    pub fn new(id: String, object_type: &str) -> Self {
        Self {
            context: ACTIVITY_STREAMS_CONTEXT.to_string(),
            id,
            object_type: object_type.to_string(),
            name: None,
            content: None,
            attributed_to: None,
            published: Some(Utc::now()),
            to: Vec::new(),
            cc: Vec::new(),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_content(mut self, content: &str) -> Self {
        self.content = Some(content.to_string());
        self
    }

    pub fn with_attributed_to(mut self, actor: &str) -> Self {
        self.attributed_to = Some(actor.to_string());
        self
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap_or(serde_json::Value::Null)
    }

    pub fn from_json(json: &serde_json::Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(json.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectRef {
    pub id: String,
    #[serde(rename = "type")]
    pub object_type: String,
    #[serde(rename = "manifestId")]
    pub manifest_id: String,
    #[serde(default)]
    pub name: Option<String>,
}

impl ObjectRef {
    pub fn new(id: String, manifest_id: String) -> Self {
        Self {
            id,
            object_type: "PeerBoxObject".to_string(),
            manifest_id,
            name: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_activity_pub_object_creation() {
        let obj = ActivityPubObject::new("https://node.example/objects/1".into(), "Note");
        assert_eq!(obj.id, "https://node.example/objects/1");
        assert_eq!(obj.object_type, "Note");
        assert_eq!(obj.context, ACTIVITY_STREAMS_CONTEXT);
    }

    #[test]
    fn test_activity_pub_object_json_roundtrip() {
        let obj = ActivityPubObject::new("https://node.example/objects/1".into(), "Note")
            .with_name("Test Object")
            .with_content("Test content")
            .with_attributed_to("https://node.example/actor");
        let json = obj.to_json_value();
        let deserialized = ActivityPubObject::from_json(&json).unwrap();
        assert_eq!(deserialized.id, obj.id);
        assert_eq!(deserialized.object_type, obj.object_type);
        assert_eq!(deserialized.name, obj.name);
        assert_eq!(deserialized.content, obj.content);
    }

    #[test]
    fn test_object_ref_creation() {
        let obj_ref = ObjectRef::new("https://node.example/objects/1".into(), "abc123".into());
        assert_eq!(obj_ref.object_type, "PeerBoxObject");
        assert_eq!(obj_ref.manifest_id, "abc123");
    }

    #[test]
    fn test_object_ref_with_name() {
        let obj_ref = ObjectRef::new("https://node.example/objects/1".into(), "abc123".into())
            .with_name("MyFile.txt");
        assert_eq!(obj_ref.name, Some("MyFile.txt".into()));
    }

    #[test]
    fn test_object_ref_serialization() {
        let obj_ref = ObjectRef::new("https://node.example/objects/1".into(), "abc123".into())
            .with_name("Test");
        let json = serde_json::to_value(&obj_ref).unwrap();
        assert_eq!(json["manifestId"], "abc123");
        assert_eq!(json["name"], "Test");
    }
}
