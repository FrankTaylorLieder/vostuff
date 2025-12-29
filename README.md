# VOStuff

A three-tier Rust application for tracking collections of stuff - vinyl records, CDs, cassettes, books, scores, electronics, and more. Built with multi-tenant architecture from the ground up.

## Features

### Implemented
- **Multi-tenant Architecture**: Organizations provide hard isolation boundaries
- **PostgreSQL Database Schema**: Comprehensive schema with:
  - Organizations and users with OIDC identity support
  - Items with type-specific details (vinyl, CD, cassette, book, score, electronics, misc)
  - Item state management (current, loaned, missing, disposed)
  - Collections, tags, and locations for organization
  - Organization-specific role-based access control (USER, ADMIN, OWNER)
  - Comprehensive audit logging
- **Schema Management**: CLI tool and reusable library for database migrations
- **Docker Development Environment**: Containerized PostgreSQL for easy local development
- **Sample Data Generator**: Load realistic test data with 100+ items across two organizations
- **REST API**: Full-featured API built with Axum
  - CRUD endpoints for items, locations, collections, and tags
  - Organization-scoped operations
  - Admin endpoints for managing users and organizations
  - User-organization membership and role management
  - JWT-based authentication with password support
  - Multi-organization authentication flow with intelligent org selection
  - Organization-specific role-based access control
  - Pagination support
  - Interactive OpenAPI/Swagger documentation at `/swagger-ui`
  - Type-safe request/response models
  - Comprehensive error handling

- **Web UI**: Leptos SSR web application with:
  - User authentication with JWT tokens stored in HTTP-only cookies
  - Multi-organization login flow with automatic or manual org selection
  - Protected routes with authentication context
  - Clean, responsive UI with custom CSS
  - Server functions that call the REST API

### Planned
- OIDC Authentication integration

## Prerequisites

- Rust 1.86.0 or later (edition 2024)
- Docker and Docker Compose
- PostgreSQL client tools (optional, for manual database access)
- WebAssembly target (for web UI): `rustup target add wasm32-unknown-unknown`
- cargo-leptos (for running the web UI): `cargo install cargo-leptos`
- sqlx-cli (for database tooling): `cargo install sqlx-cli --no-default-features --features postgres`

## Getting Started

### 1. Clone the Repository

```bash
git clone <repository-url>
cd vostuff
```

### 2. Set Up Environment Variables

Create a `.env` file from the example:

```bash
cp .env.example .env
```

The default configuration connects to the Docker PostgreSQL instance:
```
DATABASE_URL=postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev
JWT_SECRET=your_secret_key_here_change_in_production
API_BASE_URL=http://localhost:8080
```

**Important Security Notes:**
- `DATABASE_URL`: Connection string for PostgreSQL database
- `JWT_SECRET`: Secret key for signing JWT authentication tokens
  - **MUST be changed in production** to a strong, randomly generated secret
  - Used for signing and validating JWT tokens for user authentication
  - If not set, defaults to a development-only secret (insecure for production)
- `API_BASE_URL`: Base URL for the REST API (used by the web server to call API endpoints)

### 3. Start the Database

For local development, use the database-only Docker Compose file:

```bash
docker-compose -f docker-compose-dev.yml up -d
```

This starts a PostgreSQL 16 container with:
- Database: `vostuff_dev`
- User: `vostuff`
- Password: `vostuff_dev_password`
- Port: `5432`

**Note**: `docker-compose-dev.yml` starts only the database, allowing you to run the API and web services locally with `cargo`. For running the full stack in Docker, use `docker-compose.yml` instead (see Docker Deployment section).

### 4. Run Database Migrations

**Initial setup** (when database is empty):

```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Build and run migrations with sqlx validation disabled
SQLX_OFFLINE=true DATABASE_URL=postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev cargo run --bin schema-manager migrate

# Generate sqlx metadata for future builds
cd crates/vostuff-api
DATABASE_URL=postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev cargo sqlx prepare
cd ../..
```

**After initial setup**:

```bash
DATABASE_URL=postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev cargo run --bin schema-manager migrate
```

This creates all tables, indexes, triggers, and the initial SYSTEM organization.

