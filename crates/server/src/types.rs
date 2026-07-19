use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct WebfingerQuery {
    pub resource: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadRequest {
    pub name: String,
    pub size: u64,
    pub mime_type: Option<String>,
    pub data: String,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub node_id: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct ObjectListResponse {
    pub objects: Vec<ObjectSummary>,
}

#[derive(Debug, Serialize)]
pub struct ObjectSummary {
    pub id: String,
    pub object_type: String,
    pub created_at: String,
}
