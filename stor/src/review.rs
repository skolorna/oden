use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Review {
    pub id: Uuid,
    pub author: Uuid,
    pub menu_id: Uuid,
    pub date: Date,
    pub meal: String,
    pub rating: i32,
    pub comment: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub edited_at: Option<OffsetDateTime>,
}
