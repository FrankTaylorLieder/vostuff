use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    api::{
        models::{ErrorResponse, LoginRequest, LoginResponse, Organization, UserInfo},
        state::AppState,
    },
    auth::{PasswordHasher, TokenManager},
};

/// User login endpoint
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Always return same error message to prevent user enumeration
    let invalid_credentials_error = || {
        (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
                message: "Invalid credentials".to_string(),
            }),
        )
    };

    // Get user by identity
    let user_row = sqlx::query_as::<_, (uuid::Uuid, String, String, Option<String>, Vec<String>)>(
        "SELECT id, name, identity, password_hash, roles FROM users WHERE identity = $1"
    )
    .bind(&req.identity)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    let (user_id, user_name, user_identity, password_hash_opt, user_roles) = match user_row {
        Some(user) => user,
        None => return Err(invalid_credentials_error()),
    };

    // Check if user has password authentication enabled
    let password_hash = match password_hash_opt {
        Some(hash) => hash,
        None => return Err(invalid_credentials_error()),
    };

    // Verify password
    let is_valid = PasswordHasher::verify_password(&req.password, &password_hash)
        .map_err(internal_error)?;

    if !is_valid {
        return Err(invalid_credentials_error());
    }

    // Get user's organizations
    let org_rows = sqlx::query_as::<_, Organization>(
        "SELECT o.id, o.name, o.description, o.created_at, o.updated_at
         FROM organizations o
         INNER JOIN user_organizations uo ON o.id = uo.organization_id
         WHERE uo.user_id = $1
         ORDER BY o.name"
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    let org_ids: Vec<Uuid> = org_rows.iter().map(|org| org.id).collect();

    // Generate JWT token
    let token_manager = TokenManager::new(&state.jwt_secret);
    let token = token_manager
        .generate_token(user_id, user_identity.clone(), user_roles.clone(), org_ids, 24) // 24 hour expiry
        .map_err(internal_error)?;

    let response = LoginResponse {
        token,
        expires_in: 24 * 60 * 60, // 24 hours in seconds
        user: UserInfo {
            id: user_id,
            name: user_name,
            identity: user_identity,
            roles: user_roles,
            organizations: org_rows,
        },
    };

    Ok(Json(response))
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