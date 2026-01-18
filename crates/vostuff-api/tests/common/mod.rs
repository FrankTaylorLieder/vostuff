use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode},
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use sqlx::PgPool;
use tower::ServiceExt;
use uuid::Uuid;
use vostuff_api::api::{models::LoginRequest, state::AppState};
use vostuff_core::auth::PasswordHasher;

/// Test context that holds database pool and app state
pub struct TestContext {
    pub pool: PgPool,
    pub state: AppState,
    pub app: Router,
}

impl TestContext {
    /// Create a new test context with a fresh database
    pub async fn new() -> Self {
        // Use test database URL
        let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev".to_string()
        });

        let pool = PgPool::connect(&database_url)
            .await
            .expect("Failed to connect to test database");

        // Clean database before tests
        Self::clean_database(&pool).await;

        let jwt_secret = "test_jwt_secret_for_integration_tests".to_string();
        let state = AppState::new(pool.clone(), jwt_secret);

        // Build the app router and nest under /api (same as in main)
        let api_router = vostuff_api::api::handlers::build_router(state.clone());
        let app = axum::Router::new().nest("/api", api_router);

        Self { pool, state, app }
    }

    /// Clean all tables in the database
    async fn clean_database(pool: &PgPool) {
        // Disable foreign key checks temporarily and truncate all tables
        let tables = vec![
            "item_tags",
            "item_collections",
            "item_disposed_details",
            "item_missing_details",
            "item_loan_details",
            "vinyl_details",
            "cd_details",
            "cassette_details",
            "items",
            "tags",
            "collections",
            "locations",
            "user_organizations",
            "users",
            "organizations",
        ];

        for table in tables {
            sqlx::query(&format!("TRUNCATE TABLE {} CASCADE", table))
                .execute(pool)
                .await
                .expect(&format!("Failed to truncate table {}", table));
        }
    }

    /// Create a test organization
    pub async fn create_organization(&self, name: &str, description: &str) -> Uuid {
        let rec = sqlx::query!(
            "INSERT INTO organizations (name, description) VALUES ($1, $2) RETURNING id",
            name,
            description
        )
        .fetch_one(&self.pool)
        .await
        .expect("Failed to create test organization");

        rec.id
    }

    /// Create a test user with password
    pub async fn create_user(&self, name: &str, identity: &str, password: &str) -> Uuid {
        let password_hash =
            PasswordHasher::hash_password(password).expect("Failed to hash password");

        let rec = sqlx::query!(
            "INSERT INTO users (name, identity, password_hash) VALUES ($1, $2, $3) RETURNING id",
            name,
            identity,
            password_hash
        )
        .fetch_one(&self.pool)
        .await
        .expect("Failed to create test user");

        rec.id
    }

    /// Add user to organization with roles
    pub async fn add_user_to_org(&self, user_id: Uuid, org_id: Uuid, roles: Vec<String>) {
        sqlx::query!(
            "INSERT INTO user_organizations (user_id, organization_id, roles) VALUES ($1, $2, $3)",
            user_id,
            org_id,
            &roles
        )
        .execute(&self.pool)
        .await
        .expect("Failed to add user to organization");
    }

    /// Login as a user and get JWT token
    pub async fn login(&self, identity: &str, password: &str, org_id: Option<Uuid>) -> String {
        let login_req = LoginRequest {
            identity: identity.to_string(),
            password: password.to_string(),
            organization_id: org_id,
        };

        let response = self.post("/api/auth/login", &login_req, None).await;

        assert_eq!(
            response.status,
            StatusCode::OK,
            "Login failed: {:?}",
            response.body
        );

        // Parse response - could be LoginResponse or OrgSelectionResponse
        let value: Value =
            serde_json::from_value(response.body).expect("Failed to parse login response");

        if let Some(token) = value.get("token").and_then(|t| t.as_str()) {
            token.to_string()
        } else {
            panic!("No token in login response: {:?}", value);
        }
    }

    /// Make a GET request
    pub async fn get(&self, path: &str, token: Option<&str>) -> TestResponse {
        self.request("GET", path, None::<&()>, token).await
    }

    /// Make a POST request
    pub async fn post<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
        token: Option<&str>,
    ) -> TestResponse {
        self.request("POST", path, Some(body), token).await
    }

    /// Make a PATCH request
    pub async fn patch<T: serde::Serialize>(
        &self,
        path: &str,
        body: &T,
        token: Option<&str>,
    ) -> TestResponse {
        self.request("PATCH", path, Some(body), token).await
    }

    /// Make a DELETE request
    pub async fn delete(&self, path: &str, token: Option<&str>) -> TestResponse {
        self.request("DELETE", path, None::<&()>, token).await
    }

    /// Make a generic HTTP request
    async fn request<T: serde::Serialize>(
        &self,
        method: &str,
        path: &str,
        body: Option<&T>,
        token: Option<&str>,
    ) -> TestResponse {
        let mut request_builder = Request::builder().method(method).uri(path);

        // Add Authorization header if token provided
        if let Some(token) = token {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", token));
        }

        // Add body if provided
        let request = if let Some(body) = body {
            let json = serde_json::to_string(body).expect("Failed to serialize body");
            request_builder
                .header("Content-Type", "application/json")
                .body(Body::from(json))
                .expect("Failed to build request")
        } else {
            request_builder
                .body(Body::empty())
                .expect("Failed to build request")
        };

        // Execute request
        let response = self
            .app
            .clone()
            .oneshot(request)
            .await
            .expect("Failed to execute request");

        let status = response.status();
        let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("Failed to read response body");

        let body: Value = if body_bytes.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&body_bytes)
                .unwrap_or_else(|_| Value::String(String::from_utf8_lossy(&body_bytes).to_string()))
        };

        TestResponse { status, body }
    }
}

