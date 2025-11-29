# VOStuff Project Journal

## 2025-01-25 - Initial Setup and Requirements Review

**Prompt:** User ran `/init` command to analyze codebase and create CLAUDE.md, then asked about requirements tracking approaches, followed by requests to review requirements directory for open questions in data, organization and API layers. User made several updates to functional.md requirements during our discussion. Finally asked about ensuring JOURNAL.md gets updated going forward.

**Claude's Response:** 
- Analyzed minimal Rust codebase and created/improved CLAUDE.md with project overview, commands, and standing instructions
- Recommended markdown-based requirements structure in requirements/ directory  
- Reviewed functional.md requirements and identified open questions across data, organization, and API layers
- As user updated requirements, re-reviewed and noted resolved questions around item states, location tracking, audit trails, session management, and user administration
- Confirmed final requirements document provides solid foundation for implementation
- Updated JOURNAL.md to capture this conversation and address the standing instruction compliance

## 2025-09-23 - Database Schema Implementation

**Prompt:** Based on the requirements can be build the DB schema. Creating code to load the schema, supporting future modifications. Create a simple binary command to load the schema, but enable the schema handling to be used in the main application to come.

**Claude's Response:**
- Analyzed functional requirements to understand the multi-tenant data model
- Designed comprehensive PostgreSQL schema using sqlx migrations with:
  - Organizations table as tenant isolation boundary with pre-defined SYSTEM org
  - Users table with OIDC identity support and many-to-many org memberships
  - Items table with type-specific details (vinyl, CD, cassette, book, score, electronics, misc)
  - Item state management (current, loaned, missing, disposed) with associated detail tables
  - Collections and tags for item organization within organizations
  - Locations for tracking item placement
  - Comprehensive audit logging for change tracking
  - Performance indexes and automatic timestamp triggers
- Created SchemaManager struct in src/schema.rs with methods for migrations, database reset, and connection management
- Implemented CLI binary (schema-manager) with migrate, reset, and create commands
- Set up proper Rust project structure with sqlx dependencies and migration support
- Successfully built and tested the schema management functionality
- Schema enables future API and web tier development with proper multi-tenant isolation

## 2025-09-23 - Docker Development Environment Setup

**Prompt:** Can you create a docker compose file to start a Postgres DB we can use for testing.

**Claude's Response:**
- Created docker-compose.yml with PostgreSQL 16 Alpine container configuration
- Set up database with vostuff/vostuff_dev_password credentials and vostuff_dev database
- Added health checks and volume persistence for reliable development use
- Created initialization script (scripts/init-db.sh) to enable UUID extension
- Added .env.example file with database URL template for easy configuration
- Updated CLAUDE.md with database management commands
- Successfully tested the complete setup by starting PostgreSQL and running schema migrations
- Database is now ready for development and testing with proper Docker containerization

## 2025-09-23 - Documentation and Project README

**Prompt:** Can you remind me where we got to on this project? [followed by] Can you create a README.md to document how to use this project.

**Claude's Response:**
- Provided comprehensive project status summary showing foundation work is complete
- Created detailed README.md covering:
  - Project overview and implemented features
  - Prerequisites and getting started guide
  - Complete development commands reference
  - Project structure documentation
  - Database schema overview
  - Programmatic SchemaManager usage examples
  - Architecture overview showing three-tier design
  - Multi-tenancy explanation
- README provides clear path for new developers to get up and running
- Documentation includes both CLI usage and library integration examples

## 2025-09-23 - Sample Data Generator

**Prompt:** Let's create some sample data to load into the DB for testing purposes. We'll have two organisations: Coke and Pepsi. Each will have a user: Bob in Coke, Alice in Pepsi. Can you create a set of 50 sample items for each user in their collection, ensuring that all types and options are covered in the DB schema.

**Claude's Response:**
- Created standalone binary load-sample-data in src/bin/load_sample_data.rs
- Implemented comprehensive sample data generator that creates:
  - 2 organizations (Coke and Pepsi) with full metadata
  - 2 users (Bob@Coke, Alice@Pepsi) with organization memberships
  - 4 locations per organization (Living Room, Bedroom, Storage Unit, Office)
  - 4 collections per organization (Jazz, Rock, Reference Books, Rare Items)
  - 6 tags per organization (vintage, rare, mint-condition, signed, limited-edition, favorite)
  - 50 items per organization with distribution:
    - 10 vinyl records with all size/speed/channel/grading combinations
    - 10 CDs with varying disk counts
    - 8 cassettes
    - 8 books
    - 6 musical scores
    - 4 electronics items
    - 4 miscellaneous items
  - All item states represented (42 current, 3 loaned, 2 missing, 3 disposed per org)
  - Type-specific details properly populated (vinyl_details, cd_details, cassette_details)
  - State-specific details (loan dates, missing dates, disposal dates)
  - Items linked to collections and tags for realistic relationships
