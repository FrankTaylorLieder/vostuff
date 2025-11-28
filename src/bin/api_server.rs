use axum::{
    routing::{delete, get, patch, post},
    Router,
};
use sqlx::PgPool;
use std::env;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use vostuff::api::{
    handlers::{collections, items, locations, organizations, tags, users},
    models::*,
    state::AppState,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        // Items
        items::list_items,
        items::get_item,
        items::create_item,
        items::update_item,
        items::delete_item,
        // Locations
        locations::list_locations,
        locations::create_location,
        locations::delete_location,
        // Collections
        collections::list_collections,
        collections::create_collection,
        collections::delete_collection,
        // Tags
        tags::list_tags,
        tags::create_tag,
        tags::delete_tag,
        // Admin - Organizations
        organizations::list_organizations,
        organizations::get_organization,
        organizations::create_organization,
        organizations::update_organization,
        organizations::delete_organization,
        organizations::list_organization_users,
        // Admin - Users
        users::list_users,
        users::get_user,
        users::create_user,
        users::update_user,
        users::delete_user,
        users::list_user_organizations,
        users::add_user_to_organization,
        users::remove_user_from_organization,
    ),
    components(
        schemas(
            Item, ItemType, ItemState,
            CreateItemRequest, UpdateItemRequest,
            VinylDetails, VinylSize, VinylSpeed, VinylChannels, Grading,
            Location, CreateLocationRequest,
            Collection, CreateCollectionRequest,
            Tag, CreateTagRequest,
            Organization, CreateOrganizationRequest, UpdateOrganizationRequest,
            User, CreateUserRequest, UpdateUserRequest,
            UserOrganization,
            ErrorResponse,
            PaginationParams, PaginatedResponse<Item>,
        )
    ),
    tags(
        (name = "items", description = "Item management endpoints"),
        (name = "locations", description = "Location management endpoints"),
        (name = "collections", description = "Collection management endpoints"),
        (name = "tags", description = "Tag management endpoints"),
        (name = "admin-organizations", description = "Admin endpoints for managing organizations"),
        (name = "admin-users", description = "Admin endpoints for managing users")
    ),
    info(
        title = "VOStuff API",
        version = "0.1.0",
        description = "REST API for VOStuff - a multi-tenant stuff tracking application",
        contact(
            name = "VOStuff",
        )
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api_server=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Get database URL
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev".to_string());

    tracing::info!("Connecting to database: {}", database_url);
    let pool = PgPool::connect(&database_url).await?;

    // Create app state
    let state = AppState::new(pool);

    // Build API router
    let api_router = Router::new()
        // Items
        .route("/organizations/:org_id/items", get(items::list_items))
        .route("/organizations/:org_id/items", post(items::create_item))
        .route("/organizations/:org_id/items/:item_id", get(items::get_item))
        .route("/organizations/:org_id/items/:item_id", patch(items::update_item))
        .route("/organizations/:org_id/items/:item_id", delete(items::delete_item))
        // Locations
        .route("/organizations/:org_id/locations", get(locations::list_locations))
        .route("/organizations/:org_id/locations", post(locations::create_location))
        .route("/organizations/:org_id/locations/:location_id", delete(locations::delete_location))
        // Collections
        .route("/organizations/:org_id/collections", get(collections::list_collections))
        .route("/organizations/:org_id/collections", post(collections::create_collection))
        .route("/organizations/:org_id/collections/:collection_id", delete(collections::delete_collection))
        // Tags
        .route("/organizations/:org_id/tags", get(tags::list_tags))
        .route("/organizations/:org_id/tags", post(tags::create_tag))
        .route("/organizations/:org_id/tags/:tag_name", delete(tags::delete_tag))
        // Admin - Organizations
        .route("/admin/organizations", get(organizations::list_organizations))
        .route("/admin/organizations", post(organizations::create_organization))
        .route("/admin/organizations/:org_id", get(organizations::get_organization))
        .route("/admin/organizations/:org_id", patch(organizations::update_organization))
        .route("/admin/organizations/:org_id", delete(organizations::delete_organization))
        .route("/admin/organizations/:org_id/users", get(organizations::list_organization_users))
        // Admin - Users
        .route("/admin/users", get(users::list_users))
        .route("/admin/users", post(users::create_user))
        .route("/admin/users/:user_id", get(users::get_user))
        .route("/admin/users/:user_id", patch(users::update_user))
        .route("/admin/users/:user_id", delete(users::delete_user))
        // Admin - User Organizations
        .route("/admin/users/:user_id/organizations", get(users::list_user_organizations))
        .route("/admin/users/:user_id/organizations/:org_id", post(users::add_user_to_organization))
        .route("/admin/users/:user_id/organizations/:org_id", delete(users::remove_user_from_organization))
        .with_state(state);

    // Build main app with Swagger UI
    let app = Router::new()
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        .nest("/api", api_router)
        .layer(TraceLayer::new_for_http());

    // Start server
    let addr = "0.0.0.0:8080";
    tracing::info!("Starting server on {}", addr);
    tracing::info!("Swagger UI available at http://localhost:8080/swagger-ui");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}