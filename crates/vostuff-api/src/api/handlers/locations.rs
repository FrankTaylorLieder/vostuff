use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::api::{
    models::{CreateLocationRequest, ErrorResponse, Location},
    state::AppState,
};

/// List all locations for an organization
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/locations",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of locations", body = Vec<Location>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "locations"
)]
pub async fn list_locations(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<Location>>, (StatusCode, Json<ErrorResponse>)> {
    let locations = sqlx::query_as::<_, Location>(
        "SELECT id, organization_id, name, created_at, updated_at
         FROM locations WHERE organization_id = $1 ORDER BY name"
    )
    .bind(org_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(locations))
}

/// Create a new location
#[utoipa::path(
    post,
    path = "/api/organizations/{org_id}/locations",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = CreateLocationRequest,
    responses(
        (status = 201, description = "Location created successfully", body = Location),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "locations"
)]
pub async fn create_location(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateLocationRequest>,
) -> Result<(StatusCode, Json<Location>), (StatusCode, Json<ErrorResponse>)> {
    let location = sqlx::query_as::<_, Location>(
        "INSERT INTO locations (organization_id, name) VALUES ($1, $2)
         RETURNING id, organization_id, name, created_at, updated_at"
    )
    .bind(org_id)
    .bind(&req.name)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(location)))
}

/// Delete a location
#[utoipa::path(
    delete,
    path = "/api/organizations/{org_id}/locations/{location_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("location_id" = Uuid, Path, description = "Location ID")
    ),
    responses(
        (status = 204, description = "Location deleted successfully"),
        (status = 404, description = "Location not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "locations"
)]
pub async fn delete_location(
    State(state): State<AppState>,
    Path((org_id, location_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM locations WHERE id = $1 AND organization_id = $2")
        .bind(location_id)
        .bind(org_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "Location not found".to_string(),
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