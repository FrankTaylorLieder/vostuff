mod common;

use axum::http::StatusCode;
use common::TestFixture;
use serde_json::json;
use uuid::Uuid;
use vostuff_core::auth::SYSTEM_ORG_ID;

// Book is a shared kind from the seed migration, usable by any org.
const BOOK_KIND_ID: &str = "00000000-0000-0000-0000-000000000004";

// ── Org isolation / authentication ───────────────────────────────────────────

#[tokio::test]
async fn test_unauthenticated_request_rejected() {
    let f = TestFixture::new().await;

    let res = f
        .ctx
        .get(&format!("/api/organizations/{}/items", f.org1_id), None)
        .await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_cross_org_access_forbidden() {
    let f = TestFixture::new().await;

    // user1 belongs to org1, but requests org2's items.
    let res = f
        .ctx
        .get(
            &format!("/api/organizations/{}/items", f.org2_id),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::FORBIDDEN);
}

// ── Member (USER) permissions ────────────────────────────────────────────────

#[tokio::test]
async fn test_member_can_read_and_crud_items() {
    let f = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();

    // A plain USER may read items...
    f.ctx
        .get(
            &format!("/api/organizations/{}/items", f.org1_id),
            Some(&f.user1_token),
        )
        .await
        .assert_success();

    // ...and create them.
    f.ctx
        .post(
            &format!("/api/organizations/{}/items", f.org1_id),
            &json!({"kind_id": book_id, "name": "A book"}),
            Some(&f.user1_token),
        )
        .await
        .assert_success();
}

#[tokio::test]
async fn test_member_cannot_manage_kinds() {
    let f = TestFixture::new().await;

    // user1 is USER (not ADMIN) in org1.
    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/kinds", f.org1_id),
            &json!({"name": "widget", "field_ids": []}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_member_cannot_create_location() {
    let f = TestFixture::new().await;

    let res = f
        .ctx
        .post(
            &format!("/api/organizations/{}/locations", f.org1_id),
            &json!({"name": "Shelf A"}),
            Some(&f.user1_token),
        )
        .await;

    res.assert_status(StatusCode::FORBIDDEN);
}

// ── Org ADMIN permissions ────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_can_manage_kinds_and_locations() {
    let f = TestFixture::new().await;

    // user2 is ADMIN in org1.
    f.ctx
        .post(
            &format!("/api/organizations/{}/kinds", f.org1_id),
            &json!({"name": "widget", "field_ids": []}),
            Some(&f.user2_token),
        )
        .await
        .assert_status(StatusCode::CREATED);

    f.ctx
        .post(
            &format!("/api/organizations/{}/locations", f.org1_id),
            &json!({"name": "Shelf A"}),
            Some(&f.user2_token),
        )
        .await
        .assert_status(StatusCode::CREATED);
}

// ── System administration (/admin/*) ─────────────────────────────────────────

#[tokio::test]
async fn test_org_admin_cannot_access_system_admin() {
    let f = TestFixture::new().await;

    // user2 is an ADMIN in org1 but not a SYSTEM-org admin.
    let res = f.ctx.get("/api/admin/users", Some(&f.user2_token)).await;

    res.assert_status(StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_system_admin_can_access_system_admin() {
    let f = TestFixture::new().await;

    // Create a SYSTEM-org admin and obtain a token scoped to the SYSTEM org.
    let sysadmin_id = f
        .ctx
        .create_user("Root", "root@test.com", "password123")
        .await;
    f.ctx
        .add_user_to_org(sysadmin_id, SYSTEM_ORG_ID, vec!["ADMIN".to_string()])
        .await;
    let sysadmin_token = f
        .ctx
        .login("root@test.com", "password123", Some(SYSTEM_ORG_ID))
        .await;

    let res = f.ctx.get("/api/admin/users", Some(&sysadmin_token)).await;

    res.assert_success();
}

#[tokio::test]
async fn test_unauthenticated_cannot_access_system_admin() {
    let f = TestFixture::new().await;

    let res = f.ctx.get("/api/admin/users", None).await;

    res.assert_status(StatusCode::UNAUTHORIZED);
}
