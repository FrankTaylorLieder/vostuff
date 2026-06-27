use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

use crate::{
    api::{models::ErrorResponse, state::AppState},
    auth::{AuthContext, TokenManager},
};

/// Authentication middleware that extracts JWT token from Authorization header
/// and validates it, adding AuthContext to request extensions
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let headers = request.headers();

    // Extract token from Authorization header
    let token = match extract_token_from_headers(headers) {
        Some(token) => token,
        None => {
            // No token provided - set unauthenticated context
            request
                .extensions_mut()
                .insert(AuthContext::unauthenticated());
            return Ok(next.run(request).await);
        }
    };

    // Validate token
    let token_manager = TokenManager::new(&state.jwt_secret);
    match token_manager.validate_token(&token) {
        Ok(claims) => {
            // Token valid - set authenticated context
            let auth_context = AuthContext::from_claims(claims);
            request.extensions_mut().insert(auth_context);
            Ok(next.run(request).await)
        }
        Err(_) => {
            // Token invalid - return unauthorized error
            Err((
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponse {
                    error: "unauthorized".to_string(),
                    message: "Invalid or expired token".to_string(),
                }),
            ))
        }
    }
}

/// Extract JWT token from Authorization header
/// Supports both "Bearer <token>" and just "<token>" formats
fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get(header::AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;

    if auth_str.starts_with("Bearer ") {
        Some(auth_str.strip_prefix("Bearer ").unwrap().to_string())
    } else {
        // Support headerless token for simplicity in testing
        Some(auth_str.to_string())
    }
}

/// Middleware that requires authentication - returns 401 if not authenticated
pub async fn require_auth_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    // Check if user is authenticated
    let auth_context = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .unwrap_or_else(AuthContext::unauthenticated);

    if !auth_context.is_authenticated() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponse {
                error: "unauthorized".to_string(),
                message: "Authentication required".to_string(),
            }),
        ));
    }

    Ok(next.run(request).await)
}

/// Helpers for building error responses
fn unauthorized() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::UNAUTHORIZED,
        Json(ErrorResponse {
            error: "unauthorized".to_string(),
            message: "Authentication required".to_string(),
        }),
    )
}

fn forbidden(message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            error: "forbidden".to_string(),
            message: message.to_string(),
        }),
    )
}

/// Middleware for org-scoped routes (`/organizations/:org_id/*`). Requires the caller to
/// be authenticated and to have selected the same org as the one in the path. Returns 401
/// if unauthenticated, 403 if authenticated but not a member of the path org.
pub async fn org_access_middleware(
    Path(params): Path<HashMap<String, String>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let auth_context = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .unwrap_or_else(AuthContext::unauthenticated);

    if !auth_context.is_authenticated() {
        return Err(unauthorized());
    }

    let org_id = params
        .get("org_id")
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or_else(|| forbidden("Invalid organization id"))?;

    if !auth_context.has_org_access(org_id) {
        return Err(forbidden("You do not have access to this organization"));
    }

    Ok(next.run(request).await)
}

/// Middleware for system administration routes (`/admin/*`). Requires the caller to be a
/// system super-admin: authenticated with the SYSTEM org selected and holding ADMIN there.
pub async fn system_admin_middleware(
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<ErrorResponse>)> {
    let auth_context = request
        .extensions()
        .get::<AuthContext>()
        .cloned()
        .unwrap_or_else(AuthContext::unauthenticated);

    if !auth_context.is_authenticated() {
        return Err(unauthorized());
    }

    if !auth_context.is_system_admin() {
        return Err(forbidden("System administrator access required"));
    }

    Ok(next.run(request).await)
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_token_bearer() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("Bearer abc123"),
        );

        let token = extract_token_from_headers(&headers);
        assert_eq!(token, Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_token_direct() {
        let mut headers = HeaderMap::new();
        headers.insert(header::AUTHORIZATION, HeaderValue::from_static("abc123"));

        let token = extract_token_from_headers(&headers);
        assert_eq!(token, Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_token_missing() {
        let headers = HeaderMap::new();
        let token = extract_token_from_headers(&headers);
        assert_eq!(token, None);
    }
}
