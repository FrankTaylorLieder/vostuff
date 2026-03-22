mod common;

use axum::http::StatusCode;
use common::TestFixture;
use serde_json::{json, Value};
use uuid::Uuid;

// Fixed UUIDs from seed migration
const VINYL_KIND_ID: &str = "00000000-0000-0000-0000-000000000001";
const BOOK_KIND_ID: &str = "00000000-0000-0000-0000-000000000004";
const SIZE_FIELD_ID: &str = "00000000-0000-0000-0001-000000000001";

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

async fn create_kind(f: &TestFixture, name: &str, field_ids: &[Uuid]) -> Uuid {
    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/kinds", f.org1_id),
            &json!({"name": name, "field_ids": field_ids}),
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
async fn test_list_kinds_includes_shared() {
    let f = TestFixture::new().await;

    let res = f
        .ctx
        .get(
            &format!("/api/organizations/{}/kinds", f.org1_id),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    let kinds = res.body.as_array().unwrap();
    // Seed migration inserts 8 shared kinds
    assert!(kinds.len() >= 8, "expected at least 8 shared kinds, got {}", kinds.len());

    let any_shared = kinds.iter().any(|k| k["is_shared"].as_bool() == Some(true));
    assert!(any_shared, "expected at least one shared kind");
}

#[tokio::test]
async fn test_get_shared_kind_with_fields() {
    let f = TestFixture::new().await;
    let vinyl_id = Uuid::parse_str(VINYL_KIND_ID).unwrap();

    let res = f
        .ctx
        .get(
            &format!("/api/organizations/{}/kinds/{}", f.org1_id, vinyl_id),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    assert_eq!(res.body["name"], "vinyl");
    assert_eq!(res.body["is_shared"], true);

    let fields = res.body["fields"].as_array().unwrap();
    assert!(!fields.is_empty(), "vinyl kind should have fields");

    // size field should be present with enum_values
    let size_field = fields.iter().find(|f| f["name"].as_str() == Some("size")).unwrap();
    let evs = size_field["enum_values"].as_array().unwrap();
    assert!(!evs.is_empty(), "size field should have enum values");
}

#[tokio::test]
async fn test_get_kind_not_found() {
    let f = TestFixture::new().await;

    let res = f
        .ctx
        .get(
            &format!("/api/organizations/{}/kinds/{}", f.org1_id, Uuid::new_v4()),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_create_kind_with_fields() {
    let f = TestFixture::new().await;
    let field_id = create_field(&f, "author", "string").await;
    let size_id = Uuid::parse_str(SIZE_FIELD_ID).unwrap();

    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/kinds", f.org1_id),
            &json!({
                "name": "periodical",
                "display_name": "Periodical",
                "field_ids": [field_id, size_id]
            }),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CREATED);
    assert_eq!(res.body["name"], "periodical");
    assert_eq!(res.body["display_name"], "Periodical");
    assert_eq!(res.body["is_shared"], false);
    let fields = res.body["fields"].as_array().unwrap();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0]["display_order"], 0);
    assert_eq!(fields[1]["display_order"], 1);
}

#[tokio::test]
async fn test_create_kind_name_conflicts_with_shared() {
    let f = TestFixture::new().await;

    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/kinds", f.org1_id),
            &json!({"name": "vinyl", "field_ids": []}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CONFLICT);
    assert_eq!(res.body["error"], "name_conflict");
}

#[tokio::test]
async fn test_create_kind_name_conflicts_within_org() {
    let f = TestFixture::new().await;
    create_kind(&f, "ephemera", &[]).await;

    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/kinds", f.org1_id),
            &json!({"name": "ephemera", "field_ids": []}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_update_kind_display_name() {
    let f = TestFixture::new().await;
    let kind_id = create_kind(&f, "maps", &[]).await;

    let res = f
        .ctx
        .patch(
            &format!("/api/organizations/{}/kinds/{}", f.org1_id, kind_id),
            &json!({"display_name": "Maps & Charts"}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    assert_eq!(res.body["display_name"], "Maps & Charts");
    assert_eq!(res.body["name"], "maps");
}

#[tokio::test]
async fn test_update_kind_add_field() {
    let f = TestFixture::new().await;
    let kind_id = create_kind(&f, "prints", &[]).await;
    let field_id = create_field(&f, "medium", "string").await;

    let res = f
        .ctx
        .patch(
            &format!("/api/organizations/{}/kinds/{}", f.org1_id, kind_id),
            &json!({"field_ids": [field_id]}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    let fields = res.body["fields"].as_array().unwrap();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0]["name"], "medium");
}

#[tokio::test]
async fn test_update_kind_remove_field_without_data_succeeds() {
    let f = TestFixture::new().await;
    let field_a = create_field(&f, "artist", "string").await;
    let field_b = create_field(&f, "label", "string").await;
    let kind_id = create_kind(&f, "releases", &[field_a, field_b]).await;

    // Remove field_a (no items exist)
    let res = f
        .ctx
        .patch(
            &format!("/api/organizations/{}/kinds/{}", f.org1_id, kind_id),
            &json!({"field_ids": [field_b]}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    let fields = res.body["fields"].as_array().unwrap();
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0]["name"], "label");
}

#[tokio::test]
async fn test_update_kind_remove_field_with_data_blocked_without_force() {
    let f = TestFixture::new().await;
    let field_id = create_field(&f, "publisher", "string").await;
    let kind_id = create_kind(&f, "books2", &[field_id]).await;
    create_item(&f, kind_id, "My Book", json!({"publisher": "Penguin"})).await;

    // Try to remove the field without force
    let res = f
        .ctx
        .patch(
            &format!("/api/organizations/{}/kinds/{}?force=false", f.org1_id, kind_id),
            &json!({"field_ids": []}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CONFLICT);
    assert_eq!(res.body["error"], "data_loss_required");
}

#[tokio::test]
async fn test_update_kind_remove_field_with_data_succeeds_with_force() {
    let f = TestFixture::new().await;
    let field_id = create_field(&f, "imprint", "string").await;
    let kind_id = create_kind(&f, "novels", &[field_id]).await;
    let item_id = create_item(&f, kind_id, "A Novel", json!({"imprint": "HarperCollins"})).await;

    // Remove with force=true
    let res = f
        .ctx
        .patch(
            &format!("/api/organizations/{}/kinds/{}?force=true", f.org1_id, kind_id),
            &json!({"field_ids": []}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    let fields = res.body["fields"].as_array().unwrap();
    assert!(fields.is_empty(), "field should have been removed");

    // Verify the item's soft_fields no longer contains the key
    let item_res = f
        .ctx
        .get(
            &format!("/api/organizations/{}/items/{}", f.org1_id, item_id),
            Some(&f.user1_token),
        )
        .await;
    item_res.assert_success();
    assert!(
        item_res.body["soft_fields"]["imprint"].is_null(),
        "soft_fields should no longer have 'imprint'"
    );
}

#[tokio::test]
async fn test_update_shared_kind_forbidden() {
    let f = TestFixture::new().await;
    let vinyl_id = Uuid::parse_str(VINYL_KIND_ID).unwrap();

    let res = f
        .ctx
        .patch(
            &format!("/api/organizations/{}/kinds/{}", f.org1_id, vinyl_id),
            &json!({"display_name": "My Vinyl"}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_delete_kind_with_no_items() {
    let f = TestFixture::new().await;
    let kind_id = create_kind(&f, "maps2", &[]).await;

    let del = f
        .ctx
        .delete(
            &format!("/api/organizations/{}/kinds/{}", f.org1_id, kind_id),
            Some(&f.user1_token),
        )
        .await;
    del.assert_status(StatusCode::NO_CONTENT);

    let get = f
        .ctx
        .get(
            &format!("/api/organizations/{}/kinds/{}", f.org1_id, kind_id),
            Some(&f.user1_token),
        )
        .await;
    get.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_kind_with_items_is_blocked() {
    let f = TestFixture::new().await;
    let kind_id = create_kind(&f, "posters", &[]).await;
    create_item(&f, kind_id, "Woodstock Poster", json!({})).await;

    let res = f
        .ctx
        .delete(
            &format!("/api/organizations/{}/kinds/{}", f.org1_id, kind_id),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CONFLICT);
    assert_eq!(res.body["error"], "kind_in_use");
}

#[tokio::test]
async fn test_delete_shared_kind_forbidden() {
    let f = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();

    let res = f
        .ctx
        .delete(
            &format!("/api/organizations/{}/kinds/{}", f.org1_id, book_id),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_override_kind_creates_org_copy() {
    let f = TestFixture::new().await;
    let vinyl_id = Uuid::parse_str(VINYL_KIND_ID).unwrap();

    let res = f
        .ctx
        .post(
            &format!(
                "/api/organizations/{}/kinds/{}/override",
                f.org1_id, vinyl_id
            ),
            &json!({}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::CREATED);
    assert_eq!(res.body["name"], "vinyl");
    assert_eq!(res.body["is_shared"], false);
    assert_ne!(res.body["id"].as_str().unwrap(), VINYL_KIND_ID);

    // Fields should be copied
    let fields = res.body["fields"].as_array().unwrap();
    let vinyl_fields_count = 6; // size, speed, channels, disks, media_grading, sleeve_grading
    assert_eq!(fields.len(), vinyl_fields_count);
}

#[tokio::test]
async fn test_revert_kind() {
    let f = TestFixture::new().await;
    let vinyl_id = Uuid::parse_str(VINYL_KIND_ID).unwrap();

    // Override first
    let override_res = f
        .ctx
        .post(
            &format!(
                "/api/organizations/{}/kinds/{}/override",
                f.org1_id, vinyl_id
            ),
            &json!({}),
            Some(&f.user1_token),
        )
        .await;
    override_res.assert_status(StatusCode::CREATED);
    let org_kind_id: Uuid = override_res.body["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    // Create an item under the org kind
    create_item(&f, org_kind_id, "Dark Side LP", json!({})).await;

    // Revert
    let revert_res = f
        .ctx
        .post(
            &format!(
                "/api/organizations/{}/kinds/{}/revert",
                f.org1_id, org_kind_id
            ),
            &json!({}),
            Some(&f.user1_token),
        )
        .await;

    revert_res.assert_success();
    assert_eq!(revert_res.body["items_reassigned"], 1);

    // The org kind should be gone
    let get = f
        .ctx
        .get(
            &format!(
                "/api/organizations/{}/kinds/{}",
                f.org1_id, org_kind_id
            ),
            Some(&f.user1_token),
        )
        .await;
    get.assert_status(StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_field_impact_zero() {
    let f = TestFixture::new().await;
    let vinyl_id = Uuid::parse_str(VINYL_KIND_ID).unwrap();
    let size_id = Uuid::parse_str(SIZE_FIELD_ID).unwrap();

    // No items yet → impact is 0
    let res = f
        .ctx
        .get(
            &format!(
                "/api/organizations/{}/kinds/{}/fields/{}/impact",
                f.org1_id, vinyl_id, size_id
            ),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    assert_eq!(res.body["item_count"], 0);
}

#[tokio::test]
async fn test_get_field_impact_with_data() {
    let f = TestFixture::new().await;
    let vinyl_id = Uuid::parse_str(VINYL_KIND_ID).unwrap();
    let size_id = Uuid::parse_str(SIZE_FIELD_ID).unwrap();

    // Create two items with "size" set, one without
    create_item(&f, vinyl_id, "Abbey Road",  json!({"size": "12_inch"})).await;
    create_item(&f, vinyl_id, "Back in Black", json!({"size": "12_inch"})).await;
    create_item(&f, vinyl_id, "No Size", json!({})).await;

    let res = f
        .ctx
        .get(
            &format!(
                "/api/organizations/{}/kinds/{}/fields/{}/impact",
                f.org1_id, vinyl_id, size_id
            ),
            Some(&f.user1_token),
        )
        .await;

    res.assert_success();
    assert_eq!(res.body["item_count"], 2);
}

#[tokio::test]
async fn test_get_field_impact_field_not_in_kind() {
    let f = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();
    // "size" field is not part of the book kind
    let size_id = Uuid::parse_str(SIZE_FIELD_ID).unwrap();

    let res = f
        .ctx
        .get(
            &format!(
                "/api/organizations/{}/kinds/{}/fields/{}/impact",
                f.org1_id, book_id, size_id
            ),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::NOT_FOUND);
}