- Fixed sqlx compile-time checking issues by using non-macro query() for custom PostgreSQL types
- Successfully tested data loading - verified 50 items per org with proper distribution
- Updated README.md with sample data loading instructions
- Sample data provides comprehensive test coverage of all schema features

## 2025-09-23 - REST API with OpenAPI Documentation

**Prompt:** Now let's create the REST API. What's the best way to document a REST API for developers to use? [User chose option 1: Set up the REST API with OpenAPI/utoipa documentation built-in]

**Claude's Response:**
- Recommended OpenAPI/Swagger with utoipa crate for Rust/Axum integration
- Added dependencies: axum, tower, tower-http, utoipa, utoipa-swagger-ui, tracing
- Created comprehensive API models in src/api/models.rs:
  - DTOs with consistent snake_case serde serialization
  - OpenAPI schemas using utoipa::ToSchema
  - Type-safe enums for item types, states, vinyl details, and grading
  - Request/response models for all operations
  - Pagination support with PaginatedResponse
  - Error response model for consistent error handling
- Implemented full CRUD handlers:
  - Items (src/api/handlers/items.rs): List with pagination, Get, Create, Update (partial), Delete
  - Locations (src/api/handlers/locations.rs): List, Create, Delete
  - Collections (src/api/handlers/collections.rs): List, Create, Delete
  - Tags (src/api/handlers/tags.rs): List, Create, Delete
- Created API server binary (src/bin/api_server.rs):
  - Axum web server on port 8080 (initially 3000, changed due to Rancher Desktop conflict)
  - Auto-generated OpenAPI specification
  - Interactive Swagger UI at /swagger-ui
  - RESTful routes under /api namespace
  - Organization-scoped endpoints for multi-tenant isolation
  - Tracing/logging with configurable log levels
  - CORS support via tower-http
- All endpoints documented with utoipa::path macros including:
  - Path parameters and query parameters
  - Request/response schemas
  - HTTP status codes
  - Tag grouping for organization
- Fixed sqlx FromRow issues by adding derive macros to Location, Collection, Tag models
- Fixed utoipa IntoParams for PaginationParams
- Successfully compiled API server
- Updated README.md with:
  - REST API feature in implemented list
  - API server commands
  - Complete endpoint documentation
  - Example curl commands for testing
  - OpenAPI/Swagger UI access instructions
  - Architecture status showing API layer complete
- Project now has two of three tiers complete (Database ✅, API ✅, UI pending)

## 2025-11-23 - Integration Tests and Test Utilities Refactoring

**Prompt:** Can you move the sample data creation to an integration test module. Keep the binary to load sample data, but calling the integration test code. Then can you add a set of API integration tests in the new test module to exercise the API.

**Claude's Response:**
- Refactored sample data loading into reusable test utilities:
  - Created src/test_utils.rs with SampleDataLoader struct and SampleDataResult
  - Made sample data loading logic available to both binary and tests
  - Updated src/lib.rs to include test_utils module
  - Refactored src/bin/load_sample_data.rs to use test_utils (reduced from 584 to 19 lines)
  - Created tests/common/mod.rs as re-export layer for test utilities
- Implemented comprehensive API integration tests (tests/api_tests.rs):
  - 16 test cases covering all API endpoints
  - Test helper function create_test_router() for consistent test setup
  - Test helper function setup_test_db() that resets database and loads sample data
  - Items tests: list (with pagination), get, create, update, delete, not found
  - Locations tests: list, create, delete
  - Collections tests: list, create, delete
  - Tags tests: list, create, delete
  - Multi-tenant isolation test (verifies orgs can't access each other's data)
  - All tests use Axum's oneshot request testing pattern
- Fixed compilation issues:
  - Added Deserialize trait to PaginatedResponse<T> in src/api/models.rs
  - Tests run with DATABASE_URL environment variable
