use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::api::{
    models::{
        CreateItemRequest, DisposedDetails, ErrorResponse, Item, ItemFilterParams, ItemFullDetails,
        ItemState, LoanDetails, MissingDetails, PaginatedResponse, UpdateItemRequest,
    },
    state::AppState,
};

// Base SELECT shared by list, get, and details handlers
const ITEM_SELECT: &str = "
    SELECT i.id, i.organization_id, i.kind_id, k.name AS kind_name,
           i.state::text, i.name, i.description, i.notes,
           i.location_id, i.date_entered, i.date_acquired,
           i.created_at, i.updated_at, i.soft_fields
    FROM items i
    JOIN kinds k ON k.id = i.kind_id";

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
        "list_items called with filters: kind={:?}, state={:?}, location_id={:?}, search={:?}",
        filters.kind,
        filters.state,
        filters.location_id,
        filters.search
    );

    let offset = (filters.page - 1) * filters.per_page;

    // Parse filter values
    let kinds: Vec<String> = filters
        .kind
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

    // Build dynamic WHERE clause (table-prefixed for the JOIN)
    let mut where_clauses = vec!["i.organization_id = $1".to_string()];
    let mut param_idx = 2;

    if !kinds.is_empty() {
        let placeholders: Vec<String> = kinds
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", param_idx + i))
            .collect();
        where_clauses.push(format!("k.name IN ({})", placeholders.join(", ")));
        param_idx += kinds.len();
    }

    if !states.is_empty() {
        let placeholders: Vec<String> = states
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", param_idx + i))
            .collect();
        where_clauses.push(format!("i.state::text IN ({})", placeholders.join(", ")));
        param_idx += states.len();
    }

    if !location_ids.is_empty() {
        let placeholders: Vec<String> = location_ids
            .iter()
            .enumerate()
            .map(|(i, _)| format!("${}", param_idx + i))
            .collect();
        where_clauses.push(format!("i.location_id IN ({})", placeholders.join(", ")));
        param_idx += location_ids.len();
    }

    let search_pattern = filters.search.as_ref().map(|s| format!("%{}%", s));
    if search_pattern.is_some() {
        where_clauses.push(format!(
            "(i.name ILIKE ${p} OR i.description ILIKE ${p} OR i.notes ILIKE ${p})",
            p = param_idx
        ));
        param_idx += 1;
    }

    let where_clause = where_clauses.join(" AND ");

    // Count query
    let count_query = format!(
        "SELECT COUNT(*) as count FROM items i JOIN kinds k ON k.id = i.kind_id WHERE {}",
        where_clause
    );
    let mut count_builder = sqlx::query(&count_query).bind(org_id);
    for k in &kinds {
        count_builder = count_builder.bind(k);
    }
    for s in &states {
        count_builder = count_builder.bind(s);
    }
    for loc in &location_ids {
        count_builder = count_builder.bind(loc);
    }
    if let Some(ref pattern) = search_pattern {
        count_builder = count_builder.bind(pattern);
    }

    let total: i64 = count_builder
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?
        .get("count");

    // ORDER BY — whitelist to prevent injection
    let order_column = match filters.sort_by.as_deref() {
        Some("name") => "i.name",
        Some("kind") => "k.name",
        Some("state") => "i.state",
        Some("location_id") => "i.location_id",
        Some("created_at") => "i.created_at",
        _ => "i.name",
    };
    let order_direction = match filters.sort_order.as_deref() {
        Some("desc") => "DESC",
        _ => "ASC",
    };

    let items_query = format!(
        "{} WHERE {} ORDER BY {} {} LIMIT ${} OFFSET ${}",
        ITEM_SELECT, where_clause, order_column, order_direction, param_idx, param_idx + 1
    );

    let mut items_builder = sqlx::query_as::<_, ItemRow>(&items_query).bind(org_id);
    for k in &kinds {
        items_builder = items_builder.bind(k);
    }
    for s in &states {
        items_builder = items_builder.bind(s);
    }
    for loc in &location_ids {
        items_builder = items_builder.bind(loc);
    }
    if let Some(ref pattern) = search_pattern {
        items_builder = items_builder.bind(pattern);
    }
    items_builder = items_builder.bind(filters.per_page).bind(offset);

    let items: Vec<Item> = items_builder
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?
        .into_iter()
        .map(Into::into)
        .collect();

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
    let query = format!("{} WHERE i.id = $1 AND i.organization_id = $2", ITEM_SELECT);
    let item = sqlx::query_as::<_, ItemRow>(&query)
        .bind(item_id)
        .bind(org_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?;

    match item {
        Some(row) => Ok(Json(row.into())),
        None => Err(not_found()),
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
    // Validate kind exists (shared kinds have NULL org_id, org kinds must match)
    let kind_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM kinds WHERE id = $1 AND (org_id IS NULL OR org_id = $2))",
    )
    .bind(req.kind_id)
    .bind(org_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    if !kind_exists {
        return Err(bad_request("invalid_kind", "Kind not found"));
    }

    let soft_fields = req.soft_fields.unwrap_or(serde_json::json!({}));

    validate_soft_fields(&state.pool, req.kind_id, &soft_fields)
        .await
        .map_err(|e| bad_request("invalid_soft_fields", &e.to_string()))?;

    let query = format!(
        "INSERT INTO items
         (organization_id, kind_id, state, name, description, notes, location_id, date_acquired, soft_fields)
         VALUES ($1, $2, 'current'::item_state, $3, $4, $5, $6, $7, $8)
         RETURNING id, organization_id, kind_id,
           (SELECT name FROM kinds WHERE id = kind_id) AS kind_name,
           state::text, name, description, notes,
           location_id, date_entered, date_acquired, created_at, updated_at, soft_fields"
    );

    let row = sqlx::query_as::<_, ItemRow>(&query)
        .bind(org_id)
        .bind(req.kind_id)
        .bind(&req.name)
        .bind(&req.description)
        .bind(&req.notes)
        .bind(&req.location_id)
        .bind(&req.date_acquired)
        .bind(&soft_fields)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    Ok((StatusCode::CREATED, Json(row.into())))
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
    // Fetch current item to get kind_id and state for validation
    let current = sqlx::query("SELECT kind_id, state::text FROM items WHERE id = $1 AND organization_id = $2")
        .bind(item_id)
        .bind(org_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;

    let kind_id: Uuid = current.get("kind_id");
    let state_str: String = current.get("state");

    // Validate soft_fields if provided
    if let Some(ref sf) = req.soft_fields {
        validate_soft_fields(&state.pool, kind_id, sf)
            .await
            .map_err(|e| bad_request("invalid_soft_fields", &e.to_string()))?;
    }

    // Build dynamic UPDATE
    let mut query = String::from("UPDATE items SET updated_at = NOW()");
    let mut param_num = 3; // $1 = item_id, $2 = org_id

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
        param_num += 1;
    }
    if req.soft_fields.is_some() {
        // Merge: existing || new (new keys overwrite, absent keys preserved)
        query.push_str(&format!(", soft_fields = soft_fields || ${}", param_num));
        let _ = param_num; // last use of param_num
    }

    query.push_str(&format!(
        " WHERE id = $1 AND organization_id = $2
          RETURNING id, organization_id, kind_id,
            (SELECT name FROM kinds WHERE id = kind_id) AS kind_name,
            state::text, name, description, notes,
            location_id, date_entered, date_acquired, created_at, updated_at, soft_fields"
    ));

    let mut qb = sqlx::query_as::<_, ItemRow>(&query)
        .bind(item_id)
        .bind(org_id);

    if let Some(ref v) = req.name { qb = qb.bind(v); }
    if let Some(ref v) = req.description { qb = qb.bind(v); }
    if let Some(ref v) = req.notes { qb = qb.bind(v); }
    if let Some(ref v) = req.location_id { qb = qb.bind(v); }
    if let Some(ref v) = req.date_acquired { qb = qb.bind(v); }
    if let Some(ref v) = req.state { qb = qb.bind(item_state_to_db(v)); }
    if let Some(ref v) = req.soft_fields { qb = qb.bind(v); }

    let row = qb
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;

    let item: Item = row.into();

    // Upsert loan details
    let has_loan = req.loan_date_loaned.is_some()
        || req.loan_date_due_back.is_some()
        || req.loan_loaned_to.is_some();
    if has_loan && state_str == "loaned" {
        sqlx::query(
            "INSERT INTO item_loan_details (item_id, date_loaned, date_due_back, loaned_to)
             VALUES ($1, $2, $3, $4)
             ON CONFLICT (item_id) DO UPDATE SET
               date_loaned  = COALESCE($2, item_loan_details.date_loaned),
               date_due_back = COALESCE($3, item_loan_details.date_due_back),
               loaned_to    = COALESCE($4, item_loan_details.loaned_to)",
        )
        .bind(item_id)
        .bind(&req.loan_date_loaned)
        .bind(&req.loan_date_due_back)
        .bind(&req.loan_loaned_to)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
    }

    // Upsert missing details
    if req.missing_date_missing.is_some() && state_str == "missing" {
        sqlx::query(
            "INSERT INTO item_missing_details (item_id, date_missing) VALUES ($1, $2)
             ON CONFLICT (item_id) DO UPDATE SET
               date_missing = COALESCE($2, item_missing_details.date_missing)",
        )
        .bind(item_id)
        .bind(&req.missing_date_missing)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
    }

    // Upsert disposed details
    if req.disposed_date_disposed.is_some() && state_str == "disposed" {
        sqlx::query(
            "INSERT INTO item_disposed_details (item_id, date_disposed) VALUES ($1, $2)
             ON CONFLICT (item_id) DO UPDATE SET
               date_disposed = COALESCE($2, item_disposed_details.date_disposed)",
        )
        .bind(item_id)
        .bind(&req.disposed_date_disposed)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
    }

    Ok(Json(item))
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
        Err(not_found())
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}

