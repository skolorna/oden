use anyhow::Context;
use axum::{
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use meilisearch_sdk::key::Action;
use opentelemetry::{
    sdk::{propagation::TraceContextPropagator, trace, Resource},
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use serde::{Deserialize, Serialize};
use sqlx::{
    postgres::{types::PgRange, PgPoolOptions},
    PgPool,
};
use std::{env, net::SocketAddr};
use stor::Menu;
use time::Date;
use tower_http::cors::CorsLayer;
use tracing::warn;
use tracing_subscriber::{
    filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    if let Ok(otlp_endpoint) = env::var("OTLP_ENDPOINT") {
        init_telemetry(otlp_endpoint)?;
    } else {
        warn!("OTLP_ENDPOINT not set");
    }

    let pg = PgPoolOptions::new()
        .connect(&env::var("DATABASE_URL")?)
        .await
        .context("could not connect to database")?;

    let state = AppState {
        pg,
        meili: meilisearch_sdk::Client::new(env::var("MEILI_URL")?, env::var("MEILI_KEY")?),
    };

    let app = Router::new()
        .route("/stats", get(stats))
        .route("/key", get(meilisearch_key))
        .route("/menus/:id", get(menu))
        .route("/menus/:id/days", get(days))
        .layer(opentelemetry_tracing_layer())
        .route("/health", get(health))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn init_telemetry(otlp_endpoint: impl Into<String>) -> anyhow::Result<()> {
    opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(otlp_endpoint);

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(trace::config().with_resource(Resource::new(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                env!("CARGO_PKG_NAME"),
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
                env!("CARGO_PKG_VERSION"),
            ),
        ])))
        .install_batch(opentelemetry::runtime::Tokio)?;

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(otel_layer)
        .init();

    Ok(())
}

#[derive(Clone)]
struct AppState {
    pg: PgPool,
    meili: meilisearch_sdk::Client,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pg.clone()
    }
}

impl FromRef<AppState> for meilisearch_sdk::Client {
    fn from_ref(state: &AppState) -> Self {
        state.meili.clone()
    }
}

#[derive(Debug, Serialize)]
struct Health {
    version: &'static str,
    db_connections: u32,
}

async fn health(State(db): State<PgPool>) -> impl IntoResponse {
    (
        [("cache-control", "no-cache")],
        Json(Health {
            version: env!("CARGO_PKG_VERSION"),
            db_connections: db.size(),
        }),
    )
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

async fn menu(State(db): State<PgPool>, Path(id): Path<Uuid>) -> Result<impl IntoResponse> {
    let menu = sqlx::query_as::<_, Menu>("SELECT * FROM menus WHERE id = $1")
        .bind(id)
        .fetch_optional(&db)
        .await?
        .ok_or(Error::MenuNotFound)?;

    Ok(([("cache-control", "public, max-age=60")], Json(menu)))
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
) -> Result<impl IntoResponse> {
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

    Ok(([("cache-control", "public, max-age=60")], Json(days)))
}

async fn meilisearch_key(State(client): State<meilisearch_sdk::Client>) -> Result<Response> {
    Ok(
        if let Some(key) = client
            .get_keys()
            .await?
            .results
            .into_iter()
            .find(|k| k.actions == vec![Action::Search])
        {
            ([("cache-control", "public, max-age=300")], key.key).into_response()
        } else {
            StatusCode::NOT_FOUND.into_response()
        },
    )
}

type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("database error")]
    Db(#[from] sqlx::Error),

    #[error("meilisearch error")]
    MeiliSearch(#[from] meilisearch_sdk::errors::Error),

    #[error("internal server error")]
    Internal,

    #[error("menu not found")]
    MenuNotFound,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Db(_) | Self::Internal | Self::MeiliSearch(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "internal server error").into_response()
            }
            Self::MenuNotFound => (StatusCode::NOT_FOUND, "menu not found").into_response(),
        }
    }
}
