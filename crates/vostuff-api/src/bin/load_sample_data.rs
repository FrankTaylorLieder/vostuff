use anyhow::Result;
use sqlx::PgPool;
use std::env;
use vostuff_api::test_utils::SampleDataLoader;

#[tokio::main]
async fn main() -> Result<()> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgresql://vostuff:vostuff_dev_password@localhost:5432/vostuff_dev".to_string()
    });

    println!("Connecting to database: {}", database_url);
    let pool = PgPool::connect(&database_url).await?;

    let loader = SampleDataLoader::new(&pool);
    loader.load_sample_data().await?;

    pool.close().await;
    Ok(())
}
