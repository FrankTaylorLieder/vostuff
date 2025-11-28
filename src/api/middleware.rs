use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};

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
            request.extensions_mut().insert(AuthContext::unauthenticated());
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

/// Middleware that requires admin access - returns 403 if not admin
/// For now, we consider all authenticated users as admin since there's no role system yet
pub async fn require_admin_middleware(
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

    // For now, all authenticated users have admin access
    // In the future, we could check roles/permissions here

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
        headers.insert(
            header::AUTHORIZATION,
            HeaderValue::from_static("abc123"),
        );

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