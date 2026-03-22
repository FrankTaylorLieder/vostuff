use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::api::{
    models::ErrorResponse,
    state::AppState,
};

// ── Public types ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum FieldType {
    String,
    Text,
    Date,
    Datetime,
    Number,
    Enum,
    Boolean,
}

impl FieldType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "string" => FieldType::String,
            "text" => FieldType::Text,
            "date" => FieldType::Date,
            "datetime" => FieldType::Datetime,
            "number" => FieldType::Number,
            "enum" => FieldType::Enum,
            "boolean" => FieldType::Boolean,
            _ => FieldType::String,
        }
    }

    fn is_enum(&self) -> bool {
        matches!(self, FieldType::Enum)
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct EnumValue {
    pub id: Uuid,
    pub value: String,
    pub display_value: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Field {
    pub id: Uuid,
    pub org_id: Option<Uuid>,
    pub name: String,
    pub display_name: Option<String>,
    pub field_type: FieldType,
    pub is_shared: bool,
    pub enum_values: Vec<EnumValue>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct EnumValueInput {
    pub value: String,
    pub display_value: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateFieldRequest {
    pub name: String,
    pub display_name: Option<String>,
    pub field_type: FieldType,
    #[serde(default)]
    pub enum_values: Vec<EnumValueInput>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateFieldRequest {
    pub display_name: Option<String>,
    /// If Some, replaces the entire enum_values set (enum fields only)
    pub enum_values: Option<Vec<EnumValueInput>>,
}

// ── Internal row types ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct EnumValueJson {
    id: Uuid,
    value: String,
    display_value: Option<String>,
    sort_order: i32,
}

#[derive(Debug, sqlx::FromRow)]
struct FieldRow {
    id: Uuid,
    org_id: Option<Uuid>,
    name: String,
    display_name: Option<String>,
    field_type: String,
    is_shared: bool,
    enum_values: serde_json::Value,
}

impl TryFrom<FieldRow> for Field {
    type Error = serde_json::Error;

    fn try_from(row: FieldRow) -> Result<Self, Self::Error> {
        let ev_jsons: Vec<EnumValueJson> = serde_json::from_value(row.enum_values)?;
        let enum_values = ev_jsons
            .into_iter()
            .map(|ev| EnumValue {
                id: ev.id,
                value: ev.value,
                display_value: ev.display_value,
                sort_order: ev.sort_order,
            })
            .collect();
        Ok(Field {
            id: row.id,
            org_id: row.org_id,
            name: row.name,
            display_name: row.display_name,
            field_type: FieldType::from_str(&row.field_type),
            is_shared: row.is_shared,
            enum_values,
        })
    }
}

// ── Core query ──────────────────────────────────────────────────────────────

const FIELD_SELECT: &str = "
    SELECT
        f.id, f.org_id, f.name, f.display_name, f.field_type::text,
        (f.org_id IS NULL) AS is_shared,
        COALESCE(
            json_agg(
                json_build_object(
                    'id',            ev.id,
                    'value',         ev.value,
                    'display_value', ev.display_value,
                    'sort_order',    ev.sort_order
                ) ORDER BY ev.sort_order
            ) FILTER (WHERE ev.id IS NOT NULL),
            '[]'::json
        ) AS enum_values
    FROM fields f
    LEFT JOIN enum_values ev ON ev.field_id = f.id
    WHERE (f.org_id IS NULL OR f.org_id = $1)";

// ── Handlers ────────────────────────────────────────────────────────────────

/// List all fields visible to an org (shared + org-owned)
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/fields",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    responses(
        (status = 200, description = "List of fields", body = Vec<Field>),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "fields"
)]
pub async fn list_fields(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<Vec<Field>>, (StatusCode, Json<ErrorResponse>)> {
    let query = format!(
        "{} GROUP BY f.id ORDER BY f.display_name NULLS LAST, f.name",
        FIELD_SELECT
    );
    let rows = sqlx::query_as::<_, FieldRow>(&query)
        .bind(org_id)
        .fetch_all(&state.pool)
        .await
        .map_err(internal_error)?;

    let fields: Vec<Field> = rows
        .into_iter()
        .map(Field::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(internal_error)?;

    Ok(Json(fields))
}

/// Get a single field by ID
#[utoipa::path(
    get,
    path = "/api/organizations/{org_id}/fields/{field_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("field_id" = Uuid, Path, description = "Field ID"),
    ),
    responses(
        (status = 200, description = "Field details", body = Field),
        (status = 404, description = "Field not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "fields"
)]
pub async fn get_field(
    State(state): State<AppState>,
    Path((org_id, field_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<Field>, (StatusCode, Json<ErrorResponse>)> {
    let query = format!("{} AND f.id = $2 GROUP BY f.id", FIELD_SELECT);
    let row = sqlx::query_as::<_, FieldRow>(&query)
        .bind(org_id)
        .bind(field_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;

    let field = Field::try_from(row).map_err(internal_error)?;
    Ok(Json(field))
}

/// Create a new org-owned field
#[utoipa::path(
    post,
    path = "/api/organizations/{org_id}/fields",
    params(("org_id" = Uuid, Path, description = "Organization ID")),
    request_body = CreateFieldRequest,
    responses(
        (status = 201, description = "Field created", body = Field),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 409, description = "Name already in use", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "fields"
)]
pub async fn create_field(
    State(state): State<AppState>,
    Path(org_id): Path<Uuid>,
    Json(req): Json<CreateFieldRequest>,
) -> Result<(StatusCode, Json<Field>), (StatusCode, Json<ErrorResponse>)> {
    // Check shared name conflict
    let shared_conflict: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM fields WHERE name = $1 AND org_id IS NULL)",
    )
    .bind(&req.name)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    if shared_conflict {
        return Err(conflict(
            "name_conflict",
            "A shared field with this name already exists",
        ));
    }

    // Check org name conflict
    let org_conflict: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM fields WHERE name = $1 AND org_id = $2)",
    )
    .bind(&req.name)
    .bind(org_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    if org_conflict {
        return Err(conflict(
            "name_conflict",
            "A field with this name already exists in this organization",
        ));
    }

    // Validate enum values only allowed for enum fields
    if !req.field_type.is_enum() && !req.enum_values.is_empty() {
        return Err(bad_request(
            "invalid_enum_values",
            "enum_values can only be provided for enum fields",
        ));
    }

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    let field_type_str = serde_json::to_string(&req.field_type)
        .map_err(internal_error)?
        .trim_matches('"')
        .to_string();

    let new_id: Uuid = sqlx::query_scalar(
        "INSERT INTO fields (org_id, name, display_name, field_type) VALUES ($1, $2, $3, $4::field_type) RETURNING id",
    )
    .bind(org_id)
    .bind(&req.name)
    .bind(&req.display_name)
    .bind(&field_type_str)
    .fetch_one(&mut *tx)
    .await
    .map_err(internal_error)?;

    for ev in &req.enum_values {
        sqlx::query(
            "INSERT INTO enum_values (field_id, value, display_value, sort_order) VALUES ($1, $2, $3, $4)",
        )
        .bind(new_id)
        .bind(&ev.value)
        .bind(&ev.display_value)
        .bind(ev.sort_order)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    tx.commit().await.map_err(internal_error)?;

    let query = format!("{} AND f.id = $2 GROUP BY f.id", FIELD_SELECT);
    let row = sqlx::query_as::<_, FieldRow>(&query)
        .bind(org_id)
        .bind(new_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    let field = Field::try_from(row).map_err(internal_error)?;
    Ok((StatusCode::CREATED, Json(field)))
}

/// Update a field's display name or enum values
#[utoipa::path(
    patch,
    path = "/api/organizations/{org_id}/fields/{field_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("field_id" = Uuid, Path, description = "Field ID"),
    ),
    request_body = UpdateFieldRequest,
    responses(
        (status = 200, description = "Field updated", body = Field),
        (status = 400, description = "Invalid input", body = ErrorResponse),
        (status = 403, description = "Cannot modify a shared field", body = ErrorResponse),
        (status = 404, description = "Field not found", body = ErrorResponse),
        (status = 409, description = "Enum value in use by items", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "fields"
)]
pub async fn update_field(
    State(state): State<AppState>,
    Path((org_id, field_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateFieldRequest>,
) -> Result<Json<Field>, (StatusCode, Json<ErrorResponse>)> {
    // Fetch the field and verify ownership
    let row = sqlx::query(
        "SELECT id, org_id, name, field_type::text AS field_type FROM fields WHERE id = $1",
    )
    .bind(field_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or_else(not_found)?;

    use sqlx::Row;
    let field_org_id: Option<Uuid> = row.get("org_id");
    let field_name: String = row.get("name");
    let field_type_str: String = row.get("field_type");

    if field_org_id.is_none() {
        return Err(forbidden("Cannot modify a shared field"));
    }
    if field_org_id != Some(org_id) {
        return Err(forbidden("Field does not belong to this organization"));
    }

    let mut tx = state.pool.begin().await.map_err(internal_error)?;

    if let Some(ref display_name) = req.display_name {
        sqlx::query(
            "UPDATE fields SET display_name = $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(display_name)
        .bind(field_id)
        .execute(&mut *tx)
        .await
        .map_err(internal_error)?;
    }

    if let Some(ref new_values) = req.enum_values {
        let ft = FieldType::from_str(&field_type_str);
        if !ft.is_enum() {
            tx.rollback().await.map_err(internal_error)?;
            return Err(bad_request(
                "not_enum_field",
                "enum_values can only be set on enum fields",
            ));
        }

        // Fetch current values
        let current_values: Vec<String> = sqlx::query_scalar(
            "SELECT value FROM enum_values WHERE field_id = $1",
        )
        .bind(field_id)
        .fetch_all(&mut *tx)
        .await
        .map_err(internal_error)?;

        let new_value_set: std::collections::HashSet<&str> =
            new_values.iter().map(|v| v.value.as_str()).collect();
        let removed: Vec<String> = current_values
            .into_iter()
            .filter(|v| !new_value_set.contains(v.as_str()))
            .collect();

        if !removed.is_empty() {
            // Hard-block check: find removed values that are in use by items
            let blocked: Vec<(String, i64)> = sqlx::query(
                r#"
                SELECT ev.value, COUNT(i.id) AS item_count
                FROM enum_values ev
                JOIN kind_fields kf ON kf.field_id = ev.field_id
                JOIN items i        ON i.kind_id = kf.kind_id
                                   AND (i.soft_fields ->> $1) = ev.value
                WHERE ev.field_id = $2
                  AND ev.value = ANY($3)
                GROUP BY ev.value
                HAVING COUNT(i.id) > 0
                "#,
            )
            .bind(&field_name)
            .bind(field_id)
            .bind(&removed)
            .fetch_all(&mut *tx)
            .await
            .map_err(internal_error)?
            .into_iter()
            .map(|r| {
                (
                    r.get::<String, _>("value"),
                    r.get::<i64, _>("item_count"),
                )
            })
            .collect();

            if !blocked.is_empty() {
                tx.rollback().await.map_err(internal_error)?;
                let detail = blocked
                    .iter()
                    .map(|(v, c)| format!("{} ({} items)", v, c))
                    .collect::<Vec<_>>()
                    .join(", ");
                return Err((
                    StatusCode::CONFLICT,
                    Json(ErrorResponse {
                        error: "enum_value_in_use".to_string(),
                        message: format!(
                            "Cannot remove enum values that are assigned to items: {}",
                            detail
                        ),
                    }),
                ));
            }
        }

        // Replace enum values
        sqlx::query("DELETE FROM enum_values WHERE field_id = $1")
            .bind(field_id)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;

        for ev in new_values {
            sqlx::query(
                "INSERT INTO enum_values (field_id, value, display_value, sort_order) VALUES ($1, $2, $3, $4)",
            )
            .bind(field_id)
            .bind(&ev.value)
            .bind(&ev.display_value)
            .bind(ev.sort_order)
            .execute(&mut *tx)
            .await
            .map_err(internal_error)?;
        }
    }

    tx.commit().await.map_err(internal_error)?;

    let query = format!("{} AND f.id = $2 GROUP BY f.id", FIELD_SELECT);
    let row = sqlx::query_as::<_, FieldRow>(&query)
        .bind(org_id)
        .bind(field_id)
        .fetch_one(&state.pool)
        .await
        .map_err(internal_error)?;

    let field = Field::try_from(row).map_err(internal_error)?;
    Ok(Json(field))
}

/// Delete an org-owned field (fails if it belongs to any kind)
#[utoipa::path(
    delete,
    path = "/api/organizations/{org_id}/fields/{field_id}",
    params(
        ("org_id" = Uuid, Path, description = "Organization ID"),
        ("field_id" = Uuid, Path, description = "Field ID"),
    ),
    responses(
        (status = 204, description = "Field deleted"),
        (status = 403, description = "Cannot delete a shared field", body = ErrorResponse),
        (status = 404, description = "Field not found", body = ErrorResponse),
        (status = 409, description = "Field is in use by kinds", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    tag = "fields"
)]
pub async fn delete_field(
    State(state): State<AppState>,
    Path((org_id, field_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponse>)> {
    use sqlx::Row;

    let row = sqlx::query("SELECT id, org_id FROM fields WHERE id = $1")
        .bind(field_id)
        .fetch_optional(&state.pool)
        .await
        .map_err(internal_error)?
        .ok_or_else(not_found)?;

    let field_org_id: Option<Uuid> = row.get("org_id");
    if field_org_id.is_none() {
        return Err(forbidden("Cannot delete a shared field"));
    }
    if field_org_id != Some(org_id) {
        return Err(forbidden("Field does not belong to this organization"));
    }

    let kind_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM kind_fields WHERE field_id = $1",
    )
    .bind(field_id)
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    if kind_count > 0 {
        return Err(conflict(
            "field_in_use",
            &format!("{} kind(s) reference this field", kind_count),
        ));
    }

    sqlx::query("DELETE FROM fields WHERE id = $1 AND org_id = $2")
        .bind(field_id)
        .bind(org_id)
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;

    Ok(StatusCode::NO_CONTENT)
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
            message: "Field not found".to_string(),
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
