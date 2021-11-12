pub mod models;
pub mod schema;
pub mod types;

#[macro_use]
extern crate diesel;

#[cfg(feature = "meilisearch-sdk")]
pub trait MeiliIndexable {
    const MEILI_INDEX: &'static str;
}