**Why SQLX_OFFLINE?** SQLx validates SQL queries at compile time against the database. On initial setup, the tables don't exist yet, causing compilation to fail. Setting `SQLX_OFFLINE=true` disables this validation. After running migrations once, you can generate metadata with `cargo sqlx prepare` to enable offline validation without database access.

### 5. Load Sample Data (Optional)

To test the application with sample data, run:

```bash
cargo run --bin load-sample-data
```

This creates:
- 2 organizations (Coke and Pepsi)
- 2 users (Bob@Coke, Alice@Pepsi)
- 50 items per organization covering all item types (vinyl, CD, cassette, book, score, electronics, misc)
- Various item states (current, loaned, missing, disposed)
- Collections, tags, and locations for each organization

### 6. Verify Setup

Check that the database is running and healthy:

```bash
docker-compose ps
```

You should see the `vostuff-postgres` container with status "Up (healthy)".

## Docker Deployment

The entire application stack can be run using Docker Compose, which orchestrates the database, API server, and web UI together.

### Running with Docker Compose

```bash
# Build and start all services
docker-compose up --build

# Or run in background (detached mode)
docker-compose up -d --build

# View logs
docker-compose logs -f

# View logs for a specific service
docker-compose logs -f api
docker-compose logs -f web

# Stop all services
docker-compose down

# Stop and remove volumes (WARNING: deletes all data)
docker-compose down -v
```

### What Gets Started

The Docker Compose configuration starts four services:

1. **postgres** - PostgreSQL 16 database
   - Port: 5432
   - Includes healthcheck for startup coordination

2. **migrations** - Database schema initialization
   - Runs `schema-manager migrate` once on startup
   - Creates all tables, indexes, and triggers
   - Waits for postgres to be healthy before running

3. **api** - REST API server
   - Port: 8080
   - Swagger UI: http://localhost:8080/swagger-ui
   - Waits for migrations to complete before starting

4. **web** - Leptos web application
   - Port: 3001
   - Web UI: http://localhost:3001
   - Waits for API server to be available

### Environment Variables

You can customize the deployment with a `.env` file:

```bash
# JWT secret (IMPORTANT: change in production)
JWT_SECRET=your_strong_secret_key_here

# Logging level
RUST_LOG=info
```

The database credentials are set in `docker-compose.yml` and should be changed for production deployments.

### Accessing the Application

Once all services are running:

- **Web UI**: http://localhost:3001
- **REST API**: http://localhost:8080
- **Swagger UI**: http://localhost:8080/swagger-ui
- **PostgreSQL**: localhost:5432

### Loading Sample Data

To load sample data into the Dockerized environment:

```bash
# Connect to the API container and run the sample data loader
docker exec -it vostuff-api load-sample-data
```

### Rebuilding After Code Changes

```bash
# Rebuild and restart specific services
docker-compose up -d --build api
docker-compose up -d --build web

# Or rebuild everything
docker-compose up -d --build
```

### Docker Build Details

The application uses two multi-stage Dockerfiles:

- **Dockerfile.api** - Builds the API server and schema-manager binaries
  - Uses Rust 1.86 for compilation (required for edition 2024 support)
  - Produces optimized release binaries
  - Final image based on Debian Bookworm Slim

- **Dockerfile.web** - Builds the Leptos web application
  - Uses Rust 1.86 for compilation (required for edition 2024 support)
  - Installs cargo-leptos for building
  - Compiles both server binary and WASM client code
  - Includes static assets (CSS, JS, WASM)
  - Final image based on Debian Bookworm Slim

Initial build time is approximately 5-10 minutes due to Rust compilation, but Docker layer caching significantly speeds up subsequent builds.

## Development Commands

### Building and Running

```bash
# Build the project
cargo build

# Run the main application
cargo run

# Build optimized release version
cargo build --release
```

### Database Management

