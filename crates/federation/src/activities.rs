use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const ACTIVITY_STREAMS_CONTEXT: &str = "https://www.w3.org/ns/activitystreams";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityBase {
    #[serde(rename = "@context", default = "default_context")]
    pub context: String,
    pub id: String,
    #[serde(rename = "type")]
    pub activity_type: String,
    pub actor: String,
    pub object: String,
    pub published: DateTime<Utc>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub to: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cc: Vec<String>,
}

fn default_context() -> String {
    ACTIVITY_STREAMS_CONTEXT.to_string()
}

impl ActivityBase {
    pub fn new(activity_type: &str, actor: &str, object: &str) -> Self {
        Self {
            context: ACTIVITY_STREAMS_CONTEXT.to_string(),
            id: format!("{}#{}", actor, uuid::Uuid::new_v4()),
            activity_type: activity_type.to_string(),
            actor: actor.to_string(),
            object: object.to_string(),
            published: Utc::now(),
            to: Vec::new(),
            cc: Vec::new(),
        }
    }

    pub fn with_to(mut self, recipients: Vec<String>) -> Self {
        self.to = recipients;
        self
    }

    pub fn with_cc(mut self, cc: Vec<String>) -> Self {
        self.cc = cc;
        self
    }

    pub fn to_json_value(&self) -> serde_json::Value {
        let mut v = serde_json::json!({
            "@context": self.context,
            "id": self.id,
            "type": self.activity_type,
            "actor": self.actor,
            "object": self.object,
            "published": self.published.to_rfc3339(),
        });
        if !self.to.is_empty() {
            v["to"] = self
                .to
                .iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect::<Vec<_>>()
                .into();
        }
        if !self.cc.is_empty() {
            v["cc"] = self
                .cc
                .iter()
                .map(|s| serde_json::Value::String(s.clone()))
                .collect::<Vec<_>>()
                .into();
        }
        v
    }
}

macro_rules! define_activity {
    ($name:ident, $type_label:expr) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $name {
            pub base: ActivityBase,
        }

        impl $name {
            pub fn new(actor: &str, object: &str) -> Self {
                Self {
                    base: ActivityBase::new($type_label, actor, object),
                }
            }

            pub fn with_to(mut self, recipients: Vec<String>) -> Self {
                self.base = self.base.with_to(recipients);
                self
            }

            pub fn with_cc(mut self, cc: Vec<String>) -> Self {
                self.base = self.base.with_cc(cc);
                self
            }

            pub fn from_base(base: ActivityBase) -> Self {
                Self { base }
            }
        }
    };
}

define_activity!(CreateActivity, "Create");
define_activity!(UpdateActivity, "Update");
define_activity!(DeleteActivity, "Delete");
define_activity!(FollowActivity, "Follow");
define_activity!(AcceptActivity, "Accept");
define_activity!(AnnounceActivity, "Announce");
define_activity!(LikeActivity, "Like");
define_activity!(UndoActivity, "Undo");

#[derive(Debug, Clone)]
pub enum ActivityPubActivity {
    Create(CreateActivity),
    Update(UpdateActivity),
    Delete(DeleteActivity),
    Follow(FollowActivity),
    Accept(AcceptActivity),
    Announce(AnnounceActivity),
    Like(LikeActivity),
    Undo(UndoActivity),
}

impl ActivityPubActivity {
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Create(a) => a.base.to_json_value(),
            Self::Update(a) => a.base.to_json_value(),
            Self::Delete(a) => a.base.to_json_value(),
            Self::Follow(a) => a.base.to_json_value(),
            Self::Accept(a) => a.base.to_json_value(),
            Self::Announce(a) => a.base.to_json_value(),
            Self::Like(a) => a.base.to_json_value(),
            Self::Undo(a) => a.base.to_json_value(),
        }
    }

    pub fn activity_type(&self) -> &str {
        match self {
            Self::Create(_) => "Create",
            Self::Update(_) => "Update",
            Self::Delete(_) => "Delete",
            Self::Follow(_) => "Follow",
            Self::Accept(_) => "Accept",
            Self::Announce(_) => "Announce",
            Self::Like(_) => "Like",
            Self::Undo(_) => "Undo",
        }
    }

    pub fn actor(&self) -> &str {
        match self {
            Self::Create(a) => &a.base.actor,
            Self::Update(a) => &a.base.actor,
            Self::Delete(a) => &a.base.actor,
            Self::Follow(a) => &a.base.actor,
            Self::Accept(a) => &a.base.actor,
            Self::Announce(a) => &a.base.actor,
            Self::Like(a) => &a.base.actor,
            Self::Undo(a) => &a.base.actor,
        }
    }

    pub fn object(&self) -> &str {
        match self {
            Self::Create(a) => &a.base.object,
            Self::Update(a) => &a.base.object,
            Self::Delete(a) => &a.base.object,
            Self::Follow(a) => &a.base.object,
            Self::Accept(a) => &a.base.object,
            Self::Announce(a) => &a.base.object,
            Self::Like(a) => &a.base.object,
            Self::Undo(a) => &a.base.object,
        }
    }
}

