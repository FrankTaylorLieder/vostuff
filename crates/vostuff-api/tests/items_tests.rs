mod common;

use axum::http::StatusCode;
use common::TestFixture;
use serde_json::json;
use uuid::Uuid;

#[tokio::test]
async fn test_create_and_get_book_item() {
    let fixture = TestFixture::new().await;

    // Create a location first
    let loc_response = fixture.ctx.post(
        &format!("/api/organizations/{}/locations", fixture.org1_id),
        &json!({"name": "Library"}),
        Some(&fixture.user1_token)
    ).await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // Create a book item
    let create_response = fixture.ctx.post(
        &format!("/api/organizations/{}/items", fixture.org1_id),
        &json!({
            "item_type": "book",
            "name": "The Rust Programming Language",
            "description": "Official Rust book",
            "location_id": location_id
        }),
        Some(&fixture.user1_token)
    ).await;

    create_response.assert_success();
    let item_id: Uuid = create_response.body["id"].as_str().unwrap().parse().unwrap();

    // Get the item
    let get_response = fixture.ctx.get(
        &format!("/api/organizations/{}/items/{}", fixture.org1_id, item_id),
        Some(&fixture.user1_token)
    ).await;

    get_response.assert_success();
    assert_eq!(get_response.body["name"], "The Rust Programming Language");
    assert_eq!(get_response.body["item_type"], "book");
    assert_eq!(get_response.body["state"], "current");
}

#[tokio::test]
async fn test_create_vinyl_with_details() {
    let fixture = TestFixture::new().await;

    // Create location
    let loc_response = fixture.ctx.post(
        &format!("/api/organizations/{}/locations", fixture.org1_id),
        &json!({"name": "Record Room"}),
        Some(&fixture.user1_token)
    ).await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // Create vinyl item with details
    let create_response = fixture.ctx.post(
        &format!("/api/organizations/{}/items", fixture.org1_id),
        &json!({
            "item_type": "vinyl",
            "name": "Abbey Road",
            "description": "The Beatles - 1969",
            "location_id": location_id,
            "vinyl_details": {
                "size": "12_inch",
                "speed": "33",
                "channels": "stereo",
                "disks": 1,
                "media_grading": "near_mint",
                "sleeve_grading": "excellent"
            }
        }),
        Some(&fixture.user1_token)
    ).await;

    create_response.assert_success();
    let item_id: Uuid = create_response.body["id"].as_str().unwrap().parse().unwrap();

    // Get the item and verify vinyl details
    let get_response = fixture.ctx.get(
        &format!("/api/organizations/{}/items/{}", fixture.org1_id, item_id),
        Some(&fixture.user1_token)
    ).await;

    get_response.assert_success();
    assert_eq!(get_response.body["item_type"], "vinyl");
    assert_eq!(get_response.body["vinyl_details"]["size"], "12_inch");
    assert_eq!(get_response.body["vinyl_details"]["media_grading"], "near_mint");
}

#[tokio::test]
async fn test_update_item() {
    let fixture = TestFixture::new().await;

    // Create location
    let loc_response = fixture.ctx.post(
        &format!("/api/organizations/{}/locations", fixture.org1_id),
        &json!({"name": "Office"}),
        Some(&fixture.user1_token)
    ).await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // Create item
    let create_response = fixture.ctx.post(
        &format!("/api/organizations/{}/items", fixture.org1_id),
        &json!({
            "item_type": "book",
            "name": "Original Name",
            "description": "Original description",
            "location_id": location_id
        }),
        Some(&fixture.user1_token)
    ).await;
    create_response.assert_success();
    let item_id: Uuid = create_response.body["id"].as_str().unwrap().parse().unwrap();

    // Update the item
    let update_response = fixture.ctx.patch(
        &format!("/api/organizations/{}/items/{}", fixture.org1_id, item_id),
        &json!({
            "name": "Updated Name",
            "description": "Updated description"
        }),
        Some(&fixture.user1_token)
    ).await;

    update_response.assert_success();
    assert_eq!(update_response.body["name"], "Updated Name");
    assert_eq!(update_response.body["description"], "Updated description");
}

