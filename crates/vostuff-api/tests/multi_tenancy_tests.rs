mod common;

use axum::http::StatusCode;
use common::TestFixture;
use serde_json::json;
use uuid::Uuid;

const BOOK_KIND_ID: &str = "00000000-0000-0000-0000-000000000004";

#[tokio::test]
async fn test_user_cannot_access_other_org_items() {
    let fixture = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();

    // User from org1 creates an item
    fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({
                "kind_id": book_id,
                "name": "Secret Org1 Item",
                "description": "This belongs to org1"
            }),
            Some(&fixture.user1_token),
        )
        .await
        .assert_success();

    // User from org2 tries to list items from org1
    let list_response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            Some(&fixture.user3_token),
        )
        .await;

    assert!(
        list_response.status == StatusCode::FORBIDDEN
            || list_response.status == StatusCode::UNAUTHORIZED,
        "User from org2 should not access org1 items. Status: {:?}",
        list_response.status
    );
}

#[tokio::test]
async fn test_user_cannot_create_item_in_other_org() {
    let fixture = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();

    // User from org1 tries to create item in org2
    let response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org2_id),
            &json!({
                "kind_id": book_id,
                "name": "Unauthorized Item"
            }),
            Some(&fixture.user1_token),
        )
        .await;

    assert!(
        response.status == StatusCode::FORBIDDEN || response.status == StatusCode::UNAUTHORIZED,
        "User from org1 should not create items in org2. Status: {:?}",
        response.status
    );
}

#[tokio::test]
async fn test_user_cannot_access_other_org_locations() {
    let fixture = TestFixture::new().await;

    fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/locations", fixture.org1_id),
            &json!({"name": "Private Location"}),
            Some(&fixture.user1_token),
        )
        .await
        .assert_success();

    let list_response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/locations", fixture.org1_id),
            Some(&fixture.user3_token),
        )
        .await;

    assert!(
        list_response.status == StatusCode::FORBIDDEN
            || list_response.status == StatusCode::UNAUTHORIZED,
        "User should not access other org's locations. Status: {:?}",
        list_response.status
    );
}

#[tokio::test]
async fn test_user_cannot_access_other_org_collections() {
    let fixture = TestFixture::new().await;

    fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/collections", fixture.org1_id),
            &json!({"name": "Private Collection", "description": "Org1 only"}),
            Some(&fixture.user1_token),
        )
        .await
        .assert_success();

    let list_response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/collections", fixture.org1_id),
            Some(&fixture.user3_token),
        )
        .await;

    assert!(
        list_response.status == StatusCode::FORBIDDEN
            || list_response.status == StatusCode::UNAUTHORIZED,
        "User should not access other org's collections. Status: {:?}",
        list_response.status
    );
}

#[tokio::test]
async fn test_user_cannot_access_other_org_tags() {
    let fixture = TestFixture::new().await;

    fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/tags", fixture.org1_id),
            &json!({"name": "secret-tag"}),
            Some(&fixture.user1_token),
        )
        .await
        .assert_success();

    let list_response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/tags", fixture.org1_id),
            Some(&fixture.user3_token),
        )
        .await;

    assert!(
        list_response.status == StatusCode::FORBIDDEN
            || list_response.status == StatusCode::UNAUTHORIZED,
        "User should not access other org's tags. Status: {:?}",
        list_response.status
    );
}

#[tokio::test]
async fn test_users_in_same_org_can_share_data() {
    let fixture = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();

    // User1 from org1 creates an item
    fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({
                "kind_id": book_id,
                "name": "Shared Item",
                "description": "Shared within org1"
            }),
            Some(&fixture.user1_token),
        )
        .await
        .assert_success();

    // User2 (also from org1) should be able to see it
    let list_response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            Some(&fixture.user2_token),
        )
        .await;

    list_response.assert_success();
    let items = list_response.body["items"].as_array().unwrap();
    assert!(
        !items.is_empty(),
        "User2 should see items created by User1 in the same org"
    );
}

#[tokio::test]
async fn test_admin_cannot_access_other_org_data() {
    let fixture = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();

    // User1 creates item in org1
    fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({"kind_id": book_id, "name": "Org1 Item"}),
            Some(&fixture.user1_token),
        )
        .await
        .assert_success();

    // User2 is ADMIN in org1, but not in org2 — should be rejected
    let response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/items", fixture.org2_id),
            Some(&fixture.user2_token),
        )
        .await;

    assert!(
        response.status == StatusCode::FORBIDDEN || response.status == StatusCode::UNAUTHORIZED,
        "Admin from org1 should not access org2 data. Status: {:?}",
        response.status
    );
}
