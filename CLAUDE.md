# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with
code in this repository.

## Project Overview

This is a Rust-based three tier application for tracking stuff - vinyl, CDs,
cassettes, books, scores, electronics and other things. It allows the user to
record their stuff, where it is, any loans, any losses and when it is disposed
of.

## Development Commands

### Build and Run
- `cargo build` - Build the project
- `cargo run` - Build and run the application
- `cargo build --release` - Build optimized release version

### Database
- `docker-compose up -d` - Start PostgreSQL database
- `docker-compose down` - Stop PostgreSQL database
- `cargo run --bin schema-manager migrate` - Run database migrations
- `cargo run --bin schema-manager reset` - Reset database (drops all data)

### API Server
- `cargo run --bin api-server` - Run the REST API server (port 8080)
- Swagger UI available at http://localhost:8080/swagger-ui

### Web UI
- `cargo leptos watch` - Run the web UI in development mode with hot reload (port 3001)
- `cargo leptos build --release` - Build the web UI for production
- `cargo leptos serve --release` - Run the production build
- **Note**: The API server must be running for the web UI to function

### Testing and Quality
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run a specific test
- `cargo test --package <package_name>` - Run tests for a specific package
- `cargo check` - Check code without building
- `cargo clippy` - Run linting
- `cargo fmt` - Format code


## Architecture

This is a Rust-based three tier application:

- Database: PostgreSQL
- API: REST-based API built with Axum
- Front end: Web application built with Leptos (using SSR)

The application is multi-tenant from the start, with organisations (orgs) being
the basic tenant. Users, authenticated through OIDC, can belong to orgs and
select which member org they want to work on. All stuff is contained within a
specific org. Orgs are a hard isolation boundary, members of one org cannot see
stuff or users of a another org.


## Project Structure

The three tiers are contained in this one repository organized as a Cargo workspace:

- **crates/vostuff-core**: Shared code (authentication, models, utilities)
- **crates/vostuff-api**: REST API server (Axum, OpenAPI, JWT auth)
- **crates/vostuff-web**: Web UI (Leptos SSR, authentication, server functions)

The REST API and Web tier are two separate Rust binary applications. Any DB initialisation,
data import and migration tools are written in Rust as separate binaries.

### Web UI Architecture
- **Server-side rendering** with Leptos SSR for fast initial page loads
- **Server functions** call the REST API and run only on the server
- **JWT authentication** with tokens stored in HTTP-only cookies
- **Protected routes** with authentication context
- **cargo-leptos** handles WASM compilation, CSS processing, and dev server

## Development Workflow

Follow this checklist for every Claude Code session:

### During Development
- [ ] Use TodoWrite tool to track multi-step tasks
- [ ] Follow existing code conventions and patterns
- [ ] Run tests after making changes: `cargo test`
- [ ] Check code quality: `cargo clippy` and `cargo fmt`
- [ ] Update TODO.md with any new identified work

### After Significant Work (REQUIRED BEFORE FINISHING)
- [ ] Mark completed todos as done in TODO.md
- [ ] **MANDATORY: Update JOURNAL.md proactively when substantial work is completed**
    - Date and time (format: YYYY-MM-DD)
    - The prompt(s) - in full (include all user prompts from the session)
    - Summary of Claude's response and work completed
    - **DO NOT consider a session complete until JOURNAL.md is updated**

## Standing instructions

These instructions should be followed with every interaction:

- **CRITICAL: JOURNAL.md must be updated before ending any session with substantial work**
  - In order to keep a complete record of development of this project, all interactions
    through Claude Code should be recorded in a file called `JOURNAL.md` in the top
    level of the repository
  - Each entry should consist of:
    - Date and time (format: YYYY-MM-DD)
    - The prompt(s) - in full (all user prompts from the session)
    - A summary of Claude's response and work completed
  - **This is not optional - it must be done proactively when you complete substantial work**
  - What counts as "substantial work": new features, bug fixes, refactoring, schema changes,
    configuration updates, or any code changes beyond trivial edits
- You should maintain a todo list of identified work that has not been
  completed in the top level `TODO.md` file, marking items as done when they are
  completed.
- Always update the README when making changes to the API, test tools, deployment process or other user aftecting changes.