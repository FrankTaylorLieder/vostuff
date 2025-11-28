mod common;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    Router,
};
use serde_json::json;
use sqlx::PgPool;
use std::env;
use tower::ServiceExt;
use uuid::Uuid;
use vostuff::api::{models::*, state::AppState};
use vostuff::schema::SchemaManager;

use common::SampleDataLoader;

// Helper function to create test router
fn create_test_router(pool: PgPool) -> Router {
    use axum::routing::{delete, get, patch, post};

    let state = AppState::new(pool, "test_jwt_secret".to_string());

    Router::new()
        // Items
        .route("/api/organizations/:org_id/items", get(vostuff::api::handlers::items::list_items))
        .route("/api/organizations/:org_id/items", post(vostuff::api::handlers::items::create_item))
        .route("/api/organizations/:org_id/items/:item_id", get(vostuff::api::handlers::items::get_item))
        .route("/api/organizations/:org_id/items/:item_id", patch(vostuff::api::handlers::items::update_item))
        .route("/api/organizations/:org_id/items/:item_id", delete(vostuff::api::handlers::items::delete_item))
        // Locations
        .route("/api/organizations/:org_id/locations", get(vostuff::api::handlers::locations::list_locations))
        .route("/api/organizations/:org_id/locations", post(vostuff::api::handlers::locations::create_location))
        .route("/api/organizations/:org_id/locations/:location_id", delete(vostuff::api::handlers::locations::delete_location))
        // Collections
        .route("/api/organizations/:org_id/collections", get(vostuff::api::handlers::collections::list_collections))
        .route("/api/organizations/:org_id/collections", post(vostuff::api::handlers::collections::create_collection))
        .route("/api/organizations/:org_id/collections/:collection_id", delete(vostuff::api::handlers::collections::delete_collection))
        // Tags
        .route("/api/organizations/:org_id/tags", get(vostuff::api::handlers::tags::list_tags))
        .route("/api/organizations/:org_id/tags", post(vostuff::api::handlers::tags::create_tag))
        .route("/api/organizations/:org_id/tags/:tag_name", delete(vostuff::api::handlers::tags::delete_tag))
        // Admin - Organizations
        .route("/api/admin/organizations", get(vostuff::api::handlers::organizations::list_organizations))
        .route("/api/admin/organizations", post(vostuff::api::handlers::organizations::create_organization))
        .route("/api/admin/organizations/:org_id", get(vostuff::api::handlers::organizations::get_organization))
        .route("/api/admin/organizations/:org_id", patch(vostuff::api::handlers::organizations::update_organization))
        .route("/api/admin/organizations/:org_id", delete(vostuff::api::handlers::organizations::delete_organization))
        .route("/api/admin/organizations/:org_id/users", get(vostuff::api::handlers::organizations::list_organization_users))
        // Admin - Users
        .route("/api/admin/users", get(vostuff::api::handlers::users::list_users))
        .route("/api/admin/users", post(vostuff::api::handlers::users::create_user))
        .route("/api/admin/users/:user_id", get(vostuff::api::handlers::users::get_user))
        .route("/api/admin/users/:user_id", patch(vostuff::api::handlers::users::update_user))
        .route("/api/admin/users/:user_id", delete(vostuff::api::handlers::users::delete_user))
        // Admin - User Organizations
        .route("/api/admin/users/:user_id/organizations", get(vostuff::api::handlers::users::list_user_organizations))
        .route("/api/admin/users/:user_id/organizations/:org_id", post(vostuff::api::handlers::users::add_user_to_organization))
        .route("/api/admin/users/:user_id/organizations/:org_id", delete(vostuff::api::handlers::users::remove_user_from_organization))
        // Authentication
        .route("/api/auth/login", post(vostuff::api::handlers::auth::login))
        .with_state(state)
}

// Helper function to set up test database
async fn setup_test_db() -> (PgPool, Uuid, Uuid) {
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev".to_string());

    let manager = SchemaManager::new(&database_url).await.unwrap();
    manager.reset_database().await.unwrap();
    manager.run_migrations().await.unwrap();

    let pool = manager.get_pool().clone();

    // Load sample data
    let loader = SampleDataLoader::new(&pool);
    let result = loader.load_sample_data().await.unwrap();

    (pool, result.coke_org_id, result.pepsi_org_id)
}

#[tokio::test]
async fn test_list_items() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/organizations/{}/items?page=1&per_page=10", coke_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: PaginatedResponse<Item> = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_data.items.len(), 10);
    assert_eq!(response_data.total, 50); // 50 items per org
    assert_eq!(response_data.page, 1);
    assert_eq!(response_data.per_page, 10);
    assert_eq!(response_data.total_pages, 5);

    pool.close().await;
}

