use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::api::{
    models::{
        AddUserToOrgRequest, CreateUserRequest, ErrorResponse, Organization,
        UpdateUserOrgRolesRequest, UpdateUserRequest, User, UserOrganization,
    },
    state::AppState,
};
use crate::auth::PasswordHasher;

/// List all users
#[utoipa::path(
    get,
    path = "/api/admin/users",
    responses(
        (status = 200, description = "List of users", body = Vec<User>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-users"
)]
pub async fn list_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<User>>, (StatusCode, Json<ErrorResponse>)> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, name, identity, password_hash, created_at, updated_at FROM users ORDER BY name",
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(users))
}

/// Get a single user by ID
#[utoipa::path(
    get,
    path = "/api/admin/users/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User details", body = User),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-users"
)]
pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, name, identity, password_hash, created_at, updated_at FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "User not found".to_string(),
            }),
        )),
    }
}

/// Create a new user
#[utoipa::path(
    post,
    path = "/api/admin/users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = User),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-users"
)]
pub async fn create_user(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<User>), (StatusCode, Json<ErrorResponse>)> {
    // Hash password if provided
    let password_hash = if let Some(password) = &req.password {
        Some(PasswordHasher::hash_password(password).map_err(internal_error)?)
    } else {
        None
    };

    let user = sqlx::query_as::<_, User>(
        "INSERT INTO users (name, identity, password_hash) VALUES ($1, $2, $3)
         RETURNING id, name, identity, password_hash, created_at, updated_at",
    )
    .bind(&req.name)
    .bind(&req.identity)
    .bind(&password_hash)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(user)))
}

/// Update an existing user
#[utoipa::path(
    patch,
    path = "/api/admin/users/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = User),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-users"
)]
pub async fn update_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<User>, (StatusCode, Json<ErrorResponse>)> {
    // Hash password if provided
    let password_hash = if let Some(password) = &req.password {
        Some(PasswordHasher::hash_password(password).map_err(internal_error)?)
    } else {
        None
    };

    // Build dynamic update query
    let mut query = String::from("UPDATE users SET updated_at = NOW()");
    let mut param_num = 2;

    if req.name.is_some() {
        query.push_str(&format!(", name = ${}", param_num));
        param_num += 1;
    }
    if req.identity.is_some() {
        query.push_str(&format!(", identity = ${}", param_num));
        param_num += 1;
    }
    if req.password.is_some() {
        query.push_str(&format!(", password_hash = ${}", param_num));
    }

    query.push_str(
        " WHERE id = $1 RETURNING id, name, identity, password_hash, created_at, updated_at",
    );

    let mut query_builder = sqlx::query_as::<_, User>(&query).bind(user_id);

    if let Some(name) = &req.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(identity) = &req.identity {
        query_builder = query_builder.bind(identity);
    }
    if req.password.is_some() {
        query_builder = query_builder.bind(&password_hash);
    }

    let user = query_builder
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    match user {
        Some(user) => Ok(Json(user)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "User not found".to_string(),
            }),
        )),
    }
}

/// Delete a user
#[utoipa::path(
    delete,
    path = "/api/admin/users/{user_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-users"
)]
pub async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "User not found".to_string(),
            }),
        ))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

/// List organizations for a user
#[utoipa::path(
    get,
    path = "/api/admin/users/{user_id}/organizations",
    params(
        ("user_id" = Uuid, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "List of organizations for user", body = Vec<Organization>),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-users"
)]
pub async fn list_user_organizations(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<Vec<Organization>>, (StatusCode, Json<ErrorResponse>)> {
    // First check if user exists
    let user_exists = sqlx::query("SELECT id FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    if user_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "User not found".to_string(),
            }),
        ));
    }

    let organizations = sqlx::query_as::<_, Organization>(
        "SELECT o.id, o.name, o.description, o.created_at, o.updated_at
         FROM organizations o
         INNER JOIN user_organizations uo ON o.id = uo.organization_id
         WHERE uo.user_id = $1
         ORDER BY o.name",
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(organizations))
}