```bash
# Install sqlx-cli (one-time, if not already installed)
cargo install sqlx-cli --no-default-features --features postgres

# Start PostgreSQL (database only, for local development)
docker-compose -f docker-compose-dev.yml up -d

# Stop PostgreSQL
docker-compose -f docker-compose-dev.yml down

# Run migrations (initial setup - database empty)
SQLX_OFFLINE=true DATABASE_URL=postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev cargo run --bin schema-manager migrate

# Generate sqlx metadata (run after initial migration)
cd crates/vostuff-api
DATABASE_URL=postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev cargo sqlx prepare
cd ../..

# Run migrations (after initial setup)
DATABASE_URL=postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev cargo run --bin schema-manager migrate

# Reset database (WARNING: deletes all data)
SQLX_OFFLINE=true DATABASE_URL=postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev cargo run --bin schema-manager reset

# Create database only (no migrations)
SQLX_OFFLINE=true DATABASE_URL=postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev cargo run --bin schema-manager create

# Load sample data for testing
DATABASE_URL=postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev cargo run --bin load-sample-data
```

**Note on SQLX_OFFLINE**: Use `SQLX_OFFLINE=true` when the database schema doesn't exist or has been reset. After running migrations once, you can omit it or set up automatic offline mode (see next section).

**Automatic Offline Mode** (optional): Create `.cargo/config.toml` in the project root:

```toml
[env]
SQLX_OFFLINE = "true"
```

This makes offline mode the default for all builds, eliminating the need to set the environment variable each time.

### API Server

```bash
# Run the REST API server
cargo run --bin api-server

# The server will start on http://localhost:8080
# Swagger UI available at http://localhost:8080/swagger-ui
```

### Web UI

The web UI is built with Leptos and uses cargo-leptos for development and building.

```bash
# Install prerequisites (if not already installed)
rustup target add wasm32-unknown-unknown
cargo install cargo-leptos

# Run the web UI in development mode (with hot reload)
cargo leptos watch

# The web server will start on http://localhost:3001
# The API server must be running on http://localhost:8080

# Build the web UI for production
cargo leptos build --release

# Run the production build
cargo leptos serve --release
```

