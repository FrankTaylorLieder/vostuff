use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use uuid::Uuid;

use crate::{
    api::{
        models::{
            ErrorResponse, LoginRequest, LoginResponse, OrgSelectionResponse,
            Organization, OrganizationWithRoles, SelectOrgRequest, UserInfo,
        },
        state::AppState,
    },
    auth::{PasswordHasher, TokenManager},
};

/// User login endpoint with optional organization selection
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful or org selection required", body = LoginResponse),
        (status = 401, description = "Invalid credentials", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<ErrorResponse>)> {
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

    // Get user by identity (no roles in users table anymore)
    let user_row = sqlx::query_as::<_, (uuid::Uuid, String, String, Option<String>)>(
        "SELECT id, name, identity, password_hash FROM users WHERE identity = $1"
    )
    .bind(&req.identity)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    let (user_id, user_name, user_identity, password_hash_opt) = match user_row {
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

    // Get user's organizations with roles
    let org_rows = sqlx::query_as::<_, (Uuid, String, Option<String>, Vec<String>)>(
        "SELECT o.id, o.name, o.description, uo.roles
         FROM organizations o
         INNER JOIN user_organizations uo ON o.id = uo.organization_id
         WHERE uo.user_id = $1
         ORDER BY o.name"
    )
    .bind(user_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    if org_rows.is_empty() {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponse {
                error: "no_organization".to_string(),
                message: "User is not a member of any organization".to_string(),
            }),
        ));
    }

    let token_manager = TokenManager::new(&state.jwt_secret);

    // If organization_id provided, use it
    if let Some(org_id) = req.organization_id {
        // Find the requested organization
        let org_data = org_rows.iter()
            .find(|(id, _, _, _)| *id == org_id)
            .ok_or_else(|| (
                StatusCode::FORBIDDEN,
                Json(ErrorResponse {
                    error: "invalid_organization".to_string(),
                    message: "User is not a member of the specified organization".to_string(),
                }),
            ))?;

        let (org_id, org_name, org_desc, roles) = org_data;

        // Generate JWT token with selected org
        let token = token_manager
            .generate_token(user_id, user_identity.clone(), *org_id, roles.clone(), 24)
            .map_err(internal_error)?;

        // Get full organization details
        let organization = Organization {
            id: *org_id,
            name: org_name.clone(),
            description: org_desc.clone(),
            created_at: chrono::Utc::now(), // These will be properly loaded in real scenario
            updated_at: chrono::Utc::now(),
        };

        let response = LoginResponse {
            token,
            expires_in: 24 * 60 * 60,
            user: UserInfo {
                id: user_id,
                name: user_name,
                identity: user_identity,
                organization,
                roles: roles.clone(),
            },
        };

        return Ok((StatusCode::OK, Json(serde_json::to_value(response).unwrap())));
    }

    // No org_id provided - check how many orgs user belongs to
    if org_rows.len() == 1 {
        // Auto-select the only organization
        let (org_id, org_name, org_desc, roles) = &org_rows[0];

        let token = token_manager
            .generate_token(user_id, user_identity.clone(), *org_id, roles.clone(), 24)
            .map_err(internal_error)?;

        let organization = Organization {
            id: *org_id,
            name: org_name.clone(),
            description: org_desc.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let response = LoginResponse {
            token,
            expires_in: 24 * 60 * 60,
            user: UserInfo {
                id: user_id,
                name: user_name,
                identity: user_identity,
                organization,
                roles: roles.clone(),
            },
        };

        return Ok((StatusCode::OK, Json(serde_json::to_value(response).unwrap())));
    }

    // Multiple organizations - return org selection response
    let organizations: Vec<OrganizationWithRoles> = org_rows
        .into_iter()
        .map(|(id, name, description, roles)| OrganizationWithRoles {
            id,
            name,
            description,
            roles,
        })
        .collect();

    let follow_on_token = token_manager
        .generate_follow_on_token(user_id, user_identity)
        .map_err(internal_error)?;

    let response = OrgSelectionResponse {
        organizations,
        follow_on_token,
    };

    Ok((StatusCode::OK, Json(serde_json::to_value(response).unwrap())))
}

/// Select organization endpoint for multi-org users
#[utoipa::path(
    post,
    path = "/api/auth/select-org",
    request_body = SelectOrgRequest,
    responses(
        (status = 200, description = "Organization selected", body = LoginResponse),
        (status = 401, description = "Invalid or expired token", body = ErrorResponse),
        (status = 403, description = "Not a member of organization", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "auth"
)]
pub async fn select_org(
    State(state): State<AppState>,
    Json(req): Json<SelectOrgRequest>,
) -> Result<Json<LoginResponse>, (StatusCode, Json<ErrorResponse>)> {
    let token_manager = TokenManager::new(&state.jwt_secret);

    // Validate follow-on token
    let claims = token_manager
        .validate_follow_on_token(&req.follow_on_token)
        .map_err(|_| (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "invalid_token".to_string(),
                message: "Invalid or expired follow-on token".to_string(),
            }),
        ))?;

    // Get user info
    let user_row = sqlx::query_as::<_, (String,)>(
        "SELECT name FROM users WHERE id = $1"
    )
    .bind(claims.sub)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "user_not_found".to_string(),
            message: "User not found".to_string(),
        }),
    ))?;

    let user_name = user_row.0;

    // Verify user is member of selected org and get roles
    let org_data = sqlx::query_as::<_, (String, Option<String>, Vec<String>)>(
        "SELECT o.name, o.description, uo.roles
         FROM organizations o
         INNER JOIN user_organizations uo ON o.id = uo.organization_id
         WHERE uo.user_id = $1 AND o.id = $2"
    )
    .bind(claims.sub)
    .bind(req.organization_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            error: "not_member".to_string(),
            message: "User is not a member of the specified organization".to_string(),
        }),
    ))?;

    let (org_name, org_desc, roles) = org_data;

    // Generate final JWT token
    let token = token_manager
        .generate_token(claims.sub, claims.identity.clone(), req.organization_id, roles.clone(), 24)
        .map_err(internal_error)?;

    let organization = Organization {
        id: req.organization_id,
        name: org_name,
        description: org_desc,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let response = LoginResponse {
        token,
        expires_in: 24 * 60 * 60,
        user: UserInfo {
            id: claims.sub,
            name: user_name,
            identity: claims.identity,
            organization,
            roles,
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
