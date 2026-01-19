use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use sqlx::{FromRow, Row};
use uuid::Uuid;

use crate::api::{
    models::{
        CreateItemRequest, ErrorResponse, Item, ItemFilterParams, ItemState, ItemType,
        PaginatedResponse, UpdateItemRequest,
    },
    state::AppState,
};

/// List all items for an organization with optional filters
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/items",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ItemFilterParams
    ),
    responses(
        (status = 200, description = "List of items", body = PaginatedResponse<Item>),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "items"
)]
pub async fn list_items(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Query(filters): Query<ItemFilterParams>,
) -> Result<Json<PaginatedResponse<Item>>, (StatusCode, Json<ErrorResponse>)> {
    tracing::debug!(
        "list_items called with filters: item_type={:?}, state={:?}, location_id={:?}",
        filters.item_type,
        filters.state,
        filters.location_id
    );

    let offset = (filters.page - 1) * filters.per_page;

    // Parse filter values
    let item_types: Vec<String> = filters
        .item_type
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
        .unwrap_or_default();

    let states: Vec<String> = filters
        .state
        .as_ref()
        .map(|s| s.split(',').map(|t| t.trim().to_string()).collect())
        .unwrap_or_default();

    let location_ids: Vec<Uuid> = filters
        .location_id
        .as_ref()
        .map(|s| {
            s.split(',')
                .filter_map(|t| Uuid::parse_str(t.trim()).ok())
                .collect()
        })
        .unwrap_or_default();

    // Build dynamic WHERE clause
    let mut where_clauses = vec!["organization_id = $1".to_string()];
    let mut param_idx = 2;

    if !item_types.is_empty() {
        let placeholders: Vec<String> = item_types
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", param_idx + i))
            .collect();
        where_clauses.push(format!("item_type::text IN ({})", placeholders.join(", ")));
        param_idx += item_types.len();
    }

    if !states.is_empty() {
        let placeholders: Vec<String> = states
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", param_idx + i))
            .collect();
        where_clauses.push(format!("state::text IN ({})", placeholders.join(", ")));
        param_idx += states.len();
    }

    if !location_ids.is_empty() {
        let placeholders: Vec<String> = location_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", param_idx + i))
            .collect();
        where_clauses.push(format!("location_id IN ({})", placeholders.join(", ")));
        param_idx += location_ids.len();
    }

    let where_clause = where_clauses.join(" AND ");

    // Build count query
    let count_query = format!("SELECT COUNT(*) as count FROM items WHERE {}", where_clause);
    let mut count_builder = sqlx::query(&count_query).bind(org_id);
    for t in &item_types {
        count_builder = count_builder.bind(t);
    }
    for s in &states {
        count_builder = count_builder.bind(s);
    }
    for loc in &location_ids {
        count_builder = count_builder.bind(loc);
    }

    let total_result = count_builder
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;
    let total: i64 = total_result.get("count");

    // Build items query
    let items_query = format!(
        "SELECT id, organization_id, item_type::text, state::text, name, description, notes,
         location_id, date_entered, date_acquired, created_at, updated_at
         FROM items WHERE {}
         ORDER BY created_at DESC LIMIT ${} OFFSET ${}",
        where_clause,
        param_idx,
        param_idx + 1
    );

    let mut items_builder = sqlx::query_as::<_, ItemRow>(&items_query).bind(org_id);
    for t in &item_types {
        items_builder = items_builder.bind(t);
    }
    for s in &states {
        items_builder = items_builder.bind(s);
    }
    for loc in &location_ids {
        items_builder = items_builder.bind(loc);
    }
    items_builder = items_builder.bind(filters.per_page).bind(offset);

    let items = items_builder
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;

    let items: Vec<Item> = items.into_iter().map(|row| row.into()).collect();
    let total_pages = if total == 0 {
        1
    } else {
        (total + filters.per_page - 1) / filters.per_page
    };

    Ok(Json(PaginatedResponse {
        items,
        total,
        page: filters.page,
        per_page: filters.per_page,
        total_pages,
    }))
}

/// Get a single item by ID
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/items/{item_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("item_id" = Uuid, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Item details", body = Item),
        (status = 404, description = "Item not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "items"
)]
pub async fn get_item(
    State(state): State<AppState>,
    Path((org_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Item>, (StatusCode, Json<ErrorResponse>)> {
    let item = sqlx::query_as::<_, ItemRow>(
        "SELECT id, organization_id, item_type::text, state::text, name, description, notes,
         location_id, date_entered, date_acquired, created_at, updated_at
         FROM items WHERE id = $1 AND organization_id = $2",
    )
    .bind(item_id)
    .bind(org_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    match item {
        Some(item) => Ok(Json(item.into())),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "Item not found".to_string(),
            }),
        )),
    }
}

/// Create a new item
#[utoipa::path(
    post,
    path = "/api/organizations/{org_id}/items",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID")
    ),
    request_body = CreateItemRequest,
    responses(
        (status = 201, description = "Item created successfully", body = Item),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "items"
)]
pub async fn create_item(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateItemRequest>,
) -> Result<(StatusCode, Json<Item>), (StatusCode, Json<ErrorResponse>)> {
    let item_type_str = item_type_to_db(&req.item_type);

    let row = sqlx::query(
        "INSERT INTO items (organization_id, item_type, state, name, description, notes, location_id, date_acquired)
         VALUES ($1, $2::item_type, 'current'::item_state, $3, $4, $5, $6, $7)
         RETURNING id, organization_id, item_type::text, state::text, name, description, notes,
         location_id, date_entered, date_acquired, created_at, updated_at"
    )
    .bind(org_id)
    .bind(&item_type_str)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.notes)
    .bind(&req.location_id)
    .bind(&req.date_acquired)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    let item = ItemRow::from_row(&row).map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(item.into())))
}

