use anyhow::Result;
use sqlx::{PgPool, Postgres, migrate::MigrateDatabase};

pub struct SchemaManager {
    pool: PgPool,
}

impl SchemaManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        if !Postgres::database_exists(database_url).await? {
            Postgres::create_database(database_url).await?;
        }

        let pool = PgPool::connect(database_url).await?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("../../migrations").run(&self.pool).await?;
        Ok(())
    }

    pub async fn reset_database(&self) -> Result<()> {
        sqlx::query("DROP SCHEMA public CASCADE")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE SCHEMA public")
            .execute(&self.pool)
            .await?;

        self.run_migrations().await?;
        Ok(())
    }

    pub fn get_pool(&self) -> &PgPool {
        &self.pool
    }

    pub async fn close(self) {
        self.pool.close().await;
    }
}