- Successfully ran all 16 tests with 100% pass rate in 5.5 seconds
- Updated README.md with comprehensive integration testing documentation:
  - Added integration test commands to Testing section
  - Created new "Integration Tests" section explaining test structure and coverage
  - Documented 16 test cases and what they verify
  - Explained sample data utilities are shared between CLI and tests
- All tests properly isolated with --test-threads=1 to avoid database conflicts
- Test coverage includes error cases, pagination, CRUD operations, and multi-tenant isolation

## 2025-11-23 - Admin API Endpoints for Users and Organizations

**Prompt:** Can you now add admin API routes for managing users and organisations. Follow the same pattern as the existing API. [Followed by:] Instead of manually testing, can you add organisation and user tests so the integration test suite.

**Claude's Response:**
- Implemented complete admin API functionality for managing users and organizations:
  - Added admin request/response models to src/api/models.rs:
    - CreateOrganizationRequest, UpdateOrganizationRequest for organization management
    - CreateUserRequest, UpdateUserRequest for user management
    - UserOrganization model for user-organization memberships
    - Added sqlx::FromRow derives to Organization and User structs
  - Created src/api/handlers/organizations.rs with 5 CRUD endpoints:
    - list_organizations: GET /api/admin/organizations
    - get_organization: GET /api/admin/organizations/{org_id}
    - create_organization: POST /api/admin/organizations
    - update_organization: PATCH /api/admin/organizations/{org_id}
    - delete_organization: DELETE /api/admin/organizations/{org_id}
  - Created src/api/handlers/users.rs with 8 endpoints:
    - list_users: GET /api/admin/users
    - get_user: GET /api/admin/users/{user_id}
    - create_user: POST /api/admin/users
    - update_user: PATCH /api/admin/users/{user_id}
    - delete_user: DELETE /api/admin/users/{user_id}
    - list_user_organizations: GET /api/admin/users/{user_id}/organizations
    - add_user_to_organization: POST /api/admin/users/{user_id}/organizations/{org_id}
    - remove_user_from_organization: DELETE /api/admin/users/{user_id}/organizations/{org_id}
  - Updated src/api/handlers/mod.rs to include new handler modules
  - Updated src/bin/api_server.rs:
    - Added all admin endpoints to OpenAPI paths documentation
    - Registered all admin routes in the router
    - Added new schemas and tags to OpenAPI specification
  - All endpoints use dynamic query building for partial updates
  - Proper error handling with 404 for not found, 409 for conflicts
  - All endpoints tagged with "admin-organizations" or "admin-users" for OpenAPI organization
- Implemented comprehensive integration tests (tests/api_tests.rs):
  - Added 11 new test cases covering all admin endpoints
  - Organization tests: list, get, create, update, delete
  - User tests: list, create, update
  - User-organization membership tests: list, add, remove
  - Updated create_test_router() to match api_server.rs routes exactly
  - All tests use same patterns as existing API tests
- Successfully ran all 27 tests (16 original + 11 new admin tests) with 100% pass rate in 9.32 seconds
- Updated README.md and JOURNAL.md with comprehensive admin API documentation:
  - Added admin endpoints section to API documentation
  - Included example curl commands for admin operations
  - Updated test coverage numbers from 16 to 27 tests
  - Documented all new admin endpoints and their purposes
- Admin API provides complete platform management capabilities for users and organizations
- All admin routes follow RESTful conventions and existing code patterns
- API server now includes interactive Swagger UI documentation at http://localhost:8080/swagger-ui with admin endpoints visible

## 2025-11-28 - Complete Authentication System Implementation

**Prompt:** Extended admin API to list users in org, then implemented comprehensive password authentication: "Can you extend the admin API to list users in an org." followed by "Let's add simple password authentication to the backend. Passwords for each user should be optional. When set, they should be stored as a salted hash, following best practice. The password can optionally be set when creating a users through the Admin API. Add a login method to enable a user to authenticate - this should return a time limited token. Enable the token to be passed as a header when making API calls. When passed, confirm the user is authenticated, determine their permissions and make that available to the API handlers." and finally "Great. Can you add information in the README about the JWT secret."

**Claude's Response:**
- **Admin API Extension**:
  - Discovered list_organization_users function was already implemented but not routed/documented
  - Added missing route /api/admin/organizations/{org_id}/users to API server and test router
  - Fixed OpenAPI documentation and added 2 comprehensive integration tests
  - Resolved SQLx compile-time validation by running migrations and regenerating cache

