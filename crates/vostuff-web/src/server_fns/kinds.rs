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
