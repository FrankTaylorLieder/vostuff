mod common;

use axum::http::StatusCode;
use common::TestFixture;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_user_cannot_access_other_org_items() {
    let fixture = TestFixture::new().await;

    // Create location in org1
    let loc_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/locations", fixture.org1_id),
            &json!({"name": "Test Location"}),
            Some(&fixture.user1_token),
        )
        .await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // User from org1 creates an item
    let create_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({
                "item_type": "book",
                "name": "Secret Org1 Item",
                "description": "This belongs to org1",
                "location_id": location_id
            }),
            Some(&fixture.user1_token),
        )
        .await;

    create_response.assert_success();

    // User from org2 tries to list items from org1
    let list_response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            Some(&fixture.user3_token), // user3 is in org2
        )
        .await;

    // Should fail with forbidden or return empty list
    // (depending on implementation - currently returns 403)
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

    // Create location in org2
    let loc_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/locations", fixture.org2_id),
            &json!({"name": "Test Location"}),
            Some(&fixture.user3_token),
        )
        .await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // User from org1 tries to create item in org2
    let response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org2_id),
            &json!({
                "item_type": "book",
                "name": "Unauthorized Item",
                "description": "This should fail",
                "location_id": location_id
            }),
            Some(&fixture.user1_token), // user1 is in org1, not org2
        )
        .await;

    // Should be forbidden
    assert!(
        response.status == StatusCode::FORBIDDEN || response.status == StatusCode::UNAUTHORIZED,
        "User from org1 should not create items in org2. Status: {:?}",
        response.status
    );
}

#[tokio::test]
async fn test_user_cannot_access_other_org_locations() {
    let fixture = TestFixture::new().await;

    // User from org1 creates a location
    let create_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/locations", fixture.org1_id),
            &json!({"name": "Private Location"}),
            Some(&fixture.user1_token),
        )
        .await;
    create_response.assert_success();

    // User from org2 tries to list org1's locations
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

    // User from org1 creates a collection
    let create_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/collections", fixture.org1_id),
            &json!({
                "name": "Private Collection",
                "description": "Org1 only"
            }),
            Some(&fixture.user1_token),
        )
        .await;
    create_response.assert_success();

    // User from org2 tries to list org1's collections
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

    // User from org1 creates a tag
    let create_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/tags", fixture.org1_id),
            &json!({"name": "secret-tag"}),
            Some(&fixture.user1_token),
        )
        .await;
    create_response.assert_success();

    // User from org2 tries to list org1's tags
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

    // Create location in org1
    let loc_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/locations", fixture.org1_id),
            &json!({"name": "Shared Location"}),
            Some(&fixture.user1_token),
        )
        .await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // User1 from org1 creates an item
    let create_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({
                "item_type": "book",
                "name": "Shared Item",
                "description": "Shared within org1",
                "location_id": location_id
            }),
            Some(&fixture.user1_token),
        )
        .await;
    create_response.assert_success();

    // User2 (also from org1) should be able to see it
    let list_response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            Some(&fixture.user2_token), // user2 is also in org1
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

    // Create location in org1
    let loc_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/locations", fixture.org1_id),
            &json!({"name": "Test Location"}),
            Some(&fixture.user1_token),
        )
        .await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // User1 creates item in org1
    fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({
                "item_type": "book",
                "name": "Org1 Item",
                "description": "Belongs to org1",
                "location_id": location_id
            }),
            Some(&fixture.user1_token),
        )
        .await;

    // User2 is ADMIN in org1, but not in org2
    // User2 tries to access org2's items (should fail)
    let response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/items", fixture.org2_id),
            Some(&fixture.user2_token), // user2 is ADMIN but only in org1
        )
        .await;

    assert!(
        response.status == StatusCode::FORBIDDEN || response.status == StatusCode::UNAUTHORIZED,
        "Admin from org1 should not access org2 data. Status: {:?}",
        response.status
    );
}
