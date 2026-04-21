use leptos::server_fn::error::NoCustomError;
use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FieldEnumValue {
    pub id: Uuid,
    pub value: String,
    pub display_value: Option<String>,
    pub sort_order: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Field {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub name: String,
    pub display_name: Option<String>,
    pub field_type: String,
    pub is_shared: bool,
    pub enum_values: Vec<FieldEnumValue>,
}

/// Fetch all fields visible to the org (shared + org-owned)
#[server(GetFields, "/api")]
pub async fn get_fields(org_id: Uuid) -> Result<Vec<Field>, ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let url = format!("{}/api/organizations/{}/fields", api_base_url, org_id);
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
            "Failed to fetch fields: {} - {}",
            status, body
        )));
    }
    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}

/// Create a new org-owned field; enum_values is JSON-encoded or None
#[server(CreateField, "/api")]
pub async fn create_field(
    org_id: Uuid,
    name: String,
    display_name: Option<String>,
    field_type: String,
    enum_values: Option<String>,
) -> Result<Field, ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let url = format!("{}/api/organizations/{}/fields", api_base_url, org_id);
    let ev_val: serde_json::Value = match enum_values {
        Some(ref s) => serde_json::from_str(s).map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!(
                "Invalid enum_values JSON: {}",
                e
            ))
        })?,
        None => serde_json::json!([]),
    };
    let body = serde_json::json!({
        "name": name,
        "display_name": display_name,
        "field_type": field_type,
        "enum_values": ev_val,
    });
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
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
            "Failed to create field: {} - {}",
            status, body
        )));
    }
    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}

/// Update an org-owned field; enum_values is JSON-encoded Vec or None (no change)
#[server(UpdateField, "/api")]
pub async fn update_field(
    org_id: Uuid,
    field_id: Uuid,
    display_name: Option<String>,
    enum_values: Option<String>,
) -> Result<Field, ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let url = format!(
        "{}/api/organizations/{}/fields/{}",
        api_base_url, org_id, field_id
    );
    let ev_val: Option<serde_json::Value> = match enum_values {
        Some(ref s) => Some(serde_json::from_str(s).map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!(
                "Invalid enum_values JSON: {}",
                e
            ))
        })?),
        None => None,
    };
    let mut body = serde_json::json!({
        "display_name": display_name,
    });
    if let Some(ev) = ev_val {
        body["enum_values"] = ev;
    }
    let client = reqwest::Client::new();
    let response = client
        .patch(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&body)
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
            "Failed to update field: {} - {}",
            status, body
        )));
    }
    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}

/// Delete an org-owned field
#[server(DeleteField, "/api")]
pub async fn delete_field(
    org_id: Uuid,
    field_id: Uuid,
) -> Result<(), ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let url = format!(
        "{}/api/organizations/{}/fields/{}",
        api_base_url, org_id, field_id
    );
    let client = reqwest::Client::new();
    let response = client
        .delete(&url)
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
            "Failed to delete field: {} - {}",
            status, body
        )));
    }
    Ok(())
}
