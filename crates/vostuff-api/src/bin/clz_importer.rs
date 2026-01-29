//! CLZ CSV Importer - imports movies/DVDs from CLZ export files into vostuff
//!
//! This tool reads CSV files exported from CLZ applications and creates items
//! in vostuff via the REST API.

use anyhow::{Context, Result, bail};
use chrono::NaiveDate;
use clap::Parser;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use uuid::Uuid;

/// CLZ CSV Importer - Import movies/DVDs from CLZ export files into vostuff
#[derive(Parser, Debug)]
#[command(name = "clz-importer")]
#[command(about = "Import CLZ CSV exports into vostuff")]
struct Args {
    /// User email for authentication
    #[arg(short, long)]
    username: String,

    /// Password (optional, uses VOSTUFF_PASSWORD env var or interactive prompt)
    #[arg(short, long)]
    password: Option<String>,

    /// Organization ID (optional, will prompt if user has multiple orgs)
    #[arg(short, long)]
    org_id: Option<Uuid>,

    /// API base URL
    #[arg(long, default_value = "http://localhost:8080")]
    api_url: String,

    /// Parse and validate without creating items
    #[arg(long)]
    dry_run: bool,

    /// CSV file to import
    csv_file: PathBuf,
}

/// CSV record from CLZ export
#[derive(Debug, Deserialize)]
struct ClzRecord {
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "Release Date")]
    release_date: Option<String>,
    #[serde(rename = "Genres")]
    genres: Option<String>,
    #[serde(rename = "Runtime")]
    runtime: Option<String>,
    #[serde(rename = "Director")]
    director: Option<String>,
    #[serde(rename = "Format")]
    format: Option<String>,
    #[serde(rename = "Distributor")]
    distributor: Option<String>,
    #[serde(rename = "Added Date")]
    added_date: Option<String>,
}

/// Login request
#[derive(Serialize)]
struct LoginRequest {
    identity: String,
    password: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    organization_id: Option<Uuid>,
}

/// Login response (successful)
#[derive(Deserialize)]
struct LoginResponse {
    token: String,
}

/// Organization selection response (multi-org user)
#[derive(Deserialize)]
struct OrgSelectionResponse {
    organizations: Vec<OrganizationInfo>,
    follow_on_token: String,
}

#[derive(Deserialize)]
struct OrganizationInfo {
    id: Uuid,
    name: String,
}

/// Select org request
#[derive(Serialize)]
struct SelectOrgRequest {
    follow_on_token: String,
    organization_id: Uuid,
}

/// Create item request
#[derive(Serialize)]
struct CreateItemRequest {
    item_type: String,
    name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    notes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    date_acquired: Option<NaiveDate>,
}

/// API error response
#[derive(Deserialize)]
struct ErrorResponse {
    #[allow(dead_code)]
    error: String,
    message: String,
}

/// Import statistics
#[derive(Default)]
struct ImportStats {
    total: usize,
    imported: usize,
    skipped: usize,
    failed: usize,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Read and parse CSV
    println!("Reading CSV file: {}", args.csv_file.display());
    let records = read_csv(&args.csv_file)?;
    println!("Found {} records", records.len());

    if args.dry_run {
        println!("\n=== DRY RUN MODE ===");
        println!("Validating records without creating items...\n");
        validate_records(&records);
        return Ok(());
    }

    // Get password (only needed for actual import)
    let password = get_password(&args)?;

    // Create HTTP client
    let client = Client::new();

    // Authenticate
    println!("\nAuthenticating as {}...", args.username);
    let (token, org_id) = authenticate(
        &client,
        &args.api_url,
        &args.username,
        &password,
        args.org_id,
    )
    .await?;
    println!("Authentication successful!");

    // Import items
    println!("\nImporting items...\n");
    let stats = import_items(&client, &args.api_url, &token, org_id, &records).await?;

    // Print summary
    println!("\n=== Import Summary ===");
    println!("Total records: {}", stats.total);
    println!("Imported:      {}", stats.imported);
    println!("Skipped:       {}", stats.skipped);
    println!("Failed:        {}", stats.failed);

    Ok(())
}

