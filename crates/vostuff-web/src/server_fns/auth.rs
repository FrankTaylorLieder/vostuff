use leptos::server_fn::error::NoCustomError;
use leptos::*;
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
    use axum::http::HeaderValue;
    use leptos_axum::ResponseOptions;

    // Get API base URL from environment
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

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
        .map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!("API request failed: {}", e))
        })?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::<NoCustomError>::ServerError(format!(
            "Login failed: {}",
            error_text
        )));
    }

    // Try to parse as LoginResponse first (direct login)
    let body = response.text().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to read response: {}", e))
    })?;

    // Try to deserialize as LoginResponse
    if let Ok(login_resp) = serde_json::from_str::<LoginResponse>(&body) {
        // Set the JWT token in HTTP-only cookie
        let response_options = expect_context::<ResponseOptions>();
        let cookie = format!(
            "auth_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
            login_resp.token, login_resp.expires_in
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
        .map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
        })
}

// Server function to handle organization selection
#[server(SelectOrganization, "/api")]
pub async fn select_organization(
    follow_on_token: String,
    organization_id: Uuid,
) -> Result<LoginResponse, ServerFnError<NoCustomError>> {
    use axum::http::HeaderValue;
    use leptos_axum::ResponseOptions;

    // Get API base URL from environment
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

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
        .map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!("API request failed: {}", e))
        })?;

    let status = response.status();

    if !status.is_success() {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(ServerFnError::<NoCustomError>::ServerError(format!(
            "Org selection failed: {}",
            error_text
        )));
    }

    let login_resp: LoginResponse = response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })?;

    // Set the JWT token in HTTP-only cookie
    let response_options = expect_context::<ResponseOptions>();
    let cookie = format!(
        "auth_token={}; Path=/; HttpOnly; SameSite=Lax; Max-Age={}",
        login_resp.token, login_resp.expires_in
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
    use axum::http::header::COOKIE;
    use leptos_axum::extract;

    // Get cookies from request headers
    let headers = extract::<axum::http::HeaderMap>().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to extract headers: {}", e))
    })?;

    // Parse cookies to find auth_token
    let auth_token = headers
        .get(COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .map(|c| c.trim())
                .find(|c| c.starts_with("auth_token="))
                .map(|c| c.trim_start_matches("auth_token=").to_string())
        });

    // If no auth token, return None
    let token = match auth_token {
        Some(t) => t,
        None => return Ok(None),
    };

    // Get API base URL from environment
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Call the /api/auth/me endpoint to get user info
    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/api/auth/me", api_base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!("API request failed: {}", e))
        })?;

    // If unauthorized (401), return None (user not logged in or token invalid)
    if response.status() == 401 {
        return Ok(None);
    }

    // If other error, return error
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(ServerFnError::<NoCustomError>::ServerError(format!(
            "Failed to get user info: {} - {}",
            status, body
        )));
    }

    // Parse the response - it should match our UserInfo structure
    // but we need to map the API's Organization type to our OrganizationInfo
    #[derive(serde::Deserialize)]
    struct ApiUserInfo {
        id: Uuid,
        name: String,
        identity: String,
        organization: ApiOrganization,
        roles: Vec<String>,
    }

    #[derive(serde::Deserialize)]
    struct ApiOrganization {
        id: Uuid,
        name: String,
        #[allow(dead_code)]
        description: Option<String>,
        #[allow(dead_code)]
        created_at: chrono::DateTime<chrono::Utc>,
        #[allow(dead_code)]
        updated_at: chrono::DateTime<chrono::Utc>,
    }

    let api_user_info: ApiUserInfo = response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })?;

    // Convert to our UserInfo type
    let user_info = UserInfo {
        id: api_user_info.id,
        name: api_user_info.name,
        identity: api_user_info.identity,
        organization: OrganizationInfo {
            id: api_user_info.organization.id,
            name: api_user_info.organization.name,
        },
        roles: api_user_info.roles,
    };

    Ok(Some(user_info))
}

// Server function to handle logout
#[server(Logout, "/api")]
pub async fn logout() -> Result<(), ServerFnError<NoCustomError>> {
    use axum::http::HeaderValue;
    use leptos_axum::ResponseOptions;

    // Clear the auth cookie
    let response_options = expect_context::<ResponseOptions>();
    let cookie = "auth_token=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0";
    response_options.insert_header(
        axum::http::header::SET_COOKIE,
        HeaderValue::from_str(cookie).unwrap(),
    );

    Ok(())
}