/// Response from a test request
#[derive(Debug)]
pub struct TestResponse {
    pub status: StatusCode,
    pub body: Value,
}

impl TestResponse {
    /// Parse body as JSON into type T
    pub fn json<T: DeserializeOwned>(&self) -> T {
        serde_json::from_value(self.body.clone()).expect("Failed to deserialize response body")
    }

    /// Assert that status code matches expected
    pub fn assert_status(&self, expected: StatusCode) {
        assert_eq!(
            self.status, expected,
            "Expected status {}, got {}. Body: {:?}",
            expected, self.status, self.body
        );
    }

    /// Assert that response is successful (2xx)
    pub fn assert_success(&self) {
        assert!(
            self.status.is_success(),
            "Expected success status, got {}. Body: {:?}",
            self.status,
            self.body
        );
    }
}

/// Test fixture with pre-created users and organizations
pub struct TestFixture {
    pub ctx: TestContext,
    pub org1_id: Uuid,
    pub org2_id: Uuid,
    pub user1_id: Uuid, // User in org1 with USER role
    pub user2_id: Uuid, // User in org1 with ADMIN role
    pub user3_id: Uuid, // User in org2 with USER role
    pub user1_token: String,
    pub user2_token: String,
    pub user3_token: String,
}

impl TestFixture {
    /// Create a new test fixture with sample data
    pub async fn new() -> Self {
        let ctx = TestContext::new().await;

        // Create organizations
        let org1_id = ctx
            .create_organization("Test Org 1", "First test organization")
            .await;
        let org2_id = ctx
            .create_organization("Test Org 2", "Second test organization")
            .await;

        // Create users
        let user1_id = ctx
            .create_user("User One", "user1@test.com", "password123")
            .await;
        let user2_id = ctx
            .create_user("User Two", "user2@test.com", "password123")
            .await;
        let user3_id = ctx
            .create_user("User Three", "user3@test.com", "password123")
            .await;

        // Add users to organizations with roles
        ctx.add_user_to_org(user1_id, org1_id, vec!["USER".to_string()])
            .await;
        ctx.add_user_to_org(
            user2_id,
            org1_id,
            vec!["USER".to_string(), "ADMIN".to_string()],
        )
        .await;
        ctx.add_user_to_org(user3_id, org2_id, vec!["USER".to_string()])
            .await;

        // Login users to get tokens
        let user1_token = ctx
            .login("user1@test.com", "password123", Some(org1_id))
            .await;
        let user2_token = ctx
            .login("user2@test.com", "password123", Some(org1_id))
            .await;
        let user3_token = ctx
            .login("user3@test.com", "password123", Some(org2_id))
            .await;

        Self {
            ctx,
            org1_id,
            org2_id,
            user1_id,
            user2_id,
            user3_id,
            user1_token,
            user2_token,
            user3_token,
        }
    }
}