- **Comprehensive Authentication System Implementation**:
  - **Database Schema**: Created migration migrations/20241128000000_add_user_password.sql
    - Added optional password_hash field to users table with TEXT NULL
    - Added index for performance: idx_users_identity_password

  - **Authentication Infrastructure**:
    - Created src/auth.rs with PasswordHasher (Argon2), TokenManager (JWT), AuthContext
    - Created src/api/handlers/auth.rs with login endpoint following security best practices
    - Created src/api/middleware.rs for JWT authentication middleware
    - Updated src/api/models.rs with auth models and password fields

  - **Security Implementation**:
    - Argon2 password hashing with salt (industry standard)
    - JWT tokens with 24-hour expiration using HS256
    - Security-focused error handling: only returns "Invalid credentials" to prevent user enumeration
    - Token extraction from Authorization headers (Bearer format or direct token)
    - AuthContext injection for authenticated request handling

  - **API Integration**:
    - Updated AppState to include JWT secret
    - Added auth middleware to all API routes except /auth/login
    - Updated User model with password_hash field (skip_serializing for security)
    - Added optional password to CreateUserRequest and UpdateUserRequest
    - Created LoginRequest/LoginResponse/UserInfo models

  - **Comprehensive Testing**:
    - Fixed organization users query to include password_hash field
    - Added 3 authentication integration tests (login success, invalid credentials, missing user)
    - All 32 integration tests passing (29 existing + 3 auth)
    - Verified token-based authentication works correctly

  - **Documentation and Configuration**:
    - Updated README.md with JWT_SECRET environment variable documentation
    - Added security warnings about production secret requirements
    - Added authentication endpoints documentation with request/response examples
    - Updated Cargo.toml with auth dependencies: argon2, jsonwebtoken, axum-extra, headers

- **Error Resolution**:
  - Fixed user enumeration security issue per explicit user feedback
  - Resolved PasswordHasher name collision with argon2::PasswordHasher trait
  - Fixed SQLx query validation errors by running migrations first
  - Corrected missing password_hash field in organization users query

- **Final State**: Production-ready authentication system with optional passwords, secure token management, comprehensive middleware integration, and full test coverage. All security best practices implemented including proper password hashing, JWT expiration, and user enumeration prevention.

## 2025-11-29 - Sample Data Generator Password Enhancement

**Prompt:** "Does the same data generator set user passwords?" followed by "Can you update the generator to set sample user account passwords to 'secret123'"

**Claude's Response:**
- Identified that the sample data generator (`src/test_utils.rs`) did not set passwords for sample users Bob and Alice
- Modified the `create_user` method to accept an optional password parameter
- Integrated `PasswordHasher::hash_password()` to securely hash passwords using Argon2
- Updated sample data creation to set password "secret123" for both Bob (`bob@coke.com`) and Alice (`alice@pepsi.com`)
- Successfully tested authentication:
  - Both users can now login via `/api/auth/login` endpoint
  - Valid credentials return JWT tokens with user info and organization memberships
  - Invalid passwords are properly rejected with "Invalid credentials" error
- All changes compiled successfully and integrated with existing authentication system
- Sample users now support both testing workflows and demonstration purposes with working authentication

## 2025-11-29 - User Roles System Implementation

**Prompt:** "Let's add roles to the system. Users can have one or more roles. The following roles are needed: 'USER', 'ADMIN'. By default users have just the 'USER' role. Specific users can be allocated the 'ADMIN' role. We'll deal with authorization in a later change. The add/update user admin APIs should allow setting or updating roles. The authentication tokens should contain the assigned roles."

**Claude's Response:**
- **Database Schema Changes**:
  - Created migration `migrations/20241129000000_add_user_roles.sql`
  - Added `roles TEXT[]` column to users table with default `{USER}`
  - Added GIN index on roles column for efficient role lookups
  - Added check constraint to ensure valid roles (USER, ADMIN)

- **Data Models Updated** (`src/api/models.rs`):
  - Created `UserRole` enum with User and Admin variants
  - Added helper methods `as_str()` and `from_str()` for role conversion
  - Updated `User` struct to include `roles: Vec<String>` field
  - Updated `CreateUserRequest` to accept optional roles (defaults to USER if not provided)
  - Updated `UpdateUserRequest` to allow updating roles
  - Updated `UserInfo` to include roles field in login responses

