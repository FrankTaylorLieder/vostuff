use leptos::*;
use leptos::server_fn::error::NoCustomError;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Response types matching the API
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: i64,
    pub user: UserInfo,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrgSelectionResponse {
    pub organizations: Vec<OrganizationWithRoles>,
    pub follow_on_token: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub identity: String,
    pub organization: OrganizationInfo,
    pub roles: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationInfo {
    pub id: Uuid,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationWithRoles {
    pub id: Uuid,
    pub name: String,
    pub roles: Vec<String>,
}

// Server function to handle login
#[server(Login, "/api")]
pub async fn login(
    identity: String,
    password: String,
    organization_id: Option<Uuid>,
) -> Result<Result<LoginResponse, OrgSelectionResponse>, ServerFnError<NoCustomError>> {
    use leptos_axum::ResponseOptions;
    use axum::http::HeaderValue;

    // Get API base URL from environment
    let api_base_url = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Prepare login request
    let login_req = serde_json::json!({
        "identity": identity,
        "password": password,
        "organization_id": organization_id,
    });

    // Call the REST API
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/auth/login", api_base_url))
        .json(&login_req)
        .send()
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("API request failed: {}", e)))?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::<NoCustomError>::ServerError(format!("Login failed: {}", error_text)));
    }

    // Try to parse as LoginResponse first (direct login)
    let body = response.text().await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Failed to read response: {}", e)))?;

    // Try to deserialize as LoginResponse
    if let Ok(login_resp) = serde_json::from_str::<LoginResponse>(&body) {
        // Set the JWT token in HTTP-only cookie
        let response_options = expect_context::<ResponseOptions>();
        let cookie = format!(
            "auth_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
            login_resp.token,
            login_resp.expires_in
        );
        response_options.insert_header(
            axum::http::header::SET_COOKIE,
            HeaderValue::from_str(&cookie).unwrap(),
        );

        return Ok(Ok(login_resp));
    }

    // Otherwise, try to parse as OrgSelectionResponse
    serde_json::from_str::<OrgSelectionResponse>(&body)
        .map(Err)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e)))
}

// Server function to handle organization selection
#[server(SelectOrganization, "/api")]
pub async fn select_organization(
    follow_on_token: String,
    organization_id: Uuid,
) -> Result<LoginResponse, ServerFnError<NoCustomError>> {
    use leptos_axum::ResponseOptions;
    use axum::http::HeaderValue;

    // Get API base URL from environment
    let api_base_url = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Prepare request
    let select_req = serde_json::json!({
        "follow_on_token": follow_on_token,
        "organization_id": organization_id,
    });

    // Call the REST API
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/api/auth/select-org", api_base_url))
        .json(&select_req)
        .send()
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("API request failed: {}", e)))?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::<NoCustomError>::ServerError(format!("Org selection failed: {}", error_text)));
    }

    let login_resp: LoginResponse = response.json().await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e)))?;

    // Set the JWT token in HTTP-only cookie
    let response_options = expect_context::<ResponseOptions>();
    let cookie = format!(
        "auth_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        login_resp.token,
        login_resp.expires_in
    );
    response_options.insert_header(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_str(&cookie).unwrap(),
    );

    Ok(login_resp)
}

// Server function to get current authenticated user
#[server(GetCurrentUser, "/api")]
pub async fn get_current_user() -> Result<Option<UserInfo>, ServerFnError<NoCustomError>> {
    use leptos_axum::extract;
    use axum::http::header::COOKIE;
    use vostuff_core::auth::TokenManager;

    // Get the JWT secret from environment
    let jwt_secret = std::env::var("JWT_SECRET")
        .map_err(|_| ServerFnError::<NoCustomError>::ServerError("JWT_SECRET not configured".to_string()))?;

    // Get cookies from request headers
    let headers = extract::<axum::http::HeaderMap>().await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Failed to extract headers: {}", e)))?;

    // Parse cookies to find auth_token
    let auth_token = headers
        .get(COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .map(|c| c.trim())
                .find(|c| c.starts_with("auth_token="))
                .map(|c| c.trim_start_matches("auth_token="))
        });

    // If no auth token, return None
    let token = match auth_token {
        Some(t) => t,
        None => return Ok(None),
    };

    // Validate and decode the JWT
    let token_manager = TokenManager::new(&jwt_secret);
    let claims = token_manager.validate_token(token)
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Invalid token: {}", e)))?;

    // Get API base URL from environment
    let api_base_url = std::env::var("API_BASE_URL")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Fetch user details from the API (to get the latest name)
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/users/{}", api_base_url, claims.sub))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("API request failed: {}", e)))?;

    if !response.status().is_success() {
        return Err(ServerFnError::<NoCustomError>::ServerError("Failed to fetch user details".to_string()));
    }

    // Parse the user response
    #[derive(serde::Deserialize)]
    struct UserResponse {
        id: Uuid,
        name: String,
        identity: String,
    }

    let user_resp: UserResponse = response.json().await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse user response: {}", e)))?;

    // Get organization name from the API
    let org_response = client
        .get(format!("{}/api/organizations/{}", api_base_url, claims.organization_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("API request failed: {}", e)))?;

    if !org_response.status().is_success() {
        return Err(ServerFnError::<NoCustomError>::ServerError("Failed to fetch organization details".to_string()));
    }

    #[derive(serde::Deserialize)]
    struct OrgResponse {
        id: Uuid,
        name: String,
    }

    let org_resp: OrgResponse = org_response.json().await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse org response: {}", e)))?;

    // Construct UserInfo
    let user_info = UserInfo {
        id: user_resp.id,
        name: user_resp.name,
        identity: user_resp.identity,
        organization: OrganizationInfo {
            id: org_resp.id,
            name: org_resp.name,
        },
        roles: claims.roles,
    };

    Ok(Some(user_info))
}

// Server function to handle logout
#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError<NoCustomError>> {
    use leptos_axum::ResponseOptions;
    use axum::http::HeaderValue;

    // Clear the auth cookie
    let response_options = expect_context::<ResponseOptions>();
    let cookie = "auth_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    response_options.insert_header(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_str(cookie).unwrap(),
    );

    Ok(())
}
