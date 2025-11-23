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

The three tiers are contained in this one repository with the REST API and Web
tier being two separate Rust binary applications. Any DB initialisation, data
import and migration tools must be written in Rust and will be separate
binaries.

## Development Workflow

Follow this checklist for every Claude Code session:

### During Development
- [ ] Use TodoWrite tool to track multi-step tasks
- [ ] Follow existing code conventions and patterns
- [ ] Run tests after making changes: `cargo test`
- [ ] Check code quality: `cargo clippy` and `cargo fmt`
- [ ] Update TODO.md with any new identified work

### After Significant Work
- [ ] Mark completed todos as done in TODO.md
- [ ] Update JOURNAL.md proactively when substantial work is completed containing:
    - Date and time
    - The prompt(s) - in full
    - Summary of Claude's response and work completed

## Standing instructions

These instructions should be followed with every interaction:

- In order to keep a complete record of development of this project, all interactions
through Claude Code should be recorded in a file called `JOURNAL.md` in the top
level of the repository:
    - Each entry should consist of:
        - A date and time
        - The prompt - in full
        - A summary of Claude's response
- You should maintain a todo list of identified work that has not been
completed in the top level `TODO.md` file, marking items as done when they are
completed.


