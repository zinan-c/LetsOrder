use anyhow::Context;
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

pub type DbPool = SqlitePool;

pub async fn connect(database_url: &str) -> anyhow::Result<DbPool> {
    SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .with_context(|| format!("failed to connect to database at {database_url}"))
}
