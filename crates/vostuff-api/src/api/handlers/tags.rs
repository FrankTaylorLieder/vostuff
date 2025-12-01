use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::api::{
    models::{CreateTagRequest, ErrorResponse, Tag},
    state::AppState,
};

/// List all tags for an organization
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/tags",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of tags", body = Vec<Tag>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tags"
)]
pub async fn list_tags(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<Tag>>, (StatusCode, Json<ErrorResponse>)> {
    let tags = sqlx::query_as::<_, Tag>(
        "SELECT organization_id, name, created_at
         FROM tags WHERE organization_id = $1 ORDER BY name"
    )
    .bind(org_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(tags))
}

/// Create a new tag
#[utoipa::path(
    post,
    path = "/api/organizations/{org_id}/tags",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = CreateTagRequest,
    responses(
        (status = 201, description = "Tag created successfully", body = Tag),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tags"
)]
pub async fn create_tag(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateTagRequest>,
) -> Result<(StatusCode, Json<Tag>), (StatusCode, Json<ErrorResponse>)> {
    let tag = sqlx::query_as::<_, Tag>(
        "INSERT INTO tags (organization_id, name) VALUES ($1, $2)
         RETURNING organization_id, name, created_at"
    )
    .bind(org_id)
    .bind(&req.name)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(tag)))
}

/// Delete a tag
#[utoipa::path(
    delete,
    path = "/api/organizations/{org_id}/tags/{tag_name}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("tag_name" = String, Path, description = "Tag name")
    ),
    responses(
        (status = 204, description = "Tag deleted successfully"),
        (status = 404, description = "Tag not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "tags"
)]
pub async fn delete_tag(
    State(state): State<AppState>,
    Path((org_id, tag_name)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM tags WHERE organization_id = $1 AND name = $2")
        .bind(org_id)
        .bind(&tag_name)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "Tag not found".to_string(),
            }),
        ))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

fn internal_error<E: std::fmt::Display>(err: E) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse {
            error: "internal_error".to_string(),
            message: err.to_string(),
        }),
    )
}