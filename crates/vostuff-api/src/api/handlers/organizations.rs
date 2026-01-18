use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::api::{
    models::{CreateOrganizationRequest, ErrorResponse, Organization, UpdateOrganizationRequest},
    state::AppState,
};

/// List all organizations
#[utoipa::path(
    get,
    path = "/api/admin/organizations",
    responses(
        (status = 200, description = "List of organizations", body = Vec<Organization>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-organizations"
)]
pub async fn list_organizations(
    State(state): State<AppState>,
) -> Result<Json<Vec<Organization>>, (StatusCode, Json<ErrorResponse>)> {
    let organizations = sqlx::query_as::<_, Organization>(
        "SELECT id, name, description, created_at, updated_at FROM organizations ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(organizations))
}

/// Get a single organization by ID
#[utoipa::path(
    get,
    path = "/api/admin/organizations/{org_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "Organization details", body = Organization),
        (status = 404, description = "Organization not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-organizations"
)]
pub async fn get_organization(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Organization>, (StatusCode, Json<ErrorResponse>)> {
    let organization = sqlx::query_as::<_, Organization>(
        "SELECT id, name, description, created_at, updated_at FROM organizations WHERE id = $1",
    )
    .bind(org_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    match organization {
        Some(org) => Ok(Json(org)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "Organization not found".to_string(),
            }),
        )),
    }
}

/// Create a new organization
#[utoipa::path(
    post,
    path = "/api/admin/organizations",
    request_body = CreateOrganizationRequest,
    responses(
        (status = 201, description = "Organization created successfully", body = Organization),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-organizations"
)]
pub async fn create_organization(
    State(state): State<AppState>,
    Json(req): Json<CreateOrganizationRequest>,
) -> Result<(StatusCode, Json<Organization>), (StatusCode, Json<ErrorResponse>)> {
    let organization = sqlx::query_as::<_, Organization>(
        "INSERT INTO organizations (name, description) VALUES ($1, $2)
         RETURNING id, name, description, created_at, updated_at",
    )
    .bind(&req.name)
    .bind(&req.description)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(organization)))
}

/// Update an existing organization
#[utoipa::path(
    patch,
    path = "/api/admin/organizations/{org_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = UpdateOrganizationRequest,
    responses(
        (status = 200, description = "Organization updated successfully", body = Organization),
        (status = 404, description = "Organization not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-organizations"
)]
pub async fn update_organization(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<UpdateOrganizationRequest>,
) -> Result<Json<Organization>, (StatusCode, Json<ErrorResponse>)> {
    // Build dynamic update query
    let mut query = String::from("UPDATE organizations SET updated_at = NOW()");
    let mut param_num = 2;

    if req.name.is_some() {
        query.push_str(&format!(", name = ${}", param_num));
        param_num += 1;
    }
    if req.description.is_some() {
        query.push_str(&format!(", description = ${}", param_num));
    }

    query.push_str(" WHERE id = $1 RETURNING id, name, description, created_at, updated_at");

    let mut query_builder = sqlx::query_as::<_, Organization>(&query).bind(org_id);

    if let Some(name) = &req.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(description) = &req.description {
        query_builder = query_builder.bind(description);
    }

    let organization = query_builder
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    match organization {
        Some(org) => Ok(Json(org)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "Organization not found".to_string(),
            }),
        )),
    }
}

/// Delete an organization
#[utoipa::path(
    delete,
    path = "/api/admin/organizations/{org_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 204, description = "Organization deleted successfully"),
        (status = 404, description = "Organization not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-organizations"
)]
pub async fn delete_organization(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM organizations WHERE id = $1")
        .bind(org_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "Organization not found".to_string(),
            }),
        ))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

/// List users in an organization
#[utoipa::path(
    get,
    path = "/api/admin/organizations/{org_id}/users",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 200, description = "List of users in organization", body = Vec<crate::api::models::User>),
        (status = 404, description = "Organization not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-organizations"
)]
pub async fn list_organization_users(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<crate::api::models::User>>, (StatusCode, Json<ErrorResponse>)> {
    // First check if organization exists
    let org_exists = sqlx::query("SELECT id FROM organizations WHERE id = $1")
        .bind(org_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    if org_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "Organization not found".to_string(),
            }),
        ));
    }

    let users = sqlx::query_as::<_, crate::api::models::User>(
        "SELECT u.id, u.name, u.identity, u.password_hash, u.created_at, u.updated_at
         FROM users u
         INNER JOIN user_organizations uo ON u.id = uo.user_id
         WHERE uo.organization_id = $1
         ORDER BY u.name",
    )
    .bind(org_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(users))
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