#[tokio::test]
async fn test_delete_item() {
    let fixture = TestFixture::new().await;

    // Create location
    let loc_response = fixture.ctx.post(
        &format!("/api/organizations/{}/locations", fixture.org1_id),
        &json!({"name": "Storage"}),
        Some(&fixture.user1_token)
    ).await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // Create item
    let create_response = fixture.ctx.post(
        &format!("/api/organizations/{}/items", fixture.org1_id),
        &json!({
            "item_type": "misc",
            "name": "Item to Delete",
            "description": "Will be deleted",
            "location_id": location_id
        }),
        Some(&fixture.user1_token)
    ).await;
    create_response.assert_success();
    let item_id: Uuid = create_response.body["id"].as_str().unwrap().parse().unwrap();

    // Delete the item
    let delete_response = fixture.ctx.delete(
        &format!("/api/organizations/{}/items/{}", fixture.org1_id, item_id),
        Some(&fixture.user1_token)
    ).await;

    delete_response.assert_status(StatusCode::NO_CONTENT);

    // Try to get the deleted item (should fail)
    let get_response = fixture.ctx.get(
        &format!("/api/organizations/{}/items/{}", fixture.org1_id, item_id),
        Some(&fixture.user1_token)
    ).await;

    assert_eq!(get_response.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_list_items() {
    let fixture = TestFixture::new().await;

    // Create location
    let loc_response = fixture.ctx.post(
        &format!("/api/organizations/{}/locations", fixture.org1_id),
        &json!({"name": "Main Storage"}),
        Some(&fixture.user1_token)
    ).await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // Create multiple items
    for i in 1..=3 {
        fixture.ctx.post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({
                "item_type": "book",
                "name": format!("Book {}", i),
                "description": format!("Description {}", i),
                "location_id": location_id
            }),
            Some(&fixture.user1_token)
        ).await.assert_success();
    }

    // List items
    let list_response = fixture.ctx.get(
        &format!("/api/organizations/{}/items", fixture.org1_id),
        Some(&fixture.user1_token)
    ).await;

    list_response.assert_success();

    let items = list_response.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 3, "Should have 3 items");
}

#[tokio::test]
async fn test_list_items_with_pagination() {
    let fixture = TestFixture::new().await;

    // Create location
    let loc_response = fixture.ctx.post(
        &format!("/api/organizations/{}/locations", fixture.org1_id),
        &json!({"name": "Archive"}),
        Some(&fixture.user1_token)
    ).await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // Create 15 items
    for i in 1..=15 {
        fixture.ctx.post(
            &format!("/api/organizations/{}/items", fixture.org1_id),
            &json!({
                "item_type": "book",
                "name": format!("Item {:02}", i),
                "description": "Test item",
                "location_id": location_id
            }),
            Some(&fixture.user1_token)
        ).await.assert_success();
    }

    // Get first page (limit 10)
    let page1 = fixture.ctx.get(
        &format!("/api/organizations/{}/items?limit=10&offset=0", fixture.org1_id),
        Some(&fixture.user1_token)
    ).await;
    page1.assert_success();
    assert_eq!(page1.body["items"].as_array().unwrap().len(), 10);
    assert_eq!(page1.body["total"].as_u64().unwrap(), 15);

    // Get second page
    let page2 = fixture.ctx.get(
        &format!("/api/organizations/{}/items?limit=10&offset=10", fixture.org1_id),
        Some(&fixture.user1_token)
    ).await;
    page2.assert_success();
    assert_eq!(page2.body["items"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn test_filter_items_by_type() {
    let fixture = TestFixture::new().await;

    // Create location
    let loc_response = fixture.ctx.post(
        &format!("/api/organizations/{}/locations", fixture.org1_id),
        &json!({"name": "Mixed Storage"}),
        Some(&fixture.user1_token)
    ).await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // Create different types of items
    fixture.ctx.post(
        &format!("/api/organizations/{}/items", fixture.org1_id),
        &json!({
            "item_type": "book",
            "name": "A Book",
            "location_id": location_id
        }),
        Some(&fixture.user1_token)
    ).await.assert_success();

    fixture.ctx.post(
        &format!("/api/organizations/{}/items", fixture.org1_id),
        &json!({
            "item_type": "cd",
            "name": "A CD",
            "location_id": location_id,
            "cd_details": {"disks": 1}
        }),
        Some(&fixture.user1_token)
    ).await.assert_success();

    // Filter by type=book
    let response = fixture.ctx.get(
        &format!("/api/organizations/{}/items?item_type=book", fixture.org1_id),
        Some(&fixture.user1_token)
    ).await;

    response.assert_success();
    let items = response.body["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["item_type"], "book");
}

#[tokio::test]
async fn test_create_item_without_authentication() {
    let fixture = TestFixture::new().await;

    // Create location first (authenticated)
    let loc_response = fixture.ctx.post(
        &format!("/api/organizations/{}/locations", fixture.org1_id),
        &json!({"name": "Test"}),
        Some(&fixture.user1_token)
    ).await;
    loc_response.assert_success();
    let location_id: Uuid = loc_response.body["id"].as_str().unwrap().parse().unwrap();

    // Try to create item without token
    let response = fixture.ctx.post(
        &format!("/api/organizations/{}/items", fixture.org1_id),
        &json!({
            "item_type": "book",
            "name": "Unauthorized",
            "location_id": location_id
        }),
        None // No token
    ).await;

    assert_eq!(response.status, StatusCode::UNAUTHORIZED);
}
