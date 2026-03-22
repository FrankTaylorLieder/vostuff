mod common;

use axum::http::StatusCode;
use common::TestFixture;
use serde_json::{json, Value};
use uuid::Uuid;

// Fixed UUIDs from seed migration
const SIZE_FIELD_ID: &str = "00000000-0000-0000-0001-000000000001";
const DISKS_FIELD_ID: &str = "00000000-0000-0000-0001-000000000006";

// ── Test helpers ─────────────────────────────────────────────────────────────

async fn create_field(f: &TestFixture, name: &str, field_type: &str) -> Uuid {
    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/fields", f.org1_id),
            &json!({"name": name, "field_type": field_type}),
            Some(&f.user1_token),
        )
        .await;
    res.assert_status(StatusCode::CREATED);
    res.body["id"].as_str().unwrap().parse().unwrap()
}

async fn create_enum_field(f: &TestFixture, name: &str, values: &[&str]) -> Uuid {
    let enum_values: Vec<Value> = values
        .iter()
        .enumerate()
        .map(|(i, v)| json!({"value": v, "sort_order": i as i32}))
        .collect();

    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/fields", f.org1_id),
            &json!({"name": name, "field_type": "enum", "enum_values": enum_values}),
            Some(&f.user1_token),
        )
        .await;
    res.assert_status(StatusCode::CREATED);
    res.body["id"].as_str().unwrap().parse().unwrap()
}

async fn create_kind_with_field(f: &TestFixture, kind_name: &str, field_id: Uuid) -> Uuid {
    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/kinds", f.org1_id),
            &json!({"name": kind_name, "field_ids": [field_id]}),
            Some(&f.user1_token),
        )
        .await;
    res.assert_status(StatusCode::CREATED);
    res.body["id"].as_str().unwrap().parse().unwrap()
}