impl From<CreateActivity> for ActivityPubActivity {
    fn from(a: CreateActivity) -> Self {
        Self::Create(a)
    }
}
impl From<UpdateActivity> for ActivityPubActivity {
    fn from(a: UpdateActivity) -> Self {
        Self::Update(a)
    }
}
impl From<DeleteActivity> for ActivityPubActivity {
    fn from(a: DeleteActivity) -> Self {
        Self::Delete(a)
    }
}
impl From<FollowActivity> for ActivityPubActivity {
    fn from(a: FollowActivity) -> Self {
        Self::Follow(a)
    }
}
impl From<AcceptActivity> for ActivityPubActivity {
    fn from(a: AcceptActivity) -> Self {
        Self::Accept(a)
    }
}
impl From<AnnounceActivity> for ActivityPubActivity {
    fn from(a: AnnounceActivity) -> Self {
        Self::Announce(a)
    }
}
impl From<LikeActivity> for ActivityPubActivity {
    fn from(a: LikeActivity) -> Self {
        Self::Like(a)
    }
}
impl From<UndoActivity> for ActivityPubActivity {
    fn from(a: UndoActivity) -> Self {
        Self::Undo(a)
    }
}

pub fn parse_activity(
    json: &serde_json::Value,
) -> Result<ActivityPubActivity, crate::FederationError> {
    let activity_type = json["type"]
        .as_str()
        .ok_or_else(|| crate::FederationError::InvalidActivity("missing type field".into()))?;

    let base = ActivityBase {
        context: json["@context"]
            .as_str()
            .unwrap_or(ACTIVITY_STREAMS_CONTEXT)
            .to_string(),
        id: json["id"].as_str().unwrap_or("").to_string(),
        activity_type: activity_type.to_string(),
        actor: json["actor"].as_str().unwrap_or("").to_string(),
        object: json["object"].as_str().unwrap_or("").to_string(),
        published: json["published"]
            .as_str()
            .and_then(|s| {
                DateTime::parse_from_rfc3339(s)
                    .ok()
                    .map(|d| d.with_timezone(&Utc))
            })
            .unwrap_or_else(Utc::now),
        to: parse_string_array(json, "to"),
        cc: parse_string_array(json, "cc"),
    };

    match activity_type {
        "Create" => Ok(CreateActivity::from_base(base).into()),
        "Update" => Ok(UpdateActivity::from_base(base).into()),
        "Delete" => Ok(DeleteActivity::from_base(base).into()),
        "Follow" => Ok(FollowActivity::from_base(base).into()),
        "Accept" => Ok(AcceptActivity::from_base(base).into()),
        "Announce" => Ok(AnnounceActivity::from_base(base).into()),
        "Like" => Ok(LikeActivity::from_base(base).into()),
        "Undo" => Ok(UndoActivity::from_base(base).into()),
        other => Err(crate::FederationError::InvalidActivity(format!(
            "unknown activity type: {other}"
        ))),
    }
}

fn parse_string_array(json: &serde_json::Value, key: &str) -> Vec<String> {
    json[key]
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_activity() {
        let activity =
            CreateActivity::new("https://node1.example/actor", "https://node1.example/obj/1");
        let json = activity.base.to_json_value();
        assert_eq!(json["type"], "Create");
        assert_eq!(json["actor"], "https://node1.example/actor");
    }

    #[test]
    fn test_activity_with_recipients() {
        let activity = CreateActivity::new("https://a.example/actor", "https://a.example/obj/1")
            .with_to(vec!["https://b.example/inbox".into()])
            .with_cc(vec!["https://c.example/inbox".into()]);
        assert_eq!(activity.base.to.len(), 1);
    }

    #[test]
    fn test_parse_activity_json() {
        let activity = CreateActivity::new("https://a.example/actor", "https://a.example/obj/1");
        let json = activity.base.to_json_value();
        let parsed = parse_activity(&json).unwrap();
        assert_eq!(parsed.activity_type(), "Create");
    }

    #[test]
    fn test_parse_all_activity_types() {
        let types = vec![
            "Create", "Update", "Delete", "Follow", "Accept", "Announce", "Like", "Undo",
        ];
        for atype in &types {
            let json = serde_json::json!({
                "@context": ACTIVITY_STREAMS_CONTEXT,
                "type": atype,
                "actor": "https://a.example/actor",
                "object": "https://a.example/obj/1",
                "published": "2024-01-01T00:00:00Z"
            });
            let parsed = parse_activity(&json).unwrap();
            assert_eq!(parsed.activity_type(), *atype);
        }
    }

    #[test]
    fn test_parse_invalid_activity() {
        assert!(parse_activity(&serde_json::json!({"type": "Unknown"})).is_err());
    }

    #[test]
    fn test_activity_stream_format() {
        let activity = CreateActivity::new("https://a.example/actor", "https://a.example/obj/1");
        let json = activity.base.to_json_value();
        assert!(json["id"].as_str().unwrap().contains('#'));
        assert!(json["@context"].as_str().is_some());
    }
}