/// Get full details for a single item (including state-specific details)
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/items/{item_id}/details",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("item_id" = Uuid, Path, description = "Item ID")
    ),
    responses(
        (status = 200, description = "Item full details", body = ItemFullDetails),
        (status = 404, description = "Item not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    tag = "items"
)]
pub async fn get_item_details(
    State(state): State<AppState>,
    Path((org_id, item_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ItemFullDetails>, (StatusCode, Json<ErrorResponse>)> {
    let query = format!(
        "{} WHERE i.id = $1 AND i.organization_id = $2",
        ITEM_SELECT
    );
    let item_row = sqlx::query_as::<_, ItemRow>(&query)
        .bind(item_id)
        .bind(org_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;

    let state_str = item_row.state.clone();
    let item: Item = item_row.into();

    let loan_details = if state_str == "loaned" {
        sqlx::query_as::<_, LoanDetailsRow>(
            "SELECT item_id, date_loaned, date_due_back, loaned_to
             FROM item_loan_details WHERE item_id = $1",
        )
        .bind(item_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .map(|r| LoanDetails {
            item_id: r.item_id,
            date_loaned: r.date_loaned,
            date_due_back: r.date_due_back,
            loaned_to: r.loaned_to,
        })
    } else {
        None
    };

    let missing_details = if state_str == "missing" {
        sqlx::query_as::<_, MissingDetailsRow>(
            "SELECT item_id, date_missing FROM item_missing_details WHERE item_id = $1",
        )
        .bind(item_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .map(|r| MissingDetails {
            item_id: r.item_id,
            date_missing: r.date_missing,
        })
    } else {
        None
    };

    let disposed_details = if state_str == "disposed" {
        sqlx::query_as::<_, DisposedDetailsRow>(
            "SELECT item_id, date_disposed FROM item_disposed_details WHERE item_id = $1",
        )
        .bind(item_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .map(|r| DisposedDetails {
            item_id: r.item_id,
            date_disposed: r.date_disposed,
        })
    } else {
        None
    };

    Ok(Json(ItemFullDetails {
        item,
        loan_details,
        missing_details,
        disposed_details,
    }))
}

// ── Soft field validation ──────────────────────────────────────────────────

async fn validate_soft_fields(
    pool: &PgPool,
    kind_id: Uuid,
    soft_fields: &serde_json::Value,
) -> anyhow::Result<()> {
    let obj = match soft_fields.as_object() {
        Some(o) if !o.is_empty() => o,
        _ => return Ok(()),
    };

    // Fetch all field names and types for this kind
    let rows = sqlx::query(
        "SELECT f.name, f.field_type::text AS field_type
         FROM kind_fields kf
         JOIN fields f ON f.id = kf.field_id
         WHERE kf.kind_id = $1",
    )
    .bind(kind_id)
    .fetch_all(pool)
    .await?;

    let field_types: HashMap<String, String> = rows
        .iter()
        .map(|r| (r.get::<String, _>("name"), r.get::<String, _>("field_type")))
        .collect();

    for (key, value) in obj {
        let field_type = match field_types.get(key) {
            Some(t) => t.as_str(),
            None => continue, // unknown fields are passed through without validation
        };

        match field_type {
            "number" => {
                if !value.is_number() {
                    anyhow::bail!("Field '{}' must be a number", key);
                }
            }
            "boolean" => {
                if !value.is_boolean() {
                    anyhow::bail!("Field '{}' must be a boolean", key);
                }
            }
            "string" | "text" | "date" | "datetime" => {
                if !value.is_string() {
                    anyhow::bail!("Field '{}' must be a string", key);
                }
            }
            "enum" => {
                let v = value
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Field '{}' must be a string", key))?;

                let valid: bool = sqlx::query_scalar(
                    "SELECT EXISTS(
                         SELECT 1 FROM enum_values ev
                         JOIN fields f ON f.id = ev.field_id
                         JOIN kind_fields kf ON kf.field_id = f.id
                         WHERE kf.kind_id = $1 AND f.name = $2 AND ev.value = $3
                     )",
                )
                .bind(kind_id)
                .bind(key)
                .bind(v)
                .fetch_one(pool)
                .await?;

                if !valid {
                    let allowed: Vec<String> = sqlx::query_scalar(
                        "SELECT ev.value FROM enum_values ev
                         JOIN fields f ON f.id = ev.field_id
                         JOIN kind_fields kf ON kf.field_id = f.id
                         WHERE kf.kind_id = $1 AND f.name = $2
                         ORDER BY ev.sort_order",
                    )
                    .bind(kind_id)
                    .bind(key)
                    .fetch_all(pool)
                    .await?;

                    anyhow::bail!(
                        "Field '{}' value '{}' is not valid. Allowed values: {}",
                        key,
                        v,
                        allowed.join(", ")
                    );
                }
            }
            _ => {}
        }
    }

    Ok(())
}

// ── Row types ──────────────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct ItemRow {
    id: Uuid,
    organization_id: Uuid,
    kind_id: Uuid,
    kind_name: String,
    state: String,
    name: String,
    description: Option<String>,
    notes: Option<String>,
    location_id: Option<Uuid>,
    date_entered: chrono::DateTime<chrono::Utc>,
    date_acquired: Option<chrono::NaiveDate>,
    created_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    soft_fields: serde_json::Value,
}

impl From<ItemRow> for Item {
    fn from(row: ItemRow) -> Self {
        Item {
            id: row.id,
            organization_id: row.organization_id,
            kind_id: row.kind_id,
            kind_name: row.kind_name,
            state: db_to_item_state(&row.state),
            name: row.name,
            description: row.description,
            notes: row.notes,
            location_id: row.location_id,
            date_entered: row.date_entered,
            date_acquired: row.date_acquired,
            soft_fields: row.soft_fields,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}

#[derive(sqlx::FromRow)]
struct LoanDetailsRow {
    item_id: Uuid,
    date_loaned: chrono::NaiveDate,
    date_due_back: Option<chrono::NaiveDate>,
    loaned_to: String,
}

#[derive(sqlx::FromRow)]
struct MissingDetailsRow {
    item_id: Uuid,
    date_missing: chrono::NaiveDate,
}

#[derive(sqlx::FromRow)]
struct DisposedDetailsRow {
    item_id: Uuid,
    date_disposed: chrono::NaiveDate,
}

// ── Helpers ────────────────────────────────────────────────────────────────

fn db_to_item_state(s: &str) -> ItemState {
    match s {
        "current" => ItemState::Current,
        "loaned" => ItemState::Loaned,
        "missing" => ItemState::Missing,
        "disposed" => ItemState::Disposed,
        _ => ItemState::Current,
    }
}

fn item_state_to_db(s: &ItemState) -> &'static str {
    match s {
        ItemState::Current => "current",
        ItemState::Loaned => "loaned",
        ItemState::Missing => "missing",
        ItemState::Disposed => "disposed",
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

fn not_found() -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse {
            error: "not_found".to_string(),
            message: "Item not found".to_string(),
        }),
    )
}

fn bad_request(error: &str, message: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: error.to_string(),
            message: message.to_string(),
        }),
    )
}
