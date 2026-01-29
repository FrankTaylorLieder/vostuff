pub mod auth;
pub mod collections;
pub mod items;
pub mod locations;
pub mod organizations;
pub mod tags;
pub mod users;

use crate::api::{middleware::auth_middleware, state::AppState};
use axum::{
    Router, middleware,
    routing::{delete, get, patch, post},
};

/// Build the API router with all routes configured
/// This is used by both the main application and integration tests
/// Note: Routes don't include /api prefix - that's added by nesting in main app
pub fn build_router(state: AppState) -> Router {
    Router::new()
        // Items
        .route("/organizations/:org_id/items", get(items::list_items))
        .route("/organizations/:org_id/items", post(items::create_item))
        .route(
            "/organizations/:org_id/items/:item_id",
            get(items::get_item),
        )
        .route(
            "/organizations/:org_id/items/:item_id/details",
            get(items::get_item_details),
        )
        .route(
            "/organizations/:org_id/items/:item_id",
            patch(items::update_item),
        )
        .route(
            "/organizations/:org_id/items/:item_id",
            delete(items::delete_item),
        )
        // Locations
        .route(
            "/organizations/:org_id/locations",
            get(locations::list_locations),
        )
        .route(
            "/organizations/:org_id/locations",
            post(locations::create_location),
        )
        .route(
            "/organizations/:org_id/locations/:location_id",
            delete(locations::delete_location),
        )
        // Collections
        .route(
            "/organizations/:org_id/collections",
            get(collections::list_collections),
        )
        .route(
            "/organizations/:org_id/collections",
            post(collections::create_collection),
        )
        .route(
            "/organizations/:org_id/collections/:collection_id",
            delete(collections::delete_collection),
        )
        // Tags
        .route("/organizations/:org_id/tags", get(tags::list_tags))
        .route("/organizations/:org_id/tags", post(tags::create_tag))
        .route(
            "/organizations/:org_id/tags/:tag_name",
            delete(tags::delete_tag),
        )
        // Admin - Organizations
        .route(
            "/admin/organizations",
            get(organizations::list_organizations),
        )
        .route(
            "/admin/organizations",
            post(organizations::create_organization),
        )
        .route(
            "/admin/organizations/:org_id",
            get(organizations::get_organization),
        )
        .route(
            "/admin/organizations/:org_id",
            patch(organizations::update_organization),
        )
        .route(
            "/admin/organizations/:org_id",
            delete(organizations::delete_organization),
        )
        .route(
            "/admin/organizations/:org_id/users",
            get(organizations::list_organization_users),
        )
        // Admin - Users
        .route("/admin/users", get(users::list_users))
        .route("/admin/users", post(users::create_user))
        .route("/admin/users/:user_id", get(users::get_user))
        .route("/admin/users/:user_id", patch(users::update_user))
        .route("/admin/users/:user_id", delete(users::delete_user))
        // Admin - User Organizations
        .route(
            "/admin/users/:user_id/organizations",
            get(users::list_user_organizations),
        )
        .route(
            "/admin/users/:user_id/organizations/:org_id",
            post(users::add_user_to_organization),
        )
        .route(
            "/admin/users/:user_id/organizations/:org_id",
            patch(users::update_user_org_roles),
        )
        .route(
            "/admin/users/:user_id/organizations/:org_id",
            delete(users::remove_user_from_organization),
        )
        // Authentication (public endpoints)
        .route("/auth/login", post(auth::login))
        .route("/auth/select-org", post(auth::select_org))
        // Authentication (authenticated endpoints)
        .route("/auth/me", get(auth::get_me))
        .with_state(state.clone())
        // Add auth middleware to extract tokens from headers
        .layer(middleware::from_fn_with_state(state, auth_middleware))
}