/// Add user to organization
#[utoipa::path(
    post,
    path = "/api/admin/users/{user_id}/organizations/{org_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = AddUserToOrgRequest,
    responses(
        (status = 201, description = "User added to organization successfully", body = UserOrganization),
        (status = 404, description = "User or organization not found", body = ErrorResponse),
        (status = 409, description = "User already in organization", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-users"
)]
pub async fn add_user_to_organization(
    State(state): State<AppState>,
    Path((user_id, org_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<AddUserToOrgRequest>,
) -> Result<(StatusCode, Json<UserOrganization>), (StatusCode, Json<ErrorResponse>)> {
    // Verify user and organization exist
    let user_exists = sqlx::query("SELECT id FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    if user_exists.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "User not found".to_string(),
            }),
        ));
    }

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

    // Prepare roles - default to USER if not provided
    let roles: Vec<String> = req
        .roles
        .map(|r| r.iter().map(|role| role.as_str().to_string()).collect())
        .unwrap_or_else(|| vec!["USER".to_string()]);

    // Add user to organization
    let result = sqlx::query_as::<_, UserOrganization>(
        "INSERT INTO user_organizations (user_id, organization_id, roles) VALUES ($1, $2, $3)
         RETURNING user_id, organization_id, roles, created_at",
    )
    .bind(user_id)
    .bind(org_id)
    .bind(&roles)
    .fetch_one(&state.pool)
    .await;

    match result {
        Ok(user_org) => Ok((StatusCode::CREATED, Json(user_org))),
        Err(sqlx::Error::Database(db_err)) if db_err.is_unique_violation() => Err((
            StatusCode::CONFLICT,
            Json(ErrorResponse {
                error: "conflict".to_string(),
                message: "User already in organization".to_string(),
            }),
        )),
        Err(err) => Err(internal_error(err)),
    }
}

/// Update user roles in organization
#[utoipa::path(
    patch,
    path = "/api/admin/users/{user_id}/organizations/{org_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = UpdateUserOrgRolesRequest,
    responses(
        (status = 200, description = "User roles updated successfully", body = UserOrganization),
        (status = 404, description = "User not in organization", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-users"
)]
pub async fn update_user_org_roles(
    State(state): State<AppState>,
    Path((user_id, org_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateUserOrgRolesRequest>,
) -> Result<Json<UserOrganization>, (StatusCode, Json<ErrorResponse>)> {
    // Convert UserRole to strings
    let roles: Vec<String> = req
        .roles
        .iter()
        .map(|role| role.as_str().to_string())
        .collect();

    // Update user roles in organization
    let result = sqlx::query_as::<_, UserOrganization>(
        "UPDATE user_organizations
         SET roles = $3
         WHERE user_id = $1 AND organization_id = $2
         RETURNING user_id, organization_id, roles, created_at",
    )
    .bind(user_id)
    .bind(org_id)
    .bind(&roles)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    match result {
        Some(user_org) => Ok(Json(user_org)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "User not in organization".to_string(),
            }),
        )),
    }
}

/// Remove user from organization
#[utoipa::path(
    delete,
    path = "/api/admin/users/{user_id}/organizations/{org_id}",
    params(
        ("user_id" = Uuid, Path, description = "User ID"),
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    responses(
        (status = 204, description = "User removed from organization successfully"),
        (status = 404, description = "User not in organization", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "admin-users"
)]
pub async fn remove_user_from_organization(
    State(state): State<AppState>,
    Path((user_id, org_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result =
        sqlx::query("DELETE FROM user_organizations WHERE user_id = $1 AND organization_id = $2")
            .bind(user_id)
            .bind(org_id)
            .execute(&state.pool)
            .await
            .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "User not in organization".to_string(),
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
