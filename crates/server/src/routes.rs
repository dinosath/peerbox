use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use bytes::Bytes;
use common::ObjectId;
use objects::FileObject;
use objects::Object as ObjectTrait;
use serde_json::Value;

use crate::state::AppState;
use crate::types::{ObjectListResponse, ObjectSummary, UploadRequest, WebfingerQuery};

pub async fn health(State(state): State<Arc<AppState>>) -> Result<Json<Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "ok",
        "node_id": state.node_id,
        "version": env!("CARGO_PKG_VERSION"),
    })))
}

pub async fn get_object(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    let object_id = ObjectId::from(id);
    let stored = state
        .application
        .get_stored_object(&object_id)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match stored {
        Some(obj) => Ok(Json(obj.data)),
        None => Err(StatusCode::NOT_FOUND),
    }
}

pub async fn list_objects(
    State(state): State<Arc<AppState>>,
) -> Result<Json<ObjectListResponse>, StatusCode> {
    let objects = state
        .application
        .list_objects()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let summaries: Vec<ObjectSummary> = objects
        .into_iter()
        .map(|obj| ObjectSummary {
            id: obj.id,
            object_type: obj.object_type,
            created_at: obj.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(ObjectListResponse { objects: summaries }))
}

pub async fn get_manifest(Path(id): Path<String>) -> Result<Json<Value>, StatusCode> {
    let manifest_json = serde_json::json!({
        "id": id,
        "status": "not_found",
        "message": "Manifest retrieval not yet implemented"
    });
    Ok(Json(manifest_json))
}

pub async fn activitypub_inbox(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Result<StatusCode, StatusCode> {
    let activity = federation::activities::parse_activity(&body).map_err(|e| {
        tracing::warn!("invalid activity: {}", e);
        StatusCode::BAD_REQUEST
    })?;

    tracing::info!(
        "received activity type={} actor={}",
        activity.activity_type(),
        activity.actor()
    );

    let _ = state;

    Ok(StatusCode::ACCEPTED)
}

pub async fn webfinger(Query(params): Query<WebfingerQuery>) -> Result<Json<Value>, StatusCode> {
    let query = params.resource.trim_start_matches("acct:");
    let parts: Vec<&str> = query.splitn(2, '@').collect();
    if parts.len() != 2 {
        return Err(StatusCode::BAD_REQUEST);
    }
    let user = parts[0];
    let domain = parts[1];

    if user.is_empty() || domain.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let response = serde_json::json!({
        "subject": format!("acct:{user}@{domain}"),
        "aliases": [],
        "links": [{
            "rel": "self",
            "type": "application/activity+json",
            "href": format!("https://{domain}/actor/{user}")
        }]
    });

    Ok(Json(response))
}

pub async fn upload_file(
    State(state): State<Arc<AppState>>,
    Json(body): Json<UploadRequest>,
) -> Result<(StatusCode, Json<Value>), StatusCode> {
    let file = FileObject::new(body.name.clone(), body.size, body.mime_type.clone());
    let data = Bytes::from(body.data.into_bytes());
    let id = file.id().clone();

    let object_service = state.application.object_service();
    object_service
        .create_file_with_data(file, data)
        .await
        .map_err(|e| {
            tracing::error!("failed to create file object: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    Ok((
        StatusCode::CREATED,
        Json(serde_json::json!({
            "id": id.0,
            "name": body.name,
            "size": body.size,
        })),
    ))
}