/// Update an existing item
#[utoipa::path(
    patch,
    path = "/api/organizations/{org_id}/items/{item_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("item_id" = Uuid, Path, description = "Item ID")
    ),
    request_body = UpdateItemRequest,
    responses(
        (status = 200, description = "Item updated successfully", body = Item),
        (status = 404, description = "Item not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "items"
)]
pub async fn update_item(
    State(state): State<AppState>,
    Path((org_id, item_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateItemRequest>,
) -> Result<Json<Item>, (StatusCode, Json<ErrorResponse>)> {
    // Build dynamic update query
    let mut query = String::from("UPDATE items SET updated_at = NOW()");
    let mut param_num = 3;

    if req.name.is_some() {
        query.push_str(&format!(", name = ${}", param_num));
        param_num += 1;
    }
    if req.description.is_some() {
        query.push_str(&format!(", description = ${}", param_num));
        param_num += 1;
    }
    if req.notes.is_some() {
        query.push_str(&format!(", notes = ${}", param_num));
        param_num += 1;
    }
    if req.location_id.is_some() {
        query.push_str(&format!(", location_id = ${}", param_num));
        param_num += 1;
    }
    if req.date_acquired.is_some() {
        query.push_str(&format!(", date_acquired = ${}", param_num));
        param_num += 1;
    }
    if req.state.is_some() {
        query.push_str(&format!(", state = ${}::item_state", param_num));
    }

    query.push_str(" WHERE id = $1 AND organization_id = $2 RETURNING id, organization_id, item_type::text, state::text, name, description, notes, location_id, date_entered, date_acquired, created_at, updated_at");

    let mut query_builder = sqlx::query(&query).bind(item_id).bind(org_id);

    if let Some(name) = &req.name {
        query_builder = query_builder.bind(name);
    }
    if let Some(description) = &req.description {
        query_builder = query_builder.bind(description);
    }
    if let Some(notes) = &req.notes {
        query_builder = query_builder.bind(notes);
    }
    if let Some(location_id) = &req.location_id {
        query_builder = query_builder.bind(location_id);
    }
    if let Some(date_acquired) = &req.date_acquired {
        query_builder = query_builder.bind(date_acquired);
    }
    if let Some(state) = &req.state {
        query_builder = query_builder.bind(item_state_to_db(state));
    }

    let row = query_builder
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    match row {
        Some(row) => {
            let item = ItemRow::from_row(&row).map_err(internal_error)?;
            Ok(Json(item.into()))
        }
        None => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "Item not found".to_string(),
            }),
        )),
    }
}

/// Delete an item
#[utoipa::path(
    delete,
    path = "/api/organizations/{org_id}/items/{item_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("item_id" = Uuid, Path, description = "Item ID")
    ),
    responses(
        (status = 204, description = "Item deleted successfully"),
        (status = 404, description = "Item not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "items"
)]
pub async fn delete_item(
    State(state): State<AppState>,
    Path((org_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let result = sqlx::query("DELETE FROM items WHERE id = $1 AND organization_id = $2")
        .bind(item_id)
        .bind(org_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "not_found".to_string(),
                message: "Item not found".to_string(),
            }),
        ))
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

// Helper structs and functions

#[derive(sqlx::FromRow)]
struct ItemRow {
    id: Uuid,
    organization_id: Uuid,
    item_type: String,
    state: String,
    name: String,
    description: Option<String>,
    notes: Option<String>,
    location_id: Option<Uuid>,
    date_entered: chrono::DateTime<chrono::Utc>,
    date_acquired: Option<chrono::NaiveDate>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
}

impl From<ItemRow> for Item {
    fn from(row: ItemRow) -> Self {
        Item {
            id: row.id,
            organization_id: row.organization_id,
            item_type: db_to_item_type(&row.item_type),
            state: db_to_item_state(&row.state),
            name: row.name,
            description: row.description,
            notes: row.notes,
            location_id: row.location_id,
            date_entered: row.date_entered,
            date_acquired: row.date_acquired,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

fn db_to_item_type(s: &str) -> ItemType {
    match s {
        "vinyl" => ItemType::Vinyl,
        "cd" => ItemType::Cd,
        "cassette" => ItemType::Cassette,
        "book" => ItemType::Book,
        "score" => ItemType::Score,
        "electronics" => ItemType::Electronics,
        "misc" => ItemType::Misc,
        _ => ItemType::Misc,
    }
}

fn item_type_to_db(t: &ItemType) -> String {
    match t {
        ItemType::Vinyl => "vinyl".to_string(),
        ItemType::Cd => "cd".to_string(),
        ItemType::Cassette => "cassette".to_string(),
        ItemType::Book => "book".to_string(),
        ItemType::Score => "score".to_string(),
        ItemType::Electronics => "electronics".to_string(),
        ItemType::Misc => "misc".to_string(),
    }
}

fn db_to_item_state(s: &str) -> ItemState {
    match s {
        "current" => ItemState::Current,
        "loaned" => ItemState::Loaned,
        "missing" => ItemState::Missing,
        "disposed" => ItemState::Disposed,
        _ => ItemState::Current,
    }
}

fn item_state_to_db(s: &ItemState) -> String {
    match s {
        ItemState::Current => "current".to_string(),
        ItemState::Loaned => "loaned".to_string(),
        ItemState::Missing => "missing".to_string(),
        ItemState::Disposed => "disposed".to_string(),
    }
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
