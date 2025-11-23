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
  - Comprehensive audit logging
- **Schema Management**: CLI tool and reusable library for database migrations
- **Docker Development Environment**: Containerized PostgreSQL for easy local development
- **Sample Data Generator**: Load realistic test data with 100+ items across two organizations
- **REST API**: Full-featured API built with Axum
  - CRUD endpoints for items, locations, collections, and tags
  - Organization-scoped operations
  - Admin endpoints for managing users and organizations
  - User-organization membership management
  - Pagination support
  - Interactive OpenAPI/Swagger documentation at `/swagger-ui`
  - Type-safe request/response models
  - Comprehensive error handling

### Planned
- Web UI (Leptos with SSR)
- OIDC Authentication
- Session Management and JWT tokens

## Prerequisites

- Rust 1.86.0 or later (edition 2024)
- Docker and Docker Compose
- PostgreSQL client tools (optional, for manual database access)

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
```

### 3. Start the Database

```bash
docker-compose up -d
```

This starts a PostgreSQL 16 container with:
- Database: `vostuff_dev`
- User: `vostuff`
- Password: `vostuff_dev_password`
- Port: `5432`

### 4. Run Database Migrations

```bash
cargo run --bin schema-manager migrate
```

This creates all tables, indexes, triggers, and the initial SYSTEM organization.

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
# Start PostgreSQL
docker-compose up -d

# Stop PostgreSQL
docker-compose down

# Run migrations
cargo run --bin schema-manager migrate

# Reset database (WARNING: deletes all data)
cargo run --bin schema-manager reset

# Create database only (no migrations)
cargo run --bin schema-manager create

# Load sample data for testing
cargo run --bin load-sample-data
```

### API Server

```bash
# Run the REST API server
cargo run --bin api-server

# The server will start on http://localhost:8080
# Swagger UI available at http://localhost:8080/swagger-ui
```

### Testing and Quality

```bash
# Run all tests
cargo test

# Run integration tests only
cargo test --test api_tests

# Run integration tests with proper isolation (one at a time)
cargo test --test api_tests -- --test-threads=1

# Run specific test
cargo test <test_name>

# Check code without building
cargo check

# Run linter
cargo clippy

# Format code
cargo fmt
```

## Project Structure

```
vostuff/
├── migrations/              # SQL migration files (sqlx)
│   └── 20240101000000_initial_schema.sql
├── scripts/                 # Helper scripts
│   └── init-db.sh          # Database initialization
├── src/
│   ├── bin/
│   │   └── schema_manager.rs  # Schema management CLI
│   ├── lib.rs              # Library root
│   ├── main.rs             # Main application
│   └── schema.rs           # Schema management module
├── requirements/
│   └── functional.md       # Functional requirements
├── docker-compose.yml      # PostgreSQL container config
├── CLAUDE.md              # Development guidelines
├── JOURNAL.md             # Development journal
├── TODO.md                # Task tracking
└── README.md              # This file
```

## Database Schema

The PostgreSQL schema implements a multi-tenant system with:

### Core Tables
- **organizations**: Tenant isolation boundary
- **users**: User accounts with OIDC identity
- **user_organizations**: Many-to-many user/org membership

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
- `GET /api/admin/users/{user_id}/organizations` - List organizations for a user
- `POST /api/admin/users/{user_id}/organizations/{org_id}` - Add user to organization
- `DELETE /api/admin/users/{user_id}/organizations/{org_id}` - Remove user from organization

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

# Create a new user
curl -X POST "http://localhost:8080/api/admin/users" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Jane Doe",
    "identity": "jane@example.com"
  }'

# Add user to organization
USER_ID=$(docker exec vostuff-postgres psql -U vostuff -d vostuff_dev -t -c "SELECT id FROM users WHERE name='Jane Doe'")
curl -X POST "http://localhost:8080/api/admin/users/${USER_ID}/organizations/${ORG_ID}"

# List organizations for a user
curl "http://localhost:8080/api/admin/users/${USER_ID}/organizations"
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
- 27 comprehensive test cases
- Items: List (with pagination), Get, Create, Update, Delete
- Locations: List, Create, Delete
- Collections: List, Create, Delete
- Tags: List, Create, Delete
- Organizations (Admin): List, Get, Create, Update, Delete
- Users (Admin): List, Create, Update
- User-Organization Memberships (Admin): List, Add, Remove
- Multi-tenant isolation verification
- Error handling (404 responses, 409 conflicts)

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
3. **UI Layer** (Leptos SSR) - 🚧 Planned

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