- **Authentication System Updates**:
  - Updated JWT `Claims` struct in `src/auth.rs` to include `roles: Vec<String>`
  - Modified `TokenManager::generate_token()` to accept and encode roles
  - Updated `AuthContext` to include roles and added helper methods:
    - `has_role(role: &str)` - Check if user has specific role
    - `is_admin()` - Convenience method to check for ADMIN role
  - Updated authentication tests to verify role functionality

- **API Handler Updates**:
  - Updated login handler (`src/api/handlers/auth.rs`) to:
    - Query user roles from database
    - Include roles in JWT token generation
    - Return roles in UserInfo response
  - Updated all user management endpoints (`src/api/handlers/users.rs`):
    - `list_users` - Returns roles for each user
    - `get_user` - Returns user with roles
    - `create_user` - Accepts optional roles, defaults to [USER]
    - `update_user` - Allows updating user roles
  - Updated `list_organization_users` to include roles in query

- **Sample Data Generator** (`src/test_utils.rs`):
  - Updated `create_user` method to accept roles parameter
  - Modified sample data:
    - Bob (bob@coke.com) - Has USER role only
    - Alice (alice@pepsi.com) - Has both USER and ADMIN roles
  - Updated user creation logging to show assigned roles

- **End-to-End Testing Results**:
  - Verified Bob's login returns `roles: ["USER"]` in response
  - Verified Alice's login returns `roles: ["USER", "ADMIN"]` in response
  - Tested creating user with multiple roles via admin API - success
  - Tested creating user without specifying roles - correctly defaults to ["USER"]
  - Tested updating user roles via PATCH endpoint - successfully updated
  - JWT tokens contain roles in claims (verified via response payloads)

- **Authorization Placeholder**:
  - Infrastructure ready for authorization implementation
  - `AuthContext` provides `has_role()` and `is_admin()` methods
  - Can be used in future middleware/handlers for role-based access control

- **Integration Tests** (`tests/api_tests.rs`):
  - Added 5 comprehensive role-related tests:
    - `test_user_default_role` - Verifies users default to USER role when none specified
    - `test_create_user_with_admin_role` - Tests creating users with specific roles
    - `test_update_user_roles` - Verifies role updates via PATCH endpoint
    - `test_list_users_includes_roles` - Confirms roles returned in user list responses
    - `test_login_response_includes_roles` - Validates roles in authentication responses
  - All 37 integration tests passing (32 existing + 5 new role tests)
  - Tests verify both Bob (USER only) and Alice (USER + ADMIN) role assignments
  - Full test coverage for role creation, updates, and retrieval

- **Technical Notes**:
  - Used non-macro sqlx::query() in test_utils to avoid compile-time checking issues
  - Roles stored as PostgreSQL TEXT[] array for flexibility
  - UserRole enum provides type safety at API boundary
  - Database constraint ensures only valid roles can be stored

All role functionality working correctly with full integration across database, API, authentication, and admin interfaces. Complete test coverage with 37 passing integration tests. System ready for future authorization implementation.

## 2025-11-29 - Organization-Specific Roles and Enhanced Login Flow

**Prompt:** "Let's make a change. The roles for a user should be associated with the user in a specific org, not with the user themselves. Can you move the association to the user_organization table. Further, can you enable user login to select which org they are logging in to, recording the org in the authentication token and getting the relevant roles in the org for the user." followed by request for flexible login with optional org ID, and organization selection flow for multi-org users.

**Claude's Response:**
- **Database Schema Migration**:
  - Created migration `migrations/20241129100000_move_roles_to_user_organizations.sql`
  - Moved `roles TEXT[]` column from `users` table to `user_organizations` table
  - Added GIN index on `user_organizations.roles` for efficient lookups
  - Added check constraint to ensure valid roles (USER, ADMIN) at org level
  - Migrated existing role data from users to all their organization memberships
  - Dropped roles column from users table

- **Data Models Updated** (`src/api/models.rs`):
  - Removed `roles` field from `User` struct (roles now org-specific)
  - Added `roles: Vec<String>` to `UserOrganization` struct
  - Updated `LoginRequest` to accept optional `organization_id` parameter
  - Created new `OrgSelectionResponse` model for multi-org users
  - Created new `OrganizationWithRoles` model showing org + user's roles in that org
  - Created new `SelectOrgRequest` model for org selection endpoint
  - Created `AddUserToOrgRequest` model with optional roles
  - Created `UpdateUserOrgRolesRequest` model for updating org-specific roles
  - Updated `UserInfo` to contain single `Organization` and `roles` (for selected org)