/// Get password from argument, environment variable, or interactive prompt
fn get_password(args: &Args) -> Result<String> {
    if let Some(password) = &args.password {
        return Ok(password.clone());
    }

    if let Ok(password) = env::var("VOSTUFF_PASSWORD") {
        return Ok(password);
    }

    print!("Password: ");
    io::stdout().flush()?;
    let password = rpassword::read_password()?;
    Ok(password)
}

/// Read and parse CSV file
fn read_csv(path: &PathBuf) -> Result<Vec<ClzRecord>> {
    let mut reader = csv::Reader::from_path(path)
        .with_context(|| format!("Failed to open CSV file: {}", path.display()))?;

    let mut records = Vec::new();
    for (line_num, result) in reader.deserialize().enumerate() {
        match result {
            Ok(record) => records.push(record),
            Err(e) => {
                eprintln!("Warning: Skipping line {}: {}", line_num + 2, e);
            }
        }
    }

    Ok(records)
}

/// Validate records without creating items (dry run mode)
fn validate_records(records: &[ClzRecord]) {
    let mut valid = 0;
    let mut invalid = 0;

    for (i, record) in records.iter().enumerate() {
        let issues = validate_record(record);
        if issues.is_empty() {
            valid += 1;
        } else {
            invalid += 1;
            println!("Record {}: \"{}\"", i + 1, record.title);
            for issue in issues {
                println!("  - {}", issue);
            }
        }
    }

    println!("\nValidation complete:");
    println!("  Valid:   {}", valid);
    println!("  Invalid: {}", invalid);
}

/// Validate a single record, returning any issues found
fn validate_record(record: &ClzRecord) -> Vec<String> {
    let mut issues = Vec::new();

    if record.title.trim().is_empty() {
        issues.push("Empty title".to_string());
    }

    if let Some(date) = &record.added_date
        && !date.is_empty()
        && parse_clz_date(date).is_none()
    {
        issues.push(format!("Invalid added date format: {}", date));
    }

    issues
}

/// Parse CLZ date format (e.g., "Nov 09, 2022")
fn parse_clz_date(date_str: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date_str.trim(), "%b %d, %Y").ok()
}

/// Authenticate with the API
async fn authenticate(
    client: &Client,
    api_url: &str,
    username: &str,
    password: &str,
    org_id: Option<Uuid>,
) -> Result<(String, Uuid)> {
    let login_req = LoginRequest {
        identity: username.to_string(),
        password: password.to_string(),
        organization_id: org_id,
    };

    let resp = client
        .post(format!("{}/api/auth/login", api_url))
        .json(&login_req)
        .send()
        .await
        .context("Failed to connect to API server")?;

    let status = resp.status();
    let body = resp.text().await?;

    if !status.is_success() {
        let error: ErrorResponse = serde_json::from_str(&body).unwrap_or_else(|_| ErrorResponse {
            error: "unknown".to_string(),
            message: body.clone(),
        });
        bail!("Authentication failed: {}", error.message);
    }

    // Try to parse as LoginResponse first (single org or org_id provided)
    if let Ok(login_resp) = serde_json::from_str::<LoginResponse>(&body) {
        // Extract org_id from token claims (we need to get it from the response)
        // For now, we need the org_id to be provided if not in the response
        if let Some(org_id) = org_id {
            return Ok((login_resp.token, org_id));
        }
        // If org_id wasn't provided but we got a token, the user has only one org
        // We need to parse the response differently
        #[derive(Deserialize)]
        struct FullLoginResponse {
            token: String,
            user: UserInfo,
        }
        #[derive(Deserialize)]
        struct UserInfo {
            organization: OrgInfo,
        }
        #[derive(Deserialize)]
        struct OrgInfo {
            id: Uuid,
        }

        let full_resp: FullLoginResponse =
            serde_json::from_str(&body).context("Failed to parse login response")?;
        return Ok((full_resp.token, full_resp.user.organization.id));
    }

    // Parse as org selection response (multi-org user)
    let org_selection: OrgSelectionResponse =
        serde_json::from_str(&body).context("Failed to parse org selection response")?;

    println!("\nUser belongs to multiple organizations:");
    for (i, org) in org_selection.organizations.iter().enumerate() {
        println!("  {}. {} ({})", i + 1, org.name, org.id);
    }

    // Prompt for selection
    print!(
        "\nSelect organization (1-{}): ",
        org_selection.organizations.len()
    );
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let selection: usize = input.trim().parse().context("Invalid selection")?;

    if selection < 1 || selection > org_selection.organizations.len() {
        bail!("Invalid selection: {}", selection);
    }

    let selected_org = &org_selection.organizations[selection - 1];
    println!("Selected: {}", selected_org.name);

    // Call select-org endpoint
    let select_req = SelectOrgRequest {
        follow_on_token: org_selection.follow_on_token,
        organization_id: selected_org.id,
    };

    let resp = client
        .post(format!("{}/api/auth/select-org", api_url))
        .json(&select_req)
        .send()
        .await
        .context("Failed to select organization")?;

    let status = resp.status();
    let body = resp.text().await?;

    if !status.is_success() {
        let error: ErrorResponse = serde_json::from_str(&body).unwrap_or_else(|_| ErrorResponse {
            error: "unknown".to_string(),
            message: body.clone(),
        });
        bail!("Organization selection failed: {}", error.message);
    }

    let login_resp: LoginResponse = serde_json::from_str(&body)
        .context("Failed to parse login response after org selection")?;

    Ok((login_resp.token, selected_org.id))
}