#[tokio::test]
async fn test_list_items_pagination() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // Test page 2
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/organizations/{}/items?page=2&per_page=10", coke_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let response_data: PaginatedResponse<Item> = serde_json::from_slice(&body).unwrap();

    assert_eq!(response_data.items.len(), 10);
    assert_eq!(response_data.page, 2);

    pool.close().await;
}

#[tokio::test]
async fn test_get_item() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // First, get list of items to get an item ID
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/organizations/{}/items?page=1&per_page=1", coke_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
    let list_data: PaginatedResponse<Item> = serde_json::from_slice(&body).unwrap();
    let item_id = list_data.items[0].id;

    // Now get the specific item
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/organizations/{}/items/{}", coke_org_id, item_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let item: Item = serde_json::from_slice(&body).unwrap();

    assert_eq!(item.id, item_id);
    assert_eq!(item.organization_id, coke_org_id);

    pool.close().await;
}

#[tokio::test]
async fn test_get_item_not_found() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let fake_item_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/organizations/{}/items/{}", coke_org_id, fake_item_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(error.error, "not_found");

    pool.close().await;
}

#[tokio::test]
async fn test_create_item() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let new_item = json!({
        "item_type": "vinyl",
        "name": "Test Album - Integration Test",
        "description": "Created via integration test",
        "notes": "Test notes",
        "date_acquired": "2025-11-23"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/organizations/{}/items", coke_org_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_item).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let item: Item = serde_json::from_slice(&body).unwrap();

    assert_eq!(item.name, "Test Album - Integration Test");
    assert_eq!(item.organization_id, coke_org_id);

    pool.close().await;
}

#[tokio::test]
async fn test_update_item() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // First, get an item to update
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/organizations/{}/items?page=1&per_page=1", coke_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
    let list_data: PaginatedResponse<Item> = serde_json::from_slice(&body).unwrap();
    let item_id = list_data.items[0].id;

    // Update the item
    let update_data = json!({
        "name": "Updated Name",
        "description": "Updated description"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/organizations/{}/items/{}", coke_org_id, item_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&update_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let item: Item = serde_json::from_slice(&body).unwrap();

    assert_eq!(item.name, "Updated Name");
    assert_eq!(item.description, Some("Updated description".to_string()));

    pool.close().await;
}

#[tokio::test]
async fn test_delete_item() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // First, create an item to delete
    let new_item = json!({
        "item_type": "misc",
        "name": "Item to Delete",
        "description": "This will be deleted"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/organizations/{}/items", coke_org_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_item).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let created_item: Item = serde_json::from_slice(&body).unwrap();

    // Now delete it
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/organizations/{}/items/{}", coke_org_id, created_item.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    pool.close().await;
}

#[tokio::test]
async fn test_multi_tenant_isolation() {
    let (pool, coke_org_id, pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // Get an item from Coke org
    let coke_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/organizations/{}/items?page=1&per_page=1", coke_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(coke_response.into_body(), usize::MAX).await.unwrap();
    let coke_data: PaginatedResponse<Item> = serde_json::from_slice(&body).unwrap();
    let coke_item_id = coke_data.items[0].id;

    // Try to access Coke's item using Pepsi's org ID - should return 404
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/organizations/{}/items/{}", pepsi_org_id, coke_item_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    pool.close().await;
}

#[tokio::test]
async fn test_list_locations() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/organizations/{}/locations", coke_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let locations: Vec<Location> = serde_json::from_slice(&body).unwrap();

    assert_eq!(locations.len(), 4); // 4 locations per org

    pool.close().await;
}

#[tokio::test]
async fn test_create_location() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let new_location = json!({
        "name": "Garage"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/organizations/{}/locations", coke_org_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_location).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let location: Location = serde_json::from_slice(&body).unwrap();

    assert_eq!(location.name, "Garage");
    assert_eq!(location.organization_id, coke_org_id);

    pool.close().await;
}

#[tokio::test]
async fn test_delete_location() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // Create a location to delete
    let new_location = json!({
        "name": "Temporary Location"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/organizations/{}/locations", coke_org_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_location).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let location: Location = serde_json::from_slice(&body).unwrap();

    // Delete it
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/organizations/{}/locations/{}", coke_org_id, location.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    pool.close().await;
}

#[tokio::test]
async fn test_list_collections() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/organizations/{}/collections", coke_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let collections: Vec<Collection> = serde_json::from_slice(&body).unwrap();

    assert_eq!(collections.len(), 4); // 4 collections per org

    pool.close().await;
}

#[tokio::test]
async fn test_create_collection() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let new_collection = json!({
        "name": "Test Collection",
        "description": "A test collection",
        "notes": "Test notes"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/organizations/{}/collections", coke_org_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_collection).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let collection: Collection = serde_json::from_slice(&body).unwrap();

    assert_eq!(collection.name, "Test Collection");
    assert_eq!(collection.organization_id, coke_org_id);

    pool.close().await;
}

#[tokio::test]
async fn test_list_tags() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/organizations/{}/tags", coke_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let tags: Vec<Tag> = serde_json::from_slice(&body).unwrap();

    assert_eq!(tags.len(), 6); // 6 tags per org

    pool.close().await;
}

#[tokio::test]
async fn test_create_tag() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let new_tag = json!({
        "name": "test-tag"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/organizations/{}/tags", coke_org_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_tag).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let tag: Tag = serde_json::from_slice(&body).unwrap();

    assert_eq!(tag.name, "test-tag");
    assert_eq!(tag.organization_id, coke_org_id);

    pool.close().await;
}

#[tokio::test]
async fn test_delete_tag() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // Create a tag to delete
    let new_tag = json!({
        "name": "temporary-tag"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/organizations/{}/tags", coke_org_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_tag).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let tag: Tag = serde_json::from_slice(&body).unwrap();

    // Delete it
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/organizations/{}/tags/{}", coke_org_id, tag.name))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    pool.close().await;
}

// ==================== Admin Organization Tests ====================

#[tokio::test]
async fn test_admin_list_organizations() {
    let (pool, _coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/organizations")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let organizations: Vec<Organization> = serde_json::from_slice(&body).unwrap();

    // Should include at least Coke, Pepsi, and SYSTEM organizations
    assert!(organizations.len() >= 3);
    assert!(organizations.iter().any(|o| o.name == "Coke"));
    assert!(organizations.iter().any(|o| o.name == "Pepsi"));

    pool.close().await;
}

#[tokio::test]
async fn test_admin_create_organization() {
    let (pool, _coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let new_org = json!({
        "name": "Test Company",
        "description": "A test organization"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/organizations")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_org).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let organization: Organization = serde_json::from_slice(&body).unwrap();

    assert_eq!(organization.name, "Test Company");
    assert_eq!(organization.description, Some("A test organization".to_string()));

    pool.close().await;
}

#[tokio::test]
async fn test_admin_get_organization() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/organizations/{}", coke_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let organization: Organization = serde_json::from_slice(&body).unwrap();

    assert_eq!(organization.id, coke_org_id);
    assert_eq!(organization.name, "Coke");

    pool.close().await;
}

#[tokio::test]
async fn test_admin_update_organization() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let update_data = json!({
        "name": "Coca-Cola International",
        "description": "Updated description"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/admin/organizations/{}", coke_org_id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&update_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let organization: Organization = serde_json::from_slice(&body).unwrap();

    assert_eq!(organization.name, "Coca-Cola International");
    assert_eq!(organization.description, Some("Updated description".to_string()));

    pool.close().await;
}

#[tokio::test]
async fn test_admin_delete_organization() {
    let (pool, _coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // First create an organization to delete
    let new_org = json!({
        "name": "Temp Organization"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/organizations")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_org).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(create_response.into_body(), usize::MAX).await.unwrap();
    let organization: Organization = serde_json::from_slice(&body).unwrap();

    // Now delete it
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/admin/organizations/{}", organization.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    pool.close().await;
}

// ==================== Admin User Tests ====================

#[tokio::test]
async fn test_admin_list_users() {
    let (pool, _coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/admin/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let users: Vec<User> = serde_json::from_slice(&body).unwrap();

    // Should include at least Bob and Alice
    assert!(users.len() >= 2);
    assert!(users.iter().any(|u| u.name == "Bob"));
    assert!(users.iter().any(|u| u.name == "Alice"));

    pool.close().await;
}

#[tokio::test]
async fn test_admin_create_user() {
    let (pool, _coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let new_user = json!({
        "name": "Charlie",
        "identity": "charlie@example.com"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/users")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_user).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let user: User = serde_json::from_slice(&body).unwrap();

    assert_eq!(user.name, "Charlie");
    assert_eq!(user.identity, "charlie@example.com");

    pool.close().await;
}

#[tokio::test]
async fn test_admin_update_user() {
    let (pool, _coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // First get Bob's ID
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
    let users: Vec<User> = serde_json::from_slice(&body).unwrap();
    let bob = users.iter().find(|u| u.name == "Bob").unwrap();

    // Update Bob
    let update_data = json!({
        "name": "Robert"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/admin/users/{}", bob.id))
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&update_data).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let user: User = serde_json::from_slice(&body).unwrap();

    assert_eq!(user.name, "Robert");

    pool.close().await;
}

#[tokio::test]
async fn test_admin_list_user_organizations() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // First get Bob's ID
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
    let users: Vec<User> = serde_json::from_slice(&body).unwrap();
    let bob = users.iter().find(|u| u.name == "Bob").unwrap();

    // Get Bob's organizations
    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/users/{}/organizations", bob.id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let organizations: Vec<Organization> = serde_json::from_slice(&body).unwrap();

    assert_eq!(organizations.len(), 1);
    assert_eq!(organizations[0].id, coke_org_id);

    pool.close().await;
}

#[tokio::test]
async fn test_admin_add_user_to_organization() {
    let (pool, _coke_org_id, pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // Get Bob's ID
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
    let users: Vec<User> = serde_json::from_slice(&body).unwrap();
    let bob = users.iter().find(|u| u.name == "Bob").unwrap();

    // Add Bob to Pepsi organization
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/admin/users/{}/organizations/{}", bob.id, pepsi_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let user_org: UserOrganization = serde_json::from_slice(&body).unwrap();

    assert_eq!(user_org.user_id, bob.id);
    assert_eq!(user_org.organization_id, pepsi_org_id);

    pool.close().await;
}

#[tokio::test]
async fn test_admin_remove_user_from_organization() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // Get Bob's ID
    let list_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/admin/users")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    let body = axum::body::to_bytes(list_response.into_body(), usize::MAX).await.unwrap();
    let users: Vec<User> = serde_json::from_slice(&body).unwrap();
    let bob = users.iter().find(|u| u.name == "Bob").unwrap();

    // Remove Bob from Coke organization
    let response = app
        .oneshot(
            Request::builder()
                .method("DELETE")
                .uri(format!("/api/admin/users/{}/organizations/{}", bob.id, coke_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    pool.close().await;
}

#[tokio::test]
async fn test_admin_list_organization_users() {
    let (pool, coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/organizations/{}/users", coke_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let users: Vec<User> = serde_json::from_slice(&body).unwrap();

    // Coke org should have Bob
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].name, "Bob");

    pool.close().await;
}

#[tokio::test]
async fn test_admin_list_organization_users_not_found() {
    let (pool, _coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let fake_org_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/admin/organizations/{}/users", fake_org_id))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(error.error, "not_found");
    assert_eq!(error.message, "Organization not found");

    pool.close().await;
}

// ==================== Authentication Tests ====================

#[tokio::test]
async fn test_auth_login_success() {
    let (pool, _coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // First create a user with a password
    let new_user = serde_json::json!({
        "name": "Test User",
        "identity": "testuser@example.com",
        "password": "testpassword123"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/users")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_user).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);

    // Now try to login
    let login_request = serde_json::json!({
        "identity": "testuser@example.com",
        "password": "testpassword123"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let login_response: serde_json::Value = serde_json::from_slice(&body).unwrap();

    // Check that we got a token and user info
    assert!(login_response["token"].is_string());
    assert_eq!(login_response["expires_in"], 86400); // 24 hours
    assert_eq!(login_response["user"]["identity"], "testuser@example.com");
    assert_eq!(login_response["user"]["name"], "Test User");

    pool.close().await;
}

#[tokio::test]
async fn test_auth_login_invalid_credentials() {
    let (pool, _coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    let login_request = serde_json::json!({
        "identity": "nonexistent@example.com",
        "password": "wrongpassword"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: ErrorResponse = serde_json::from_slice(&body).unwrap();

    assert_eq!(error.error, "unauthorized");
    assert_eq!(error.message, "Invalid credentials");

    pool.close().await;
}

#[tokio::test]
async fn test_auth_login_user_without_password() {
    let (pool, _coke_org_id, _pepsi_org_id) = setup_test_db().await;
    let app = create_test_router(pool.clone());

    // Create a user without a password
    let new_user = serde_json::json!({
        "name": "No Password User",
        "identity": "nopassword@example.com"
    });

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/admin/users")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&new_user).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);

    // Try to login - should fail with same "Invalid credentials" message
    let login_request = serde_json::json!({
        "identity": "nopassword@example.com",
        "password": "anypassword"
    });

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_vec(&login_request).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

    let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
    let error: ErrorResponse = serde_json::from_slice(&body).unwrap();

    // Should get same generic error message to prevent user enumeration
    assert_eq!(error.error, "unauthorized");
    assert_eq!(error.message, "Invalid credentials");

    pool.close().await;
}