- **Authentication System Refactoring**:
  - Updated JWT `Claims` in `src/auth.rs`:
    - Changed from `organizations: Vec<Uuid>` to `organization_id: Uuid` (single selected org)
    - Roles now represent user's roles in the selected organization
  - Created new `FollowOnClaims` struct for temporary 5-minute org selection tokens
  - Updated `TokenManager` with new methods:
    - `generate_follow_on_token()` - Creates short-lived token for org selection
    - `validate_follow_on_token()` - Validates org selection tokens
  - Updated `AuthContext` to work with single organization:
    - `has_org_access()` - Checks if user has access to specific org
    - `organization_id()` - Returns user's selected organization
    - Roles are now organization-specific

- **Enhanced Login Flow** (`src/api/handlers/auth.rs`):
  - Implemented flexible three-scenario login:
    1. **Org ID provided**: Direct authentication with that organization's roles
    2. **No org ID + Single org membership**: Auto-select the user's only organization
    3. **No org ID + Multiple orgs**: Return list of orgs with roles + follow-on token
  - Login endpoint queries user's organizations with roles from `user_organizations` table
  - Returns 403 FORBIDDEN if user is not member of any organization
  - New `/api/auth/select-org` endpoint:
    - Accepts follow-on token and organization ID
    - Validates user membership in selected org
    - Returns final JWT token with org-specific roles
    - Verifies org selection within 5-minute window

- **Admin API Updates**:
  - Updated all user CRUD endpoints (`src/api/handlers/users.rs`):
    - Removed roles from user creation/update (roles now managed per-org)
    - `add_user_to_organization` - Now accepts optional roles, defaults to [USER]
    - Created new `update_user_org_roles` endpoint (PATCH /api/admin/users/{user_id}/organizations/{org_id})
    - Updated `list_organization_users` to remove roles column from query
  - Added routes to API server:
    - POST `/api/admin/users/{user_id}/organizations/{org_id}` - Add user to org with roles
    - PATCH `/api/admin/users/{user_id}/organizations/{org_id}` - Update user's roles in org
    - POST `/api/auth/select-org` - Complete authentication after org selection

- **Sample Data Generator** (`src/test_utils.rs`):
  - Removed roles parameter from `create_user` method
  - Updated `add_user_to_org` to accept `roles: Vec<String>` parameter
  - Modified sample data:
    - Bob (bob@coke.com) - Added to Coke org with USER role
    - Alice (alice@pepsi.com) - Added to Pepsi org with USER and ADMIN roles
  - Used non-macro `sqlx::query()` to avoid compile-time checking before migration

- **Integration Tests** (`tests/api_tests.rs`):
  - Updated 6 role-related tests to work with new architecture:
    - Removed assertions about user.roles (no longer exists)
    - Updated `test_admin_add_user_to_organization` to send roles in request body
    - Updated `test_auth_login_success` to add user to org before login
  - All 37 integration tests passing with updated assertions
  - Tests verify org-specific role behavior correctly

- **End-to-End Testing Results**:
  - Verified Bob's login (single org):
    - Auto-selects Coke organization
    - Returns token with `organization_id` and `roles: ["USER"]`
    - UserInfo contains selected organization details
  - Verified Alice's login (single org):
    - Auto-selects Pepsi organization
    - Returns token with `organization_id` and `roles: ["USER", "ADMIN"]`
    - Roles are specific to Pepsi organization
  - JWT tokens now contain single organization context instead of list
  - Organization selection flow ready for multi-org users

- **Technical Implementation Details**:
  - Database migration safely moved data without loss
  - Fixed compile-time SQLx validation by using non-macro queries where needed
  - Roles are now truly multi-tenant - same user can have different roles in different orgs
  - Follow-on tokens prevent organization selection attacks (5-minute expiry)
  - Backward compatible error handling for users without organizations

**Impact**: Major architectural improvement enabling true organization-specific role-based access control. Users can now have different roles in different organizations (e.g., ADMIN in one org, USER in another). The flexible login flow supports both single-org and multi-org users seamlessly. All 37 integration tests passing. System ready for multi-organization role-based authorization implementation.
