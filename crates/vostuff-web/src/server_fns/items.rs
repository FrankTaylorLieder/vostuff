use leptos::server_fn::error::NoCustomError;
use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Types matching the API response

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Vinyl,
    Cd,
    Cassette,
    Book,
    Score,
    Electronics,
    Misc,
}

impl ItemType {
    pub fn display_name(&self) -> &'static str {
        match self {
            ItemType::Vinyl => "Vinyl",
            ItemType::Cd => "CD",
            ItemType::Cassette => "Cassette",
            ItemType::Book => "Book",
            ItemType::Score => "Score",
            ItemType::Electronics => "Electronics",
            ItemType::Misc => "Misc",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ItemState {
    Current,
    Loaned,
    Missing,
    Disposed,
}

impl ItemState {
    pub fn display_name(&self) -> &'static str {
        match self {
            ItemState::Current => "Current",
            ItemState::Loaned => "Loaned",
            ItemState::Missing => "Missing",
            ItemState::Disposed => "Disposed",
        }
    }

    pub fn css_class(&self) -> &'static str {
        match self {
            ItemState::Current => "state-current",
            ItemState::Loaned => "state-loaned",
            ItemState::Missing => "state-missing",
            ItemState::Disposed => "state-disposed",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_type: ItemType,
    pub state: ItemState,
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub location_id: Option<Uuid>,
    pub date_entered: chrono::DateTime<chrono::Utc>,
    pub date_acquired: Option<chrono::NaiveDate>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Location {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}

/// Helper function to extract auth token from cookies (server-side only)
#[cfg(feature = "ssr")]
async fn get_auth_token() -> Result<String, ServerFnError<NoCustomError>> {
    use axum::http::header::COOKIE;
    use leptos_axum::extract;

    let headers = extract::<axum::http::HeaderMap>().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to extract headers: {}", e))
    })?;

    headers
        .get(COOKIE)
        .and_then(|cookie_header| cookie_header.to_str().ok())
        .and_then(|cookies| {
            cookies
                .split(';')
                .map(|c| c.trim())
                .find(|c| c.starts_with("auth_token="))
                .map(|c| c.trim_start_matches("auth_token=").to_string())
        })
        .ok_or_else(|| ServerFnError::<NoCustomError>::ServerError("Not authenticated".to_string()))
}

/// Fetch paginated items for an organization
#[server(GetItems, "/api")]
pub async fn get_items(
    org_id: Uuid,
    page: i64,
    per_page: i64,
) -> Result<PaginatedResponse<Item>, ServerFnError<NoCustomError>> {
    let token = get_auth_token().await?;

    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "{}/api/organizations/{}/items?page={}&per_page={}",
            api_base_url, org_id, page, per_page
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!("API request failed: {}", e))
        })?;

    if response.status() == 401 {
        return Err(ServerFnError::<NoCustomError>::ServerError(
            "Not authenticated".to_string(),
        ));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(ServerFnError::<NoCustomError>::ServerError(format!(
            "Failed to fetch items: {} - {}",
            status, body
        )));
    }

    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}

/// Fetch all locations for an organization
#[server(GetLocations, "/api")]
pub async fn get_locations(org_id: Uuid) -> Result<Vec<Location>, ServerFnError<NoCustomError>> {
    let token = get_auth_token().await?;

    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let client = reqwest::Client::new();
    let response = client
        .get(format!(
            "{}/api/organizations/{}/locations",
            api_base_url, org_id
        ))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!("API request failed: {}", e))
        })?;

    if response.status() == 401 {
        return Err(ServerFnError::<NoCustomError>::ServerError(
            "Not authenticated".to_string(),
        ));
    }

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(ServerFnError::<NoCustomError>::ServerError(format!(
            "Failed to fetch locations: {} - {}",
            status, body
        )));
    }

    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}
