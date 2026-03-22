mod common;

use axum::http::StatusCode;
use common::TestFixture;
use serde_json::json;
use uuid::Uuid;

// Fixed UUIDs from seed migration
const BOOK_KIND_ID: &str = "00000000-0000-0000-0000-000000000004";
const VINYL_KIND_ID: &str = "00000000-0000-0000-0000-000000000001";
const CD_KIND_ID: &str = "00000000-0000-0000-0000-000000000002";
const MISC_KIND_ID: &str = "00000000-0000-0000-0000-000000000007";

#[tokio::test]
async fn test_create_and_get_book_item() {
    let fixture = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();

    let create_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({
                "kind_id": book_id,
                "name": "The Rust Programming Language",
                "description": "Official Rust book"
            }),
            Some(&fixture.user1_token),
        )
        .await;

    create_response.assert_success();
    let item_id: Uuid = create_response.body["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let get_response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/items/{}", fixture.org1_id, item_id),
            Some(&fixture.user1_token),
        )
        .await;

    get_response.assert_success();
    assert_eq!(get_response.body["name"], "The Rust Programming Language");
    assert_eq!(get_response.body["kind_name"], "book");
    assert_eq!(get_response.body["state"], "current");
}

#[tokio::test]
async fn test_create_vinyl_with_soft_fields() {
    let fixture = TestFixture::new().await;
    let vinyl_id = Uuid::parse_str(VINYL_KIND_ID).unwrap();

    let create_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({
                "kind_id": vinyl_id,
                "name": "Abbey Road",
                "description": "The Beatles - 1969",
                "soft_fields": {
                    "size": "12_inch",
                    "speed": "33",
                    "channels": "stereo",
                    "disks": 1,
                    "media_grading": "near_mint",
                    "sleeve_grading": "excellent"
                }
            }),
            Some(&fixture.user1_token),
        )
        .await;

    create_response.assert_success();
    let item_id: Uuid = create_response.body["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let get_response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/items/{}", fixture.org1_id, item_id),
            Some(&fixture.user1_token),
        )
        .await;

    get_response.assert_success();
    assert_eq!(get_response.body["kind_name"], "vinyl");
    assert_eq!(get_response.body["soft_fields"]["size"], "12_inch");
    assert_eq!(get_response.body["soft_fields"]["media_grading"], "near_mint");
}

#[tokio::test]
async fn test_update_item() {
    let fixture = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();

    let create_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({
                "kind_id": book_id,
                "name": "Original Name",
                "description": "Original description"
            }),
            Some(&fixture.user1_token),
        )
        .await;
    create_response.assert_success();
    let item_id: Uuid = create_response.body["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let update_response = fixture
        .ctx
        .patch(
            &format!("/api/organizations/{}/items/{}", fixture.org1_id, item_id),
            &json!({
                "name": "Updated Name",
                "description": "Updated description"
            }),
            Some(&fixture.user1_token),
        )
        .await;

    update_response.assert_success();
    assert_eq!(update_response.body["name"], "Updated Name");
    assert_eq!(update_response.body["description"], "Updated description");
}

#[tokio::test]
async fn test_delete_item() {
    let fixture = TestFixture::new().await;
    let misc_id = Uuid::parse_str(MISC_KIND_ID).unwrap();

    let create_response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({
                "kind_id": misc_id,
                "name": "Item to Delete"
            }),
            Some(&fixture.user1_token),
        )
        .await;
    create_response.assert_success();
    let item_id: Uuid = create_response.body["id"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap();

    let delete_response = fixture
        .ctx
        .delete(
            &format!("/api/organizations/{}/items/{}", fixture.org1_id, item_id),
            Some(&fixture.user1_token),
        )
        .await;
    delete_response.assert_status(StatusCode::NO_CONTENT);

    let get_response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/items/{}", fixture.org1_id, item_id),
            Some(&fixture.user1_token),
        )
        .await;
    assert_eq!(get_response.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_items() {
    let fixture = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();

    for i in 1..=3 {
        fixture
            .ctx
            .post(
                &format!("/api/organizations/{}/items", fixture.org1_id),
                &json!({
                    "kind_id": book_id,
                    "name": format!("Book {}", i)
                }),
                Some(&fixture.user1_token),
            )
            .await
            .assert_success();
    }

    let list_response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            Some(&fixture.user1_token),
        )
        .await;

    list_response.assert_success();
    let items = list_response.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "Should have 3 items");
}

#[tokio::test]
async fn test_list_items_with_pagination() {
    let fixture = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();

    for i in 1..=15 {
        fixture
            .ctx
            .post(
                &format!("/api/organizations/{}/items", fixture.org1_id),
                &json!({
                    "kind_id": book_id,
                    "name": format!("Item {:02}", i)
                }),
                Some(&fixture.user1_token),
            )
            .await
            .assert_success();
    }

    let page1 = fixture
        .ctx
        .get(
            &format!(
                "/api/organizations/{}/items?per_page=10&page=1",
                fixture.org1_id
            ),
            Some(&fixture.user1_token),
        )
        .await;
    page1.assert_success();
    assert_eq!(page1.body["items"].as_array().unwrap().len(), 10);
    assert_eq!(page1.body["total"].as_i64().unwrap(), 15);

    let page2 = fixture
        .ctx
        .get(
            &format!(
                "/api/organizations/{}/items?per_page=10&page=2",
                fixture.org1_id
            ),
            Some(&fixture.user1_token),
        )
        .await;
    page2.assert_success();
    assert_eq!(page2.body["items"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn test_filter_items_by_kind() {
    let fixture = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();
    let cd_id = Uuid::parse_str(CD_KIND_ID).unwrap();

    fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({"kind_id": book_id, "name": "A Book"}),
            Some(&fixture.user1_token),
        )
        .await
        .assert_success();

    fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({"kind_id": cd_id, "name": "A CD"}),
            Some(&fixture.user1_token),
        )
        .await
        .assert_success();

    let response = fixture
        .ctx
        .get(
            &format!("/api/organizations/{}/items?kind=book", fixture.org1_id),
            Some(&fixture.user1_token),
        )
        .await;

    response.assert_success();
    let items = response.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["kind_name"], "book");
}

#[tokio::test]
async fn test_create_item_without_authentication() {
    let fixture = TestFixture::new().await;
    let book_id = Uuid::parse_str(BOOK_KIND_ID).unwrap();

    let response = fixture
        .ctx
        .post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({"kind_id": book_id, "name": "Unauthorized"}),
            None,
        )
        .await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}
