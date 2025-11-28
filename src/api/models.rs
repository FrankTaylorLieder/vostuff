use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

// Item types
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Vinyl,
    Cd,
    Cassette,
    Book,
    Score,
    Electronics,
    Misc,
}

// Item states
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ItemState {
    Current,
    Loaned,
    Missing,
    Disposed,
}

// Vinyl specific enums
#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VinylSize {
    #[serde(rename = "12_inch")]
    TwelveInch,
    #[serde(rename = "6_inch")]
    SixInch,
    Other,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VinylSpeed {
    #[serde(rename = "33")]
    ThirtyThree,
    #[serde(rename = "45")]
    FortyFive,
    Other,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum VinylChannels {
    Mono,
    Stereo,
    Surround,
    Other,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, Clone)]
#[serde(rename_all = "snake_case")]
pub enum Grading {
    Mint,
    NearMint,
    Excellent,
    Good,
    Fair,
    Poor,
}

// Item response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct Item {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub item_type: ItemType,
    pub state: ItemState,
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub location_id: Option<Uuid>,
    pub date_entered: DateTime<Utc>,
    pub date_acquired: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Create item request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateItemRequest {
    pub item_type: ItemType,
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub location_id: Option<Uuid>,
    pub date_acquired: Option<NaiveDate>,
}

// Update item request
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateItemRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub location_id: Option<Uuid>,
    pub date_acquired: Option<NaiveDate>,
    pub state: Option<ItemState>,
}

// Vinyl details
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct VinylDetails {
    pub item_id: Uuid,
    pub size: Option<VinylSize>,
    pub speed: Option<VinylSpeed>,
    pub channels: Option<VinylChannels>,
    pub disks: Option<i32>,
    pub media_grading: Option<Grading>,
    pub sleeve_grading: Option<Grading>,
}

// Location
#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct Location {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateLocationRequest {
    pub name: String,
}

// Collection
#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct Collection {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCollectionRequest {
    pub name: String,
    pub description: Option<String>,
    pub notes: Option<String>,
}

// Tag
#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct Tag {
    pub organization_id: Uuid,
    pub name: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTagRequest {
    pub name: String,
}

// Organization
#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct Organization {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrganizationRequest {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOrganizationRequest {
    pub name: Option<String>,
    pub description: Option<String>,
}

// User
#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub name: String,
    pub identity: String,
    #[serde(skip_serializing)] // Never serialize password hash
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub name: String,
    pub identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub name: Option<String>,
    pub identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

// Authentication models
#[derive(Debug, Deserialize, ToSchema)]
pub struct LoginRequest {
    pub identity: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct LoginResponse {
    pub token: String,
    pub expires_in: i64, // seconds
    pub user: UserInfo,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UserInfo {
    pub id: Uuid,
    pub name: String,
    pub identity: String,
    pub organizations: Vec<Organization>,
}

// User organization membership
#[derive(Debug, Serialize, Deserialize, ToSchema, sqlx::FromRow)]
pub struct UserOrganization {
    pub user_id: Uuid,
    pub organization_id: Uuid,
    pub created_at: DateTime<Utc>,
}

// Error response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
    pub message: String,
}

// Pagination
#[derive(Debug, Deserialize, ToSchema, utoipa::IntoParams)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_per_page")]
    pub per_page: i64,
}

fn default_page() -> i64 {
    1
}

fn default_per_page() -> i64 {
    50
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: i64,
    pub page: i64,
    pub per_page: i64,
    pub total_pages: i64,
}