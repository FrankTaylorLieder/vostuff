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

### Planned
- REST API (Axum)
- Web UI (Leptos with SSR)
- OIDC Authentication
- Session Management

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

### 5. Verify Setup

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
```

### Testing and Quality

```bash
# Run all tests
cargo test

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

## Contributing

This project follows specific development workflows documented in `CLAUDE.md`. Key points:

- Use TodoWrite tool for multi-step tasks
- Update JOURNAL.md after significant work
- Run tests and quality checks before committing
- Follow existing code conventions

## Architecture

VOStuff is designed as a three-tier application:

1. **Database Layer** (PostgreSQL) - ✅ Complete
2. **API Layer** (Axum REST API) - 🚧 Planned
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
