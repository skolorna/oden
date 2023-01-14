use anyhow::Context;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{types::PgRange, PgPoolOptions},
    PgPool,
};
use std::{env, net::SocketAddr};
use stor::{Day, Menu};
use time::Date;
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    let database_url = env::var("DATABASE_URL")?;

    let db = PgPoolOptions::new()
        .connect(&database_url)
        .await
        .context("could not connect to database")?;

    let app = Router::new()
        .route("/health", get(health))
        .route("/stats", get(stats))
        .route("/menus/:id", get(menu))
        .route("/menus/:id/days", get(days))
        .with_state(db);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Debug, Serialize)]
struct Health {
    version: &'static str,
    db_connections: u32,
}

async fn health(State(db): State<PgPool>) -> impl IntoResponse {
    Json(Health {
        version: env!("CARGO_PKG_VERSION"),
        db_connections: db.size(),
    })
}

#[derive(Debug, Serialize)]
struct Stats {
    menus: i64,
    days: i64,
}

async fn stats(State(db): State<PgPool>) -> Result<impl IntoResponse> {
    let stats = Stats {
        menus: sqlx::query!("SELECT COUNT(*) FROM menus")
            .fetch_one(&db)
            .await?
            .count
            .ok_or(Error::Internal)?,
        days: sqlx::query!("SELECT COUNT(*) FROM days")
            .fetch_one(&db)
            .await?
            .count
            .ok_or(Error::Internal)?,
    };

    Ok(([("cache-control", "public, max-age=600")], Json(stats)))
}

async fn menu(State(db): State<PgPool>, Path(id): Path<Uuid>) -> Result<Json<Menu>> {
    sqlx::query_as::<_, Menu>("SELECT * FROM menus WHERE id = $1")
        .bind(id)
        .fetch_optional(&db)
        .await?
        .map(Json)
        .ok_or(Error::MenuNotFound)
}

#[derive(Debug, Deserialize)]
struct QueryDays {
    first: Date,
    last: Date,
}

async fn days(
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Query(QueryDays { first, last }): Query<QueryDays>,
) -> Result<Json<Vec<Day>>> {
    let days = sqlx::query_as::<_, stor::Day>(
        r#"
            SELECT * FROM days
            WHERE menu_id = $1
            AND $2 @> date
            ORDER BY date ASC
        "#,
    )
    .bind(id)
    .bind(PgRange::from(first..=last))
    .fetch_all(&db)
    .await?;

    Ok(Json(days))
}

type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("database error")]
    Db(#[from] sqlx::Error),

    #[error("internal server error")]
    Internal,

    #[error("menu not found")]
    MenuNotFound,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Db(_) | Self::Internal => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
            Self::MenuNotFound => (StatusCode::NOT_FOUND, "menu not found").into_response(),
        }
    }
}
