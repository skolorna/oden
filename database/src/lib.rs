use diesel_migrations::MigrationConnection;
use tracing::info;

pub mod models;
pub mod schema;
pub mod smaztext;

#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

embed_migrations!();

pub fn run_migrations(
    connection: &impl MigrationConnection,
) -> Result<(), diesel_migrations::RunMigrationsError> {
    info!("Running migrations");
    embedded_migrations::run(connection)
}

#[cfg(feature = "meilisearch-sdk")]
pub trait MeiliIndexable {
    const MEILI_INDEX: &'static str;
}
