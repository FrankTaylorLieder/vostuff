use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::api::{
    models::{Collection, CreateCollectionRequest, ErrorResponse},
    state::AppState,
};

/// List all collections for an organization
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/collections",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of collections", body = Vec<Collection>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "collections"
)]
pub async fn list_collections(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<Collection>>, (StatusCode, Json<ErrorResponse>)> {
    let collections = sqlx::query_as::<_, Collection>(
        "SELECT id, organization_id, name, description, notes, created_at, updated_at
         FROM collections WHERE organization_id = $1 ORDER BY name"
    )
    .bind(org_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(collections))
}

/// Create a new collection
#[utoipa::path(
    post,
    path = "/api/organizations/{org_id}/collections",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = CreateCollectionRequest,
    responses(
        (status = 201, description = "Collection created successfully", body = Collection),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "collections"
)]
pub async fn create_collection(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<(StatusCode, Json<Collection>), (StatusCode, Json<ErrorResponse>)> {
    let collection = sqlx::query_as::<_, Collection>(
        "INSERT INTO collections (organization_id, name, description, notes)
         VALUES ($1, $2, $3, $4)
         RETURNING id, organization_id, name, description, notes, created_at, updated_at"
    )
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.notes)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(collection)))
}

/// Delete a collection
#[utoipa::path(
    delete,
    path = "/api/organizations/{org_id}/collections/{collection_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("collection_id" = Uuid, Path, description = "Collection ID")
    ),
    responses(
        (status = 204, description = "Collection deleted successfully"),
        (status = 404, description = "Collection not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "collections"
)]
pub async fn delete_collection(
    State(state): State<AppState>,
    Path((org_id, collection_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM collections WHERE id = $1 AND organization_id = $2")
        .bind(collection_id)
        .bind(org_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "Collection not found".to_string(),
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