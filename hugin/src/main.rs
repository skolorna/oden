use anyhow::Context;
use auth1_sdk::{Identity, KeyStore};
use axum::{
    extract::{FromRef, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{delete, get},
    Extension, Json, Router,
};
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
use itertools::Itertools;
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
use std::{env, net::SocketAddr, time::Duration};
use stor::Menu;
use time::Date;
use tower_http::cors::CorsLayer;
use tracing::{error, info};
use tracing_subscriber::{
    filter::LevelFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv::dotenv().ok();

    init_telemetry(
        env::var("OTLP_ENDPOINT").unwrap_or_else(|_| "http://localhost:4317".to_owned()),
    )?;

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
        .route("/menus", get(menus))
        .route("/menus/:menu_id", get(menu))
        .route("/menus/:menu_id/days", get(days))
        .route("/reviews", get(list_reviews).post(create_review))
        .route("/reviews/:review_id", delete(delete_review))
        .layer(opentelemetry_tracing_layer())
        .route("/health", get(health))
        .layer(CorsLayer::permissive().max_age(Duration::from_secs(3600)))
        .layer(Extension(KeyStore::default()))
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8000));

    info!("listening on {addr}");

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
    meals: i64,
}

async fn stats(State(db): State<PgPool>) -> Result<impl IntoResponse> {
    let stats = Stats {
        menus: sqlx::query!("SELECT COUNT(*) FROM menus")
            .fetch_one(&db)
            .await?
            .count
            .ok_or(Error::Internal)?,
        meals: sqlx::query!("SELECT COUNT(*) FROM meals")
            .fetch_one(&db)
            .await?
            .count
            .ok_or(Error::Internal)?,
    };

    Ok(([("cache-control", "public, max-age=600")], Json(stats)))
}

async fn menus(State(db): State<PgPool>) -> Result<impl IntoResponse> {
    let menus = sqlx::query_as::<_, Menu>("SELECT * FROM menus")
        .fetch_all(&db)
        .await?;

    Ok(([("cache-control", "public, max-age=60")], Json(menus)))
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

#[derive(Debug, Serialize)]
struct Meal {
    value: String,
    rating: Option<f32>,
    reviews: i64,
}

#[derive(Debug, Serialize)]
struct Day {
    date: Date,
    meals: Vec<Meal>,
}

async fn days(
    State(db): State<PgPool>,
    Path(id): Path<Uuid>,
    Query(QueryDays { first, last }): Query<QueryDays>,
) -> Result<impl IntoResponse> {
    let meals = sqlx::query_file!("queries/meals.sql", id, PgRange::from(first..=last))
        .fetch_all(&db)
        .await?;

    let days: Vec<Day> = meals
        .into_iter()
        .group_by(|m| m.date)
        .into_iter()
        .map(|(date, meals)| Day {
            date,
            meals: meals
                .map(|m| Meal {
                    value: m.meal,
                    rating: m.rating,
                    reviews: m.reviews.unwrap_or_default(),
                })
                .collect(),
        })
        .collect();

    Ok(([("cache-control", "no-cache")], Json(days)))
}

#[derive(Debug, Deserialize)]
struct ReviewQuery {
    menu: Option<Uuid>,
    meal: Option<String>,
    date: Option<Date>,
}

async fn list_reviews(
    State(db): State<PgPool>,
    Query(ReviewQuery { menu, meal, date }): Query<ReviewQuery>,
) -> Result<impl IntoResponse> {
    let reviews = sqlx::query_as::<_, stor::Review>(
        r#"
            SELECT * FROM reviews WHERE
                ($1 IS NULL or menu_id = $1) AND
                ($2 IS NULL or meal = $2) AND
                ($3 IS NULL or date = $3)
        "#,
    )
    .bind(menu)
    .bind(meal)
    .bind(date)
    .fetch_all(&db)
    .await?;

    Ok(([("cache-control", "no-cache")], Json(reviews)))
}

#[derive(Debug, Deserialize)]
struct CreateReview {
    menu_id: Uuid,
    date: Date,
    meal: String,
    rating: i32,
    comment: Option<String>,
}

async fn create_review(
    State(db): State<PgPool>,
    identity: Identity,
    Json(review): Json<CreateReview>,
) -> Result<impl IntoResponse> {
    let CreateReview {
        menu_id,
        date,
        meal,
        rating,
        comment,
    } = review;

    if let Some(ref comment) = comment {
        if comment.len() > 4096 {
            return Err(Error::CommentTooLong);
        }
    }

    let id = Uuid::new_v4();

    let review = sqlx::query_as::<_, stor::Review>(
        r#"
            INSERT INTO reviews (
                id,
                author,
                menu_id,
                date,
                meal,
                rating,
                comment
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
        "#,
    )
    .bind(id)
    .bind(identity.claims.sub)
    .bind(menu_id)
    .bind(date)
    .bind(meal)
    .bind(rating)
    .bind(comment)
    .fetch_one(&db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(dbe)
            if dbe.constraint() == Some("reviews_author_menu_id_date_meal_key") =>
        {
            Error::ReviewExists
        }
        e => e.into(),
    })?;

    Ok((StatusCode::CREATED, Json(review)))
}

async fn delete_review(
    State(db): State<PgPool>,
    identity: Identity,
    Path(review_id): Path<Uuid>,
) -> Result<impl IntoResponse> {
    let res = sqlx::query!(
        "DELETE FROM reviews WHERE id = $1 AND author = $2",
        review_id,
        identity.claims.sub
    )
    .execute(&db)
    .await?;

    if res.rows_affected() == 0 {
        Err(Error::ReviewNotFound)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
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

    #[error("review not found")]
    ReviewNotFound,

    #[error("review already exists")]
    ReviewExists,

    #[error("comment too long")]
    CommentTooLong,
}

impl Error {
    fn status_code(&self) -> StatusCode {
        match self {
            Error::MenuNotFound | Error::ReviewNotFound => StatusCode::NOT_FOUND,
            Error::ReviewExists => StatusCode::CONFLICT,
            Error::CommentTooLong => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        if status.is_server_error() {
            error!("response error: {self:?}");
        }
        (status, self.to_string()).into_response()
    }
}