async fn create_item(f: &TestFixture, kind_id: Uuid, name: &str, soft_fields: Value) -> Uuid {
    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/items", f.org1_id),
            &json!({"kind_id": kind_id, "name": name, "soft_fields": soft_fields}),
            Some(&f.user1_token),
        )
        .await;
    res.assert_success();
    res.body["id"].as_str().unwrap().parse().unwrap()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_list_fields_includes_shared() {
    let f = TestFixture::new().await;

    let res = f
        .ctx
        .get(
            &format!("/api/organizations/{}/fields", f.org1_id),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    let fields = res.body.as_array().unwrap();
    // Seed migration inserts 7 shared fields
    assert!(fields.len() >= 7, "expected at least 7 shared fields, got {}", fields.len());

    // Verify at least one shared field is present with is_shared=true
    let any_shared = fields.iter().any(|f| f["is_shared"].as_bool() == Some(true));
    assert!(any_shared, "expected at least one shared field");
}

#[tokio::test]
async fn test_get_shared_field_with_enum_values() {
    let f = TestFixture::new().await;
    let size_id = Uuid::parse_str(SIZE_FIELD_ID).unwrap();

    let res = f
        .ctx
        .get(
            &format!("/api/organizations/{}/fields/{}", f.org1_id, size_id),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    assert_eq!(res.body["name"], "size");
    assert_eq!(res.body["field_type"], "enum");
    assert_eq!(res.body["is_shared"], true);

    let enum_values = res.body["enum_values"].as_array().unwrap();
    assert_eq!(enum_values.len(), 3); // 12_inch, 6_inch, other
    assert_eq!(enum_values[0]["value"], "12_inch");
}

#[tokio::test]
async fn test_get_field_not_found() {
    let f = TestFixture::new().await;

    let res = f
        .ctx
        .get(
            &format!(
                "/api/organizations/{}/fields/{}",
                f.org1_id,
                Uuid::new_v4()
            ),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_string_field() {
    let f = TestFixture::new().await;

    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/fields", f.org1_id),
            &json!({"name": "edition", "display_name": "Edition", "field_type": "string"}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CREATED);
    assert_eq!(res.body["name"], "edition");
    assert_eq!(res.body["display_name"], "Edition");
    assert_eq!(res.body["field_type"], "string");
    assert_eq!(res.body["is_shared"], false);
    assert_eq!(res.body["org_id"], f.org1_id.to_string());
    let enum_values = res.body["enum_values"].as_array().unwrap();
    assert!(enum_values.is_empty());
}

#[tokio::test]
async fn test_create_enum_field_with_values() {
    let f = TestFixture::new().await;

    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/fields", f.org1_id),
            &json!({
                "name": "condition",
                "field_type": "enum",
                "enum_values": [
                    {"value": "new",  "display_value": "New",  "sort_order": 1},
                    {"value": "good", "display_value": "Good", "sort_order": 2},
                    {"value": "fair", "display_value": "Fair", "sort_order": 3}
                ]
            }),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CREATED);
    assert_eq!(res.body["field_type"], "enum");
    let evs = res.body["enum_values"].as_array().unwrap();
    assert_eq!(evs.len(), 3);
    assert_eq!(evs[0]["value"], "new");
    assert_eq!(evs[0]["display_value"], "New");
}

#[tokio::test]
async fn test_create_field_conflicts_with_shared_name() {
    let f = TestFixture::new().await;

    // "size" is a shared field name
    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/fields", f.org1_id),
            &json!({"name": "size", "field_type": "string"}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CONFLICT);
    assert_eq!(res.body["error"], "name_conflict");
}

#[tokio::test]
async fn test_create_field_conflicts_within_org() {
    let f = TestFixture::new().await;

    // First create succeeds
    create_field(&f, "rating", "number").await;

    // Second with same name fails
    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/fields", f.org1_id),
            &json!({"name": "rating", "field_type": "string"}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CONFLICT);
    assert_eq!(res.body["error"], "name_conflict");
}

#[tokio::test]
async fn test_create_non_enum_field_with_enum_values_is_rejected() {
    let f = TestFixture::new().await;

    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/fields", f.org1_id),
            &json!({
                "name": "notes_field",
                "field_type": "text",
                "enum_values": [{"value": "x", "sort_order": 1}]
            }),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"], "invalid_enum_values");
}

#[tokio::test]
async fn test_update_field_display_name() {
    let f = TestFixture::new().await;
    let field_id = create_field(&f, "pub_year", "number").await;

    let res = f
        .ctx
        .patch(
            &format!("/api/organizations/{}/fields/{}", f.org1_id, field_id),
            &json!({"display_name": "Publication Year"}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    assert_eq!(res.body["display_name"], "Publication Year");
    assert_eq!(res.body["name"], "pub_year"); // name unchanged
}

#[tokio::test]
async fn test_update_enum_field_replace_values() {
    let f = TestFixture::new().await;
    let field_id = create_enum_field(&f, "fmt", &["cd", "vinyl"]).await;

    // Replace with a new set
    let res = f
        .ctx
        .patch(
            &format!("/api/organizations/{}/fields/{}", f.org1_id, field_id),
            &json!({
                "enum_values": [
                    {"value": "vinyl",  "sort_order": 1},
                    {"value": "tape",   "sort_order": 2},
                    {"value": "stream", "sort_order": 3}
                ]
            }),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    let evs = res.body["enum_values"].as_array().unwrap();
    let values: Vec<&str> = evs.iter().map(|e| e["value"].as_str().unwrap()).collect();
    assert!(values.contains(&"vinyl"));
    assert!(values.contains(&"tape"));
    assert!(values.contains(&"stream"));
    assert!(!values.contains(&"cd")); // removed
}

#[tokio::test]
async fn test_update_enum_field_remove_value_in_use_is_blocked() {
    let f = TestFixture::new().await;
    let field_id = create_enum_field(&f, "cond", &["new", "used"]).await;
    let kind_id = create_kind_with_field(&f, "goods", field_id).await;
    create_item(&f, kind_id, "Widget", json!({"cond": "new"})).await;

    // Try to remove "new" which is in use
    let res = f
        .ctx
        .patch(
            &format!("/api/organizations/{}/fields/{}", f.org1_id, field_id),
            &json!({
                "enum_values": [
                    {"value": "used", "sort_order": 1}
                ]
            }),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CONFLICT);
    assert_eq!(res.body["error"], "enum_value_in_use");
    // Error message should mention the blocked value
    let msg = res.body["message"].as_str().unwrap();
    assert!(msg.contains("new"), "message should mention 'new': {}", msg);
}

#[tokio::test]
async fn test_update_shared_field_forbidden() {
    let f = TestFixture::new().await;
    let size_id = Uuid::parse_str(SIZE_FIELD_ID).unwrap();

    let res = f
        .ctx
        .patch(
            &format!("/api/organizations/{}/fields/{}", f.org1_id, size_id),
            &json!({"display_name": "New Label"}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_update_non_enum_field_with_enum_values_is_rejected() {
    let f = TestFixture::new().await;
    let field_id = create_field(&f, "page_count", "number").await;

    let res = f
        .ctx
        .patch(
            &format!("/api/organizations/{}/fields/{}", f.org1_id, field_id),
            &json!({
                "enum_values": [{"value": "x", "sort_order": 1}]
            }),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::BAD_REQUEST);
    assert_eq!(res.body["error"], "not_enum_field");
}

#[tokio::test]
async fn test_delete_field() {
    let f = TestFixture::new().await;
    let field_id = create_field(&f, "subtitle", "string").await;

    let del = f
        .ctx
        .delete(
            &format!("/api/organizations/{}/fields/{}", f.org1_id, field_id),
            Some(&f.user1_token),
        )
        .await;
    del.assert_status(StatusCode::NO_CONTENT);

    // Should 404 now
    let get = f
        .ctx
        .get(
            &format!("/api/organizations/{}/fields/{}", f.org1_id, field_id),
            Some(&f.user1_token),
        )
        .await;
    get.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_field_in_use_by_kind_is_blocked() {
    let f = TestFixture::new().await;
    let field_id = create_field(&f, "genre", "string").await;
    create_kind_with_field(&f, "media", field_id).await;

    let res = f
        .ctx
        .delete(
            &format!("/api/organizations/{}/fields/{}", f.org1_id, field_id),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CONFLICT);
    assert_eq!(res.body["error"], "field_in_use");
}

#[tokio::test]
async fn test_delete_shared_field_forbidden() {
    let f = TestFixture::new().await;
    let disks_id = Uuid::parse_str(DISKS_FIELD_ID).unwrap();

    let res = f
        .ctx
        .delete(
            &format!("/api/organizations/{}/fields/{}", f.org1_id, disks_id),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_list_fields_shows_org_fields() {
    let f = TestFixture::new().await;
    let _field_id = create_field(&f, "my_custom_field", "boolean").await;

    let res = f
        .ctx
        .get(
            &format!("/api/organizations/{}/fields", f.org1_id),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    let fields = res.body.as_array().unwrap();
    let has_custom = fields
        .iter()
        .any(|f| f["name"].as_str() == Some("my_custom_field"));
    assert!(has_custom, "org field should appear in list");
}
