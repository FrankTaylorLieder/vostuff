use leptos::server_fn::error::NoCustomError;
use leptos::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KindSummary {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
}

/// Full Kind object including fields and org ownership info (used in settings)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Kind {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub name: String,
    pub display_name: Option<String>,
    pub is_shared: bool,
    pub fields: Vec<KindFieldDef>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RevertResponse {
    pub items_reassigned: i64,
    pub orphaned_field_names: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KindEnumValue {
    pub value: String,
    pub display_value: Option<String>,
    pub sort_order: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct KindFieldDef {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub field_type: String,
    pub display_order: i32,
    pub enum_values: Vec<KindEnumValue>,
}

#[derive(Deserialize)]
struct KindWithFields {
    fields: Vec<KindFieldDef>,
}

#[server(GetKindFields, "/api")]
pub async fn get_kind_fields(
    org_id: Uuid,
    kind_id: Uuid,
) -> Result<Vec<KindFieldDef>, ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let url = format!(
        "{}/api/organizations/{}/kinds/{}",
        api_base_url, org_id, kind_id
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
            "Failed to fetch kind: {} - {}",
            status, body
        )));
    }
    let kind: KindWithFields = response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })?;
    Ok(kind.fields)
}

#[server(GetKinds, "/api")]
pub async fn get_kinds(org_id: Uuid) -> Result<Vec<KindSummary>, ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());

    let url = format!("{}/api/organizations/{}/kinds", api_base_url, org_id);

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
            "Failed to fetch kinds: {} - {}",
            status, body
        )));
    }

    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}

/// Fetch full Kind objects (with fields and ownership info) — used by the settings page
#[server(GetKindsFull, "/api")]
pub async fn get_kinds_full(org_id: Uuid) -> Result<Vec<Kind>, ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let url = format!("{}/api/organizations/{}/kinds", api_base_url, org_id);
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
            "Failed to fetch kinds: {} - {}",
            status, body
        )));
    }
    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}

/// Create a new org-owned kind; field_ids is JSON-encoded Vec<Uuid>
#[server(CreateKind, "/api")]
pub async fn create_kind(
    org_id: Uuid,
    name: String,
    display_name: Option<String>,
    field_ids: String,
) -> Result<Kind, ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let url = format!("{}/api/organizations/{}/kinds", api_base_url, org_id);
    let ids: Vec<Uuid> = serde_json::from_str(&field_ids).map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Invalid field_ids JSON: {}", e))
    })?;
    let body = serde_json::json!({
        "name": name,
        "display_name": display_name,
        "field_ids": ids,
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
            "Failed to create kind: {} - {}",
            status, body
        )));
    }
    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}

/// Update an org-owned kind; field_ids is JSON-encoded Vec<Uuid> or None; force strips data
#[server(UpdateKind, "/api")]
pub async fn update_kind(
    org_id: Uuid,
    kind_id: Uuid,
    display_name: Option<String>,
    field_ids: Option<String>,
    force: bool,
) -> Result<Kind, ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let mut url = format!(
        "{}/api/organizations/{}/kinds/{}",
        api_base_url, org_id, kind_id
    );
    if force {
        url.push_str("?force=true");
    }
    let ids: Option<Vec<Uuid>> = match field_ids {
        Some(ref s) => Some(serde_json::from_str(s).map_err(|e| {
            ServerFnError::<NoCustomError>::ServerError(format!("Invalid field_ids JSON: {}", e))
        })?),
        None => None,
    };
    let body = serde_json::json!({
        "display_name": display_name,
        "field_ids": ids,
    });
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
            "Failed to update kind: {} - {}",
            status, body
        )));
    }
    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}

/// Delete an org-owned kind
#[server(DeleteKind, "/api")]
pub async fn delete_kind(org_id: Uuid, kind_id: Uuid) -> Result<(), ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let url = format!(
        "{}/api/organizations/{}/kinds/{}",
        api_base_url, org_id, kind_id
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
            "Failed to delete kind: {} - {}",
            status, body
        )));
    }
    Ok(())
}

/// Create an org-specific override of a shared kind
#[server(OverrideKind, "/api")]
pub async fn override_kind(
    org_id: Uuid,
    kind_id: Uuid,
) -> Result<Kind, ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let url = format!(
        "{}/api/organizations/{}/kinds/{}/override",
        api_base_url, org_id, kind_id
    );
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({}))
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
            "Failed to override kind: {} - {}",
            status, body
        )));
    }
    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}

/// Revert an org-owned kind back to the shared kind
#[server(RevertKind, "/api")]
pub async fn revert_kind(
    org_id: Uuid,
    kind_id: Uuid,
) -> Result<RevertResponse, ServerFnError<NoCustomError>> {
    let token = super::items::get_auth_token().await?;
    let api_base_url =
        std::env::var("API_BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
    let url = format!(
        "{}/api/organizations/{}/kinds/{}/revert",
        api_base_url, org_id, kind_id
    );
    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({}))
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
            "Failed to revert kind: {} - {}",
            status, body
        )));
    }
    response.json().await.map_err(|e| {
        ServerFnError::<NoCustomError>::ServerError(format!("Failed to parse response: {}", e))
    })
}
