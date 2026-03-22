use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::{
    models::ErrorResponse,
    state::AppState,
};

pub use super::fields::{EnumValue, FieldType};

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct KindField {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
    pub field_type: FieldType,
    pub display_order: i32,
    pub enum_values: Vec<EnumValue>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Kind {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub name: String,
    pub display_name: Option<String>,
    pub is_shared: bool,
    pub fields: Vec<KindField>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct KindSummary {
    pub id: Uuid,
    pub name: String,
    pub display_name: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateKindRequest {
    pub name: String,
    pub display_name: Option<String>,
    /// Ordered field IDs; display_order follows index position (0-based)
    pub field_ids: Vec<Uuid>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateKindRequest {
    pub display_name: Option<String>,
    /// If Some, replaces the entire field set in the given order
    pub field_ids: Option<Vec<Uuid>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateKindQuery {
    #[serde(default)]
    pub force: bool,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct RevertResponse {
    pub items_reassigned: i64,
    /// Field names that were in the org kind but not the shared kind;
    /// their values remain in items' soft_fields as orphan keys
    pub orphaned_field_names: Vec<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct DataLossError {
    pub error: String,
    pub message: String,
    pub fields_with_data: Vec<String>,
}

// ── Internal row types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct EnumValueJson {
    id: Uuid,
    value: String,
    display_value: Option<String>,
    sort_order: i32,
}

#[derive(Debug, Deserialize)]
struct KindFieldJson {
    id: Uuid,
    name: String,
    display_name: Option<String>,
    field_type: String,
    display_order: i32,
    enum_values: Vec<EnumValueJson>,
}

#[derive(Debug, sqlx::FromRow)]
struct KindRow {
    id: Uuid,
    org_id: Option<Uuid>,
    name: String,
    display_name: Option<String>,
    is_shared: bool,
    fields: serde_json::Value,
}

impl TryFrom<KindRow> for Kind {
    type Error = serde_json::Error;

    fn try_from(row: KindRow) -> Result<Self, Self::Error> {
        let field_jsons: Vec<KindFieldJson> = serde_json::from_value(row.fields)?;
        let fields = field_jsons
            .into_iter()
            .map(|f| KindField {
                id: f.id,
                name: f.name,
                display_name: f.display_name,
                field_type: FieldType::from_str(&f.field_type),
                display_order: f.display_order,
                enum_values: f
                    .enum_values
                    .into_iter()
                    .map(|ev| EnumValue {
                        id: ev.id,
                        value: ev.value,
                        display_value: ev.display_value,
                        sort_order: ev.sort_order,
                    })
                    .collect(),
            })
            .collect();
        Ok(Kind {
            id: row.id,
            org_id: row.org_id,
            name: row.name,
            display_name: row.display_name,
            is_shared: row.is_shared,
            fields,
        })
    }
}

// ── Core query ──────────────────────────────────────────────────────────────

const KIND_SELECT: &str = "
    SELECT
        k.id, k.org_id, k.name, k.display_name,
        (k.org_id IS NULL) AS is_shared,
        COALESCE(
            json_agg(
                json_build_object(
                    'id',            f.id,
                    'name',          f.name,
                    'display_name',  f.display_name,
                    'field_type',    f.field_type::text,
                    'display_order', kf.display_order,
                    'enum_values',   COALESCE(
                        (SELECT json_agg(
                             json_build_object(
                                 'id',            ev.id,
                                 'value',         ev.value,
                                 'display_value', ev.display_value,
                                 'sort_order',    ev.sort_order
                             ) ORDER BY ev.sort_order
                         ) FROM enum_values ev WHERE ev.field_id = f.id),
                        '[]'::json
                    )
                ) ORDER BY kf.display_order
            ) FILTER (WHERE f.id IS NOT NULL),
            '[]'::json
        ) AS fields
    FROM kinds k
    LEFT JOIN kind_fields kf ON kf.kind_id = k.id
    LEFT JOIN fields f       ON f.id = kf.field_id
    WHERE (k.org_id IS NULL OR k.org_id = $1)";

// ── Handlers ────────────────────────────────────────────────────────────────

/// List all kinds visible to an org (shared + org-owned), with full field details
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/kinds",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "List of kinds", body = Vec<Kind>),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "kinds"
)]
pub async fn list_kinds(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<Kind>>, (StatusCode, Json<ErrorResponse>)> {
    let query = format!(
        "{} GROUP BY k.id ORDER BY k.display_name NULLS LAST, k.name",
        KIND_SELECT
    );
    let rows = sqlx::query_as::<_, KindRow>(&query)
        .bind(org_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;

    let kinds: Vec<Kind> = rows
        .into_iter()
        .map(Kind::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(internal_error)?;

    Ok(Json(kinds))
}

/// Get a single kind by ID
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/kinds/{kind_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("kind_id" = Uuid, Path, description = "Kind ID"),
    ),
    responses(
        (status = 200, description = "Kind details", body = Kind),
        (status = 404, description = "Kind not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "kinds"
)]
pub async fn get_kind(
    State(state): State<AppState>,
    Path((org_id, kind_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Kind>, (StatusCode, Json<ErrorResponse>)> {
    let query = format!("{} AND k.id = $2 GROUP BY k.id", KIND_SELECT);
    let row = sqlx::query_as::<_, KindRow>(&query)
        .bind(org_id)
        .bind(kind_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;

    let kind = Kind::try_from(row).map_err(internal_error)?;
    Ok(Json(kind))
}

/// Create a new org-owned kind
#[utoipa::path(
    post,
    path = "/api/organizations/{org_id}/kinds",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    request_body = CreateKindRequest,
    responses(
        (status = 201, description = "Kind created", body = Kind),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 409, description = "Name already in use", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "kinds"
)]
pub async fn create_kind(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateKindRequest>,
) -> Result<(StatusCode, Json<Kind>), (StatusCode, Json<ErrorResponse>)> {
    // Check name is not taken by a shared kind
    let shared_conflict: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM kinds WHERE name = $1 AND org_id IS NULL)",
    )
    .bind(&req.name)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    if shared_conflict {
        return Err(conflict(
            "name_conflict",
            "A shared kind with this name already exists",
        ));
    }

    // Check name is not taken by an org kind
    let org_conflict: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM kinds WHERE name = $1 AND org_id = $2)",
    )
    .bind(&req.name)
    .bind(org_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    if org_conflict {
        return Err(conflict(
            "name_conflict",
            "A kind with this name already exists in this organization",
        ));
    }

    // Validate that all field_ids exist (shared or org-owned)
    if !req.field_ids.is_empty() {
        let valid_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM fields WHERE id = ANY($1) AND (org_id IS NULL OR org_id = $2)",
        )
        .bind(&req.field_ids)
        .bind(org_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

        if valid_count != req.field_ids.len() as i64 {
            return Err(bad_request(
                "invalid_fields",
                "One or more field IDs are invalid or not accessible",
            ));
        }
    }

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    let new_id: Uuid = sqlx::query_scalar(
        "INSERT INTO kinds (org_id, name, display_name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.display_name)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_error)?;

    for (idx, field_id) in req.field_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO kind_fields (kind_id, field_id, display_order) VALUES ($1, $2, $3)",
        )
        .bind(new_id)
        .bind(field_id)
        .bind(idx as i32)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;

    let query = format!("{} AND k.id = $2 GROUP BY k.id", KIND_SELECT);
    let row = sqlx::query_as::<_, KindRow>(&query)
        .bind(org_id)
        .bind(new_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    let kind = Kind::try_from(row).map_err(internal_error)?;
    Ok((StatusCode::CREATED, Json(kind)))
}

/// Update a kind's display name or field list
#[utoipa::path(
    patch,
    path = "/api/organizations/{org_id}/kinds/{kind_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("kind_id" = Uuid, Path, description = "Kind ID"),
        ("force" = bool, Query, description = "Force removal of fields that have item data"),
    ),
    request_body = UpdateKindRequest,
    responses(
        (status = 200, description = "Kind updated", body = Kind),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 403, description = "Cannot modify a shared kind", body = ErrorResponse),
        (status = 404, description = "Kind not found", body = ErrorResponse),
        (status = 409, description = "Data loss required; pass force=true to confirm", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "kinds"
)]
pub async fn update_kind(
    State(state): State<AppState>,
    Path((org_id, kind_id)): Path<(Uuid, Uuid)>,
    Query(q): Query<UpdateKindQuery>,
    Json(req): Json<UpdateKindRequest>,
) -> Result<Json<Kind>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch the kind and verify it belongs to this org
    let row = sqlx::query("SELECT id, org_id FROM kinds WHERE id = $1")
        .bind(kind_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;

    let kind_org_id: Option<Uuid> = row.get("org_id");
    if kind_org_id.is_none() {
        return Err(forbidden("Cannot modify a shared kind"));
    }
    if kind_org_id != Some(org_id) {
        return Err(forbidden("Kind does not belong to this organization"));
    }

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    if let Some(ref display_name) = req.display_name {
        sqlx::query(
            "UPDATE kinds SET display_name = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(display_name)
        .bind(kind_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    if let Some(ref new_field_ids) = req.field_ids {
        // Get current field IDs and names
        let current_fields: Vec<(Uuid, String)> = sqlx::query(
            "SELECT f.id, f.name FROM kind_fields kf JOIN fields f ON f.id = kf.field_id WHERE kf.kind_id = $1",
        )
        .bind(kind_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(internal_error)?
        .into_iter()
        .map(|r| (r.get::<Uuid, _>("id"), r.get::<String, _>("name")))
        .collect();

        let new_ids_set: std::collections::HashSet<Uuid> = new_field_ids.iter().copied().collect();
        let removed: Vec<(Uuid, String)> = current_fields
            .into_iter()
            .filter(|(id, _)| !new_ids_set.contains(id))
            .collect();

        if !removed.is_empty() {
            let removed_ids: Vec<Uuid> = removed.iter().map(|(id, _)| *id).collect();

            // Find removed fields that have non-null data in items
            let fields_with_data: Vec<String> = sqlx::query(
                r#"
                SELECT DISTINCT f.name
                FROM fields f
                WHERE f.id = ANY($1)
                  AND EXISTS (
                      SELECT 1 FROM items i
                      WHERE i.kind_id = $2
                        AND i.soft_fields ? f.name
                        AND (i.soft_fields ->> f.name) IS NOT NULL
                  )
                "#,
            )
            .bind(&removed_ids)
            .bind(kind_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(internal_error)?
            .into_iter()
            .map(|r| r.get::<String, _>("name"))
            .collect();

            if !fields_with_data.is_empty() && !q.force {
                tx.rollback().await.map_err(internal_error)?;
                return Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: "data_loss_required".to_string(),
                        message: format!(
                            "Removing fields [{}] would delete data from existing items. Pass force=true to confirm.",
                            fields_with_data.join(", ")
                        ),
                    }),
                ));
            }

            // Strip all removed fields from soft_fields (null keys included)
            for (_, field_name) in &removed {
                sqlx::query(
                    "UPDATE items SET soft_fields = soft_fields - $1 WHERE kind_id = $2 AND soft_fields ? $1",
                )
                .bind(field_name)
                .bind(kind_id)
                .execute(&mut *tx)
                .await
                .map_err(internal_error)?;
            }
        }

        // Replace the entire field set
        sqlx::query("DELETE FROM kind_fields WHERE kind_id = $1")
            .bind(kind_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;

        for (idx, field_id) in new_field_ids.iter().enumerate() {
            sqlx::query(
                "INSERT INTO kind_fields (kind_id, field_id, display_order) VALUES ($1, $2, $3)",
            )
            .bind(kind_id)
            .bind(field_id)
            .bind(idx as i32)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
        }
    }

    tx.commit().await.map_err(internal_error)?;

    let query = format!("{} AND k.id = $2 GROUP BY k.id", KIND_SELECT);
    let row = sqlx::query_as::<_, KindRow>(&query)
        .bind(org_id)
        .bind(kind_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    let kind = Kind::try_from(row).map_err(internal_error)?;
    Ok(Json(kind))
}

/// Delete an org-owned kind (fails if items reference it)
#[utoipa::path(
    delete,
    path = "/api/organizations/{org_id}/kinds/{kind_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("kind_id" = Uuid, Path, description = "Kind ID"),
    ),
    responses(
        (status = 204, description = "Kind deleted"),
        (status = 403, description = "Cannot delete a shared kind", body = ErrorResponse),
        (status = 404, description = "Kind not found", body = ErrorResponse),
        (status = 409, description = "Kind is in use by items", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "kinds"
)]
pub async fn delete_kind(
    State(state): State<AppState>,
    Path((org_id, kind_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    let row = sqlx::query("SELECT id, org_id FROM kinds WHERE id = $1")
        .bind(kind_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;

    let kind_org_id: Option<Uuid> = row.get("org_id");
    if kind_org_id.is_none() {
        return Err(forbidden("Cannot delete a shared kind"));
    }
    if kind_org_id != Some(org_id) {
        return Err(forbidden("Kind does not belong to this organization"));
    }

    let item_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM items WHERE kind_id = $1",
    )
    .bind(kind_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    if item_count > 0 {
        return Err(conflict(
            "kind_in_use",
            &format!("{} item(s) use this kind", item_count),
        ));
    }

    sqlx::query("DELETE FROM kinds WHERE id = $1 AND org_id = $2")
        .bind(kind_id)
        .bind(org_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Create an org-specific override of a shared kind
#[utoipa::path(
    post,
    path = "/api/organizations/{org_id}/kinds/{kind_id}/override",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("kind_id" = Uuid, Path, description = "Shared kind ID to override"),
    ),
    responses(
        (status = 201, description = "Override kind created", body = Kind),
        (status = 400, description = "Kind is not shared", body = ErrorResponse),
        (status = 409, description = "Override already exists", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "kinds"
)]
pub async fn override_kind(
    State(state): State<AppState>,
    Path((org_id, kind_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<Kind>), (StatusCode, Json<ErrorResponse>)> {
    // Fetch and verify it is a shared kind
    let shared_row = sqlx::query(
        "SELECT id, name, display_name FROM kinds WHERE id = $1 AND org_id IS NULL",
    )
    .bind(kind_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| bad_request("not_shared", "The specified kind is not a shared kind"))?;

    let shared_name: String = shared_row.get("name");
    let shared_display_name: Option<String> = shared_row.get("display_name");

    // Check if org already has a kind with this name
    let already_exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM kinds WHERE name = $1 AND org_id = $2)",
    )
    .bind(&shared_name)
    .bind(org_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    if already_exists {
        return Err(conflict(
            "override_exists",
            "This organization already has a kind with this name",
        ));
    }

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    let new_id: Uuid = sqlx::query_scalar(
        "INSERT INTO kinds (org_id, name, display_name) VALUES ($1, $2, $3) RETURNING id",
    )
    .bind(org_id)
    .bind(&shared_name)
    .bind(&shared_display_name)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_error)?;

    sqlx::query(
        "INSERT INTO kind_fields (kind_id, field_id, display_order)
         SELECT $1, field_id, display_order FROM kind_fields WHERE kind_id = $2",
    )
    .bind(new_id)
    .bind(kind_id)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;

    let query = format!("{} AND k.id = $2 GROUP BY k.id", KIND_SELECT);
    let row = sqlx::query_as::<_, KindRow>(&query)
        .bind(org_id)
        .bind(new_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    let kind = Kind::try_from(row).map_err(internal_error)?;
    Ok((StatusCode::CREATED, Json(kind)))
}

/// Revert an org override back to the shared kind
#[utoipa::path(
    post,
    path = "/api/organizations/{org_id}/kinds/{kind_id}/revert",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("kind_id" = Uuid, Path, description = "Org kind ID to revert"),
    ),
    responses(
        (status = 200, description = "Kind reverted to shared", body = RevertResponse),
        (status = 400, description = "Kind is already shared", body = ErrorResponse),
        (status = 404, description = "No matching shared kind found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "kinds"
)]
pub async fn revert_kind(
    State(state): State<AppState>,
    Path((org_id, kind_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RevertResponse>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch org kind
    let org_row = sqlx::query("SELECT id, org_id, name FROM kinds WHERE id = $1")
        .bind(kind_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;

    let kind_org_id: Option<Uuid> = org_row.get("org_id");
    let kind_name: String = org_row.get("name");

    if kind_org_id.is_none() {
        return Err(bad_request("already_shared", "Cannot revert a shared kind"));
    }
    if kind_org_id != Some(org_id) {
        return Err(forbidden("Kind does not belong to this organization"));
    }

    // Find the matching shared kind
    let shared_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM kinds WHERE name = $1 AND org_id IS NULL",
    )
    .bind(&kind_name)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "no_shared_kind".to_string(),
                message: "No shared kind found with this name to revert to".to_string(),
            }),
        )
    })?;

    // Determine orphaned fields: in org kind but not in shared kind
    let orphaned_field_names: Vec<String> = sqlx::query(
        r#"
        SELECT f.name
        FROM kind_fields kf
        JOIN fields f ON f.id = kf.field_id
        WHERE kf.kind_id = $1
          AND kf.field_id NOT IN (
              SELECT field_id FROM kind_fields WHERE kind_id = $2
          )
        "#,
    )
    .bind(kind_id)
    .bind(shared_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?
    .into_iter()
    .map(|r| r.get::<String, _>("name"))
    .collect();

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    let items_reassigned = sqlx::query(
        "UPDATE items SET kind_id = $1 WHERE kind_id = $2",
    )
    .bind(shared_id)
    .bind(kind_id)
    .execute(&mut *tx)
    .await
    .map_err(internal_error)?
    .rows_affected() as i64;

    sqlx::query("DELETE FROM kinds WHERE id = $1")
        .bind(kind_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;

    tx.commit().await.map_err(internal_error)?;

    Ok(Json(RevertResponse {
        items_reassigned,
        orphaned_field_names,
    }))
}

// ── Impact endpoint ──────────────────────────────────────────────────────────

#[derive(Debug, Serialize, ToSchema)]
pub struct FieldImpact {
    pub item_count: i64,
}

/// Return how many items would lose data if a field were removed from a kind
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/kinds/{kind_id}/fields/{field_id}/impact",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("kind_id" = Uuid, Path, description = "Kind ID"),
        ("field_id" = Uuid, Path, description = "Field ID"),
    ),
    responses(
        (status = 200, description = "Impact count", body = FieldImpact),
        (status = 404, description = "Field not part of this kind", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "kinds"
)]
pub async fn get_field_impact(
    State(state): State<AppState>,
    Path((org_id, kind_id, field_id)): Path<(Uuid, Uuid, Uuid)>,
) -> Result<Json<FieldImpact>, (StatusCode, Json<ErrorResponse>)> {
    // Verify field is part of this kind
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM kind_fields WHERE kind_id = $1 AND field_id = $2)",
    )
    .bind(kind_id)
    .bind(field_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    if !exists {
        return Err(not_found());
    }

    // Get the field name
    let field_name: String = sqlx::query_scalar("SELECT name FROM fields WHERE id = $1")
        .bind(field_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;

    // Count impacted items
    let item_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM items WHERE organization_id = $1 AND kind_id = $2 AND (soft_fields ->> $3) IS NOT NULL",
    )
    .bind(org_id)
    .bind(kind_id)
    .bind(&field_name)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(FieldImpact { item_count }))
}

// ── Error helpers ────────────────────────────────────────────────────────────

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
            message: "Kind not found".to_string(),
        }),
    )
}

fn bad_request(code: &str, msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::BAD_REQUEST,
        Json(ErrorResponse {
            error: code.to_string(),
            message: msg.to_string(),
        }),
    )
}

fn conflict(code: &str, msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse {
            error: code.to_string(),
            message: msg.to_string(),
        }),
    )
}

fn forbidden(msg: &str) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse {
            error: "forbidden".to_string(),
            message: msg.to_string(),
        }),
    )
}
