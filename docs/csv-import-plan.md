# Plan: CSV Import CLI Tool

## Overview
Create a command-line tool to import CSV data (movies/DVDs) into vostuff using the REST API.

## CSV File Details
- Location: `data/CLZMovies20241015.csv`
- Columns: Title, Release Date, Genres, Runtime, Director, Format, Distributor, Added Date
- All entries to be treated as DVDs

## Field Mapping

| CSV Field      | Item Field    | Notes                                    |
|---------------|---------------|------------------------------------------|
| Title         | name          | Direct mapping                           |
| (all DVDs)    | item_type     | Use "dvd" item type                      |
| Added Date    | date_acquired | Parse as NaiveDate (YYYY-MM-DD format)   |
| Release Date  | notes         | Include in formatted notes               |
| Genres        | notes         | Include in formatted notes               |
| Runtime       | notes         | Include in formatted notes               |
| Director      | notes         | Include in formatted notes               |
| Format        | notes         | Include in formatted notes (DVD/Blu-ray) |
| Distributor   | notes         | Include in formatted notes               |

## Notes Field Format
```
Format: DVD
Release Date: 2001-09-11
Director: Peter Jackson
Runtime: 178
Genres: Action, Adventure, Fantasy
Distributor: New Line Cinema
```

## CLI Interface

```
csv-import --username <email> [--password <password>] [--org-id <uuid>] <csv-file>

Options:
  --username, -u    User email for authentication (required)
  --password, -p    Password (optional, uses VOSTUFF_PASSWORD env var or interactive prompt)
  --org-id, -o      Organization ID (optional, will prompt if user has multiple orgs)
  --dry-run         Parse and validate without creating items
  --help, -h        Show help
```

## Files to Create

### 1. `crates/vostuff-api/src/bin/csv-import.rs`
Main CLI binary:
- Parse command-line arguments with `clap`
- Read password from env var `VOSTUFF_PASSWORD` or prompt interactively with `rpassword`
- Parse CSV file with `csv` crate
- Authenticate via API (`POST /api/auth/login`)
- Get user's organizations if needed
- Create items via API (`POST /api/organizations/{org_id}/items`)
- Report progress and any errors

## Dependencies to Add

In `crates/vostuff-api/Cargo.toml`:
```toml
clap = { version = "4", features = ["derive"] }
csv = "1.3"
rpassword = "7"
```

Note: `reqwest`, `serde`, `tokio`, `chrono` are already available in the workspace.

## Implementation Steps

1. Add dependencies to `crates/vostuff-api/Cargo.toml`
2. Create `csv-import.rs` binary with:
   - CLI argument parsing
   - Password handling (env var → interactive prompt)
   - CSV parsing and validation
   - API client for login and item creation
   - Progress reporting
   - Error handling with summary at end

## Authentication Flow

1. Call `POST /api/auth/login` with username/password
2. Extract JWT token from response
3. If user has multiple organizations and no --org-id provided, list them and prompt
4. Use token in `Authorization: Bearer <token>` header for subsequent requests

## Error Handling

- Invalid CSV format: Report line number and skip row
- API errors: Report item name and error, continue with next
- Authentication failure: Exit with clear message
- At end: Summary of imported/skipped/failed counts

## Verification

1. Start database: `docker-compose -f docker-compose-dev.yml up -d`
2. Start API server: `cargo run --bin api-server`
3. Run import: `cargo run --bin csv-import -- --username alice@pepsi.com data/CLZMovies20241015.csv`
4. Verify items appear in web UI with filters set to "dvd" type