/// Import items into vostuff
async fn import_items(
    client: &Client,
    api_url: &str,
    token: &str,
    org_id: Uuid,
    records: &[ClzRecord],
) -> Result<ImportStats> {
    let mut stats = ImportStats {
        total: records.len(),
        ..Default::default()
    };

    for (i, record) in records.iter().enumerate() {
        // Skip records with empty titles
        if record.title.trim().is_empty() {
            println!("[{}/{}] Skipped: empty title", i + 1, records.len());
            stats.skipped += 1;
            continue;
        }

        // Build notes from metadata
        let notes = build_notes(record);

        // Parse date
        let date_acquired = record.added_date.as_ref().and_then(|d| parse_clz_date(d));

        // Create item request
        let create_req = CreateItemRequest {
            item_type: "dvd".to_string(),
            name: record.title.clone(),
            notes,
            date_acquired,
        };

        // Send request
        let resp = client
            .post(format!("{}/api/organizations/{}/items", api_url, org_id))
            .header("Authorization", format!("Bearer {}", token))
            .json(&create_req)
            .send()
            .await;

        match resp {
            Ok(response) => {
                if response.status().is_success() {
                    println!("[{}/{}] Imported: {}", i + 1, records.len(), record.title);
                    stats.imported += 1;
                } else {
                    let error_body = response.text().await.unwrap_or_default();
                    let error: ErrorResponse =
                        serde_json::from_str(&error_body).unwrap_or_else(|_| ErrorResponse {
                            error: "unknown".to_string(),
                            message: error_body,
                        });
                    eprintln!(
                        "[{}/{}] Failed: {} - {}",
                        i + 1,
                        records.len(),
                        record.title,
                        error.message
                    );
                    stats.failed += 1;
                }
            }
            Err(e) => {
                eprintln!(
                    "[{}/{}] Failed: {} - {}",
                    i + 1,
                    records.len(),
                    record.title,
                    e
                );
                stats.failed += 1;
            }
        }
    }

    Ok(stats)
}

/// Build notes field from CLZ record metadata
fn build_notes(record: &ClzRecord) -> Option<String> {
    let mut parts = Vec::new();

    // Helper to add non-empty optional fields
    let mut add_field = |label: &str, value: &Option<String>| {
        if let Some(v) = value
            && !v.is_empty()
        {
            parts.push(format!("- **{}:** {}", label, v));
        }
    };

    add_field("Format", &record.format);
    add_field("Release Date", &record.release_date);
    add_field("Director", &record.director);
    add_field("Runtime", &record.runtime);
    add_field("Genres", &record.genres);
    add_field("Distributor", &record.distributor);

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("\n"))
    }
}
