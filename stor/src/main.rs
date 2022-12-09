use std::str::FromStr;

use sqlx::{sqlite::SqliteConnectOptions, SqlitePool};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let options = SqliteConnectOptions::from_str("sqlite://data.db")?.create_if_missing(true);
    let pool = SqlitePool::connect_with(options).await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(())
}
