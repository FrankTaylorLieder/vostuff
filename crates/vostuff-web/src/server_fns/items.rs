use leptos::server_fn::error::NoCustomError;
use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// Types matching the API response

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Vinyl,
    Cd,
    Cassette,
    Book,
    Score,
    Electronics,
    Misc,
    Dvd,
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
            ItemType::Dvd => "DVD",
        }
    }

    pub fn api_value(&self) -> &'static str {
        match self {
            ItemType::Vinyl => "vinyl",
            ItemType::Cd => "cd",
            ItemType::Cassette => "cassette",
            ItemType::Book => "book",
            ItemType::Score => "score",
            ItemType::Electronics => "electronics",
            ItemType::Misc => "misc",
            ItemType::Dvd => "dvd",
        }
    }

    pub fn all() -> Vec<ItemType> {
        vec![
            ItemType::Vinyl,
            ItemType::Cd,
            ItemType::Cassette,
            ItemType::Book,
            ItemType::Score,
            ItemType::Electronics,
            ItemType::Misc,
            ItemType::Dvd,
        ]
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

    pub fn api_value(&self) -> &'static str {
        match self {
            ItemState::Current => "current",
            ItemState::Loaned => "loaned",
            ItemState::Missing => "missing",
            ItemState::Disposed => "disposed",
        }
    }

    pub fn all() -> Vec<ItemState> {
        vec![
            ItemState::Current,
            ItemState::Loaned,
            ItemState::Missing,
            ItemState::Disposed,
        ]
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

// Vinyl-specific enums
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VinylSize {
    #[serde(rename = "12_inch")]
    TwelveInch,
    #[serde(rename = "6_inch")]
    SixInch,
    Other,
}

impl VinylSize {
    pub fn display_name(&self) -> &'static str {
        match self {
            VinylSize::TwelveInch => "12\"",
            VinylSize::SixInch => "6\"",
            VinylSize::Other => "Other",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VinylSpeed {
    #[serde(rename = "33")]
    ThirtyThree,
    #[serde(rename = "45")]
    FortyFive,
    Other,
}

impl VinylSpeed {
    pub fn display_name(&self) -> &'static str {
        match self {
            VinylSpeed::ThirtyThree => "33 RPM",
            VinylSpeed::FortyFive => "45 RPM",
            VinylSpeed::Other => "Other",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VinylChannels {
    Mono,
    Stereo,
    Surround,
    Other,
}

impl VinylChannels {
    pub fn display_name(&self) -> &'static str {
        match self {
            VinylChannels::Mono => "Mono",
            VinylChannels::Stereo => "Stereo",
            VinylChannels::Surround => "Surround",
            VinylChannels::Other => "Other",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Grading {
    Mint,
    NearMint,
    Excellent,
    Good,
    Fair,
    Poor,
}

impl Grading {
    pub fn display_name(&self) -> &'static str {
        match self {
            Grading::Mint => "Mint",
            Grading::NearMint => "Near Mint",
            Grading::Excellent => "Excellent",
            Grading::Good => "Good",
            Grading::Fair => "Fair",
            Grading::Poor => "Poor",
        }
    }
}

// Detail structs
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VinylDetails {
    pub item_id: Uuid,
    pub size: Option<VinylSize>,
    pub speed: Option<VinylSpeed>,
    pub channels: Option<VinylChannels>,
    pub disks: Option<i32>,
    pub media_grading: Option<Grading>,
    pub sleeve_grading: Option<Grading>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CdDetails {
    pub item_id: Uuid,
    pub disks: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CassetteDetails {
    pub item_id: Uuid,
    pub cassettes: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DvdDetails {
    pub item_id: Uuid,
    pub disks: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoanDetails {
    pub item_id: Uuid,
    pub date_loaned: chrono::NaiveDate,
    pub date_due_back: Option<chrono::NaiveDate>,
    pub loaned_to: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct MissingDetails {
    pub item_id: Uuid,
    pub date_missing: chrono::NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DisposedDetails {
    pub item_id: Uuid,
    pub date_disposed: chrono::NaiveDate,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ItemFullDetails {
    pub item: Item,
    pub vinyl_details: Option<VinylDetails>,
    pub cd_details: Option<CdDetails>,
    pub cassette_details: Option<CassetteDetails>,
    pub dvd_details: Option<DvdDetails>,
    pub loan_details: Option<LoanDetails>,
    pub missing_details: Option<MissingDetails>,
    pub disposed_details: Option<DisposedDetails>,
}

/// Fetch full details for a single item
#[server(GetItemDetails, "/api")]
pub async fn get_item_details(
    org_id: Uuid,
    item_id: Uuid,
) -> Result<ItemFullDetails, ServerFnError<NoCustomError>> {
    let token = get_auth_token().await?;

    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let url = format!(
        "{}/api/organizations/{}/items/{}/details",
        api_base_url, org_id, item_id
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
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
            "Failed to fetch item details: {} - {}",
            status, body
        )));
    }

    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}

/// Filter parameters for items query
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ItemFilters {
    pub item_types: Vec<String>,
    pub states: Vec<String>,
    pub location_ids: Vec<Uuid>,
    pub search_query: Option<String>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

/// Fetch paginated items for an organization with optional filters
#[server(GetItems, "/api")]
pub async fn get_items(
    org_id: Uuid,
    page: i64,
    per_page: i64,
    filters: Option<ItemFilters>,
) -> Result<PaginatedResponse<Item>, ServerFnError<NoCustomError>> {
    let token = get_auth_token().await?;

    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    // Build query string with filters
    let mut url = format!(
        "{}/api/organizations/{}/items?page={}&per_page={}",
        api_base_url, org_id, page, per_page
    );

    if let Some(ref f) = filters {
        if !f.item_types.is_empty() {
            url.push_str(&format!("&item_type={}", f.item_types.join(",")));
        }
        if !f.states.is_empty() {
            url.push_str(&format!("&state={}", f.states.join(",")));
        }
        if !f.location_ids.is_empty() {
            let loc_str: Vec<String> = f.location_ids.iter().map(|id| id.to_string()).collect();
            url.push_str(&format!("&location_id={}", loc_str.join(",")));
        }
        if let Some(ref q) = f.search_query
            && !q.is_empty()
        {
            // Manual percent-encoding for the search query
            let encoded: String = q
                .chars()
                .map(|c| match c {
                    'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                    ' ' => "+".to_string(),
                    _ => format!("%{:02X}", c as u32),
                })
                .collect();
            url.push_str(&format!("&search={}", encoded));
        }
        if let Some(ref sb) = f.sort_by {
            url.push_str(&format!("&sort_by={}", sb));
        }
        if let Some(ref so) = f.sort_order {
            url.push_str(&format!("&sort_order={}", so));
        }
    }

    tracing::debug!(
        "get_items requesting URL: {} with filters: {:?}",
        url,
        filters
    );

    let client = reqwest::Client::new();
    let response = client
        .get(&url)
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