**Environment Variables for Web Server:**
- `DATABASE_URL`: PostgreSQL connection string (required)
- `JWT_SECRET`: Secret for validating JWT tokens (required)
- `API_BASE_URL`: URL of the REST API server (default: http://localhost:8080)

The web UI provides:
- User authentication with login page
- Multi-organization support with automatic or manual org selection
- Protected dashboard page (requires authentication)
- Logout functionality
- JWT tokens stored in HTTP-only cookies for security

### Testing and Quality

The project includes a comprehensive test suite with unit tests and integration tests.

#### API Integration Tests

The API has integration tests covering:
- **Authentication**: Login, multi-org selection, `/api/auth/me` endpoint
- **Multi-tenancy isolation**: Ensuring users cannot access other organizations' data
- **Items CRUD**: Create, read, update, delete operations
- **Authorization**: Role-based access control

```bash
# Run all tests
cargo test

# Run all API integration tests (recommended: use --test-threads=1 for database isolation)
cargo test --package vostuff-api --tests -- --test-threads=1

# Run specific test suites
cargo test --package vostuff-api --test auth_tests -- --test-threads=1
cargo test --package vostuff-api --test multi_tenancy_tests -- --test-threads=1
cargo test --package vostuff-api --test items_tests -- --test-threads=1

# Run specific test
cargo test <test_name>

# Run unit tests only (fast, no database required)
cargo test --package vostuff-core
cargo test --package vostuff-api --lib

# Check code without building
cargo check

# Run linter
cargo clippy

# Format code
cargo fmt
```

**Note**: Integration tests require a running PostgreSQL database. They use the `DATABASE_URL` from your environment or `.env` file. Each test suite cleans the database before running to ensure test isolation.

## Project Structure

```
vostuff/
├── crates/                  # Workspace members
│   ├── vostuff-core/       # Shared code (auth, models)
│   ├── vostuff-api/        # REST API server
│   └── vostuff-web/        # Leptos web UI
│       ├── src/
│       │   ├── main.rs     # Web server entry point
│       │   ├── lib.rs      # Leptos app library
│       │   ├── app.rs      # Root app component
│       │   ├── pages/      # Page components
│       │   ├── components/ # Reusable components
│       │   └── server_fns/ # Server functions
│       ├── style/          # CSS styles
│       └── assets/         # Static assets
├── migrations/              # SQL migration files (sqlx)
│   └── 20240101000000_initial_schema.sql
├── scripts/                 # Helper scripts
│   └── init-db.sh          # Database initialization
├── requirements/
│   └── functional.md       # Functional requirements
├── docker-compose.yml      # Full stack (DB + API + Web + migrations)
├── docker-compose-dev.yml  # Database only for local development
├── Leptos.toml             # cargo-leptos configuration
├── CLAUDE.md              # Development guidelines
├── JOURNAL.md             # Development journal
├── TODO.md                # Task tracking
└── README.md              # This file
```

## Database Schema

The PostgreSQL schema implements a multi-tenant system with:

### Core Tables
- **organizations**: Tenant isolation boundary
- **users**: User accounts with OIDC identity and password authentication
- **user_organizations**: Many-to-many user/org membership with role-based access control (USER, ADMIN, OWNER)

### Item Management
- **items**: Core item data with type and state
- **locations**: User-defined storage locations
- **collections**: User-defined groupings
- **tags**: Flexible tagging system

### Type-Specific Details
- **vinyl_details**: Size, speed, channels, grading
- **cd_details**: Disk count
- **cassette_details**: Cassette count

### State Management
- **item_loan_details**: Loaned items tracking
- **item_missing_details**: Missing items tracking
- **item_disposed_details**: Disposed items tracking

### Audit Trail
- **audit_log**: Change tracking for all items

## Using the Schema Manager

The `SchemaManager` struct can be used programmatically in your code:

```rust
use vostuff::schema::SchemaManager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager = SchemaManager::new("postgresql://...").await?;

    // Run migrations
    manager.run_migrations().await?;

    // Get connection pool for queries
    let pool = manager.get_pool();

    // Use pool for database operations...

    manager.close().await;
    Ok(())
}
```

## REST API

The REST API is built with Axum and provides comprehensive OpenAPI/Swagger documentation.

### Starting the API Server

```bash
# Make sure database is running
docker-compose up -d

# Start the API server
cargo run --bin api-server
```

The server starts on `http://localhost:8080` with interactive API documentation at `http://localhost:8080/swagger-ui`.

### API Endpoints

#### Organization-Scoped Endpoints

All organization-scoped endpoints enforce multi-tenant isolation:

**Items**
- `GET /api/organizations/{org_id}/items` - List items (with pagination)
- `POST /api/organizations/{org_id}/items` - Create an item
- `GET /api/organizations/{org_id}/items/{item_id}` - Get item details
- `PATCH /api/organizations/{org_id}/items/{item_id}` - Update an item
- `DELETE /api/organizations/{org_id}/items/{item_id}` - Delete an item

**Locations**
- `GET /api/organizations/{org_id}/locations` - List locations
- `POST /api/organizations/{org_id}/locations` - Create a location
- `DELETE /api/organizations/{org_id}/locations/{location_id}` - Delete a location

**Collections**
- `GET /api/organizations/{org_id}/collections` - List collections
- `POST /api/organizations/{org_id}/collections` - Create a collection
- `DELETE /api/organizations/{org_id}/collections/{collection_id}` - Delete a collection

**Tags**
- `GET /api/organizations/{org_id}/tags` - List tags
- `POST /api/organizations/{org_id}/tags` - Create a tag
- `DELETE /api/organizations/{org_id}/tags/{tag_name}` - Delete a tag

#### Admin Endpoints

Admin endpoints for platform-level management of users and organizations:

**Organizations**
- `GET /api/admin/organizations` - List all organizations
- `POST /api/admin/organizations` - Create a new organization
- `GET /api/admin/organizations/{org_id}` - Get organization details
- `PATCH /api/admin/organizations/{org_id}` - Update an organization
- `DELETE /api/admin/organizations/{org_id}` - Delete an organization

**Users**
- `GET /api/admin/users` - List all users
- `POST /api/admin/users` - Create a new user
- `GET /api/admin/users/{user_id}` - Get user details
- `PATCH /api/admin/users/{user_id}` - Update a user
- `DELETE /api/admin/users/{user_id}` - Delete a user

**User-Organization Memberships**
- `GET /api/admin/users/{user_id}/organizations` - List organizations for a user (with roles)
- `POST /api/admin/users/{user_id}/organizations/{org_id}` - Add user to organization with roles
- `PATCH /api/admin/users/{user_id}/organizations/{org_id}` - Update user's roles in an organization
- `DELETE /api/admin/users/{user_id}/organizations/{org_id}` - Remove user from organization
- `GET /api/admin/organizations/{org_id}/users` - List users in an organization

**Roles**: Each user-organization membership includes one or more roles:
- `USER` - Basic access to organization resources
- `ADMIN` - Administrative privileges within the organization
- `OWNER` - Full control including org settings and user management

#### Authentication Endpoints

Authentication endpoints for user login and JWT token management. The authentication flow intelligently handles users who belong to multiple organizations.

**Login** - `POST /api/auth/login`
- Password-based authentication with smart organization selection
- JWT tokens are scoped to a single organization and include org-specific roles

**Three Authentication Scenarios:**

1. **User belongs to multiple orgs, no org specified**
   - Request: `{"identity": "user@example.com", "password": "password"}`
   - Response: Organization selection response with follow-on token
   ```json
   {
     "organizations": [
       {
         "id": "uuid",
         "name": "Org Name",
         "roles": ["USER", "ADMIN"]
       }
     ],
     "follow_on_token": "temporary_token"
   }
   ```
   - Follow-on token valid for 5 minutes
   - Use `POST /api/auth/select-org` to complete authentication

2. **User belongs to single org (auto-selected)**
   - Request: `{"identity": "user@example.com", "password": "password"}`
   - Response: Direct authentication with JWT token
   ```json
   {
     "token": "jwt_token",
     "expires_in": 86400,
     "user": {
       "id": "uuid",
       "name": "User Name",
       "identity": "user@example.com",
       "organization": {
         "id": "uuid",
         "name": "Org Name"
       },
       "roles": ["USER"]
     }
   }
   ```

3. **User specifies organization upfront**
   - Request: `{"identity": "user@example.com", "password": "password", "organization_id": "uuid"}`
   - Response: Direct authentication with JWT token (same as scenario 2)

**Select Organization** - `POST /api/auth/select-org`
- Complete multi-org authentication flow
- Request: `{"follow_on_token": "token", "organization_id": "uuid"}`
- Response: Final JWT token with organization-specific access
- Follow-on tokens expire after 5 minutes

### Example API Usage

#### Organization-Scoped Operations

```bash
# Get Coke organization ID from sample data
ORG_ID=$(docker exec vostuff-postgres psql -U vostuff -d vostuff_dev -t -c "SELECT id FROM organizations WHERE name='Coke'")

# List items for Coke organization
curl "http://localhost:8080/api/organizations/${ORG_ID}/items?page=1&per_page=10"

# Create a new item
curl -X POST "http://localhost:8080/api/organizations/${ORG_ID}/items" \
  -H "Content-Type: application/json" \
  -d '{
    "item_type": "vinyl",
    "name": "Dark Side of the Moon",
    "description": "Pink Floyd classic album"
  }'
```

#### Admin Operations

```bash
# List all organizations
curl "http://localhost:8080/api/admin/organizations"

# Create a new organization
curl -X POST "http://localhost:8080/api/admin/organizations" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New Org",
    "description": "A new organization"
  }'

# List all users
curl "http://localhost:8080/api/admin/users"

# Create a new user with password
curl -X POST "http://localhost:8080/api/admin/users" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Jane Doe",
    "identity": "jane@example.com",
    "password": "secure_password_123"
  }'

# Add user to organization with roles
USER_ID=$(docker exec vostuff-postgres psql -U vostuff -d vostuff_dev -t -c "SELECT id FROM users WHERE name='Jane Doe'")
curl -X POST "http://localhost:8080/api/admin/users/${USER_ID}/organizations/${ORG_ID}" \
  -H "Content-Type: application/json" \
  -d '{
    "roles": ["USER", "ADMIN"]
  }'

# Update user's roles in organization
curl -X PATCH "http://localhost:8080/api/admin/users/${USER_ID}/organizations/${ORG_ID}" \
  -H "Content-Type: application/json" \
  -d '{
    "roles": ["USER", "ADMIN", "OWNER"]
  }'

# List organizations for a user (shows roles for each org)
curl "http://localhost:8080/api/admin/users/${USER_ID}/organizations"
```

#### Authentication Operations

```bash
# Login - Single organization (auto-selected)
curl -X POST "http://localhost:8080/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "identity": "jane@example.com",
    "password": "secure_password_123"
  }'
# Returns: {"token": "jwt_token", "expires_in": 86400, "user": {...}}

# Login - Multi-organization (requires org selection)
curl -X POST "http://localhost:8080/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "identity": "multiorg@example.com",
    "password": "password"
  }'
# Returns: {"organizations": [...], "follow_on_token": "temp_token"}

# Complete multi-org authentication by selecting organization
curl -X POST "http://localhost:8080/api/auth/select-org" \
  -H "Content-Type: application/json" \
  -d '{
    "follow_on_token": "temp_token_from_previous_response",
    "organization_id": "org_uuid_to_select"
  }'
# Returns: {"token": "jwt_token", "expires_in": 86400, "user": {...}}

# Login - Direct org specification (bypasses selection)
curl -X POST "http://localhost:8080/api/auth/login" \
  -H "Content-Type: application/json" \
  -d '{
    "identity": "multiorg@example.com",
    "password": "password",
    "organization_id": "specific_org_uuid"
  }'
# Returns: {"token": "jwt_token", "expires_in": 86400, "user": {...}}
```

### OpenAPI Documentation

Visit `http://localhost:8080/swagger-ui` for:
- Interactive API exploration
- Request/response schemas
- Try out API calls directly from the browser
- Download OpenAPI specification

## Integration Tests

The project includes comprehensive integration tests that exercise all API endpoints. The tests:

- **Automatically set up and tear down test databases** for each test run
- **Load sample data** using the same utilities as the load-sample-data binary
- **Test all CRUD operations** for items, locations, collections, and tags
- **Verify pagination** works correctly
- **Test error cases** (404s, invalid requests)
- **Verify multi-tenant isolation** (organizations cannot access each other's data)

### Running Integration Tests

```bash
# Run all integration tests
cargo test --test api_tests

# Run with proper isolation (recommended)
cargo test --test api_tests -- --test-threads=1

# Run a specific test
cargo test --test api_tests test_list_items
```

### Test Coverage

The integration tests cover:
- 41 comprehensive test cases
- Items: List (with pagination), Get, Create, Update, Delete
- Locations: List, Create, Delete
- Collections: List, Create, Delete
- Tags: List, Create, Delete
- Organizations (Admin): List, Get, Create, Update, Delete, List Users
- Users (Admin): List, Create, Update, Delete
- User-Organization Memberships (Admin): List, Add, Remove, Update Roles
- Authentication: Login (single-org, multi-org, with org_id), Org Selection, Invalid Tokens
- Multi-tenant isolation verification
- Error handling (404 responses, 409 conflicts, 401 unauthorized)

### Sample Data Utilities

The sample data loading functionality is now shared between:
- `cargo run --bin load-sample-data` - CLI tool for loading data into a development database
- Integration tests - Automatically loads clean data for each test run

Both use the same `SampleDataLoader` from `src/test_utils.rs`, ensuring consistency.

## Contributing

This project follows specific development workflows documented in `CLAUDE.md`. Key points:

- Use TodoWrite tool for multi-step tasks
- Update JOURNAL.md after significant work
- Run tests and quality checks before committing
- Follow existing code conventions

## Architecture

VOStuff is designed as a three-tier application:

1. **Database Layer** (PostgreSQL) - ✅ Complete
2. **API Layer** (Axum REST API) - ✅ Complete
3. **UI Layer** (Leptos SSR) - ✅ Complete

### Multi-Tenancy

Organizations provide hard isolation:
- All data is scoped to an organization
- Users can belong to multiple organizations
- SYSTEM organization manages platform-level data
- No cross-organization data access

## License

[To be determined]

## Support

For issues or questions, please check:
- `CLAUDE.md` for development guidelines
- `requirements/functional.md` for detailed requirements
- `JOURNAL.md` for implementation history
