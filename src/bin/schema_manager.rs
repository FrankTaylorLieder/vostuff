use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env;

use vostuff::schema::SchemaManager;

#[derive(Parser)]
#[command(name = "schema-manager")]
#[command(about = "VOStuff database schema management tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Run pending migrations")]
    Migrate {
        #[arg(long, env = "DATABASE_URL")]
        database_url: Option<String>,
    },
    #[command(about = "Reset database (drop all tables and re-run migrations)")]
    Reset {
        #[arg(long, env = "DATABASE_URL")]
        database_url: Option<String>,
    },
    #[command(about = "Create database if it doesn't exist")]
    Create {
        #[arg(long, env = "DATABASE_URL")]
        database_url: Option<String>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    let default_db_url = "postgresql://localhost/vostuff_dev".to_string();

    match cli.command {
        Commands::Migrate { database_url } => {
            let db_url = database_url
                .or_else(|| env::var("DATABASE_URL").ok())
                .unwrap_or(default_db_url);
            
            println!("Running migrations on: {}", db_url);
            let schema_manager = SchemaManager::new(&db_url).await?;
            schema_manager.run_migrations().await?;
            println!("Migrations completed successfully!");
            schema_manager.close().await;
        }
        Commands::Reset { database_url } => {
            let db_url = database_url
                .or_else(|| env::var("DATABASE_URL").ok())
                .unwrap_or(default_db_url);
            
            println!("Resetting database: {}", db_url);
            println!("WARNING: This will delete all data!");
            
            let schema_manager = SchemaManager::new(&db_url).await?;
            schema_manager.reset_database().await?;
            println!("Database reset completed successfully!");
            schema_manager.close().await;
        }
        Commands::Create { database_url } => {
            let db_url = database_url
                .or_else(|| env::var("DATABASE_URL").ok())
                .unwrap_or(default_db_url);
            
            println!("Creating database if needed: {}", db_url);
            let schema_manager = SchemaManager::new(&db_url).await?;
            println!("Database ready!");
            schema_manager.close().await;
        }
    }

    Ok(())
}