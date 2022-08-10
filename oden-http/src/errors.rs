use actix_web::{http::StatusCode, ResponseError};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("an internal error occurred")]
    InternalError,

    #[error("menu not found")]
    MenuNotFound,

    #[error("{0}")]
    BadRequest(String),
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::MenuNotFound => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
        }
    }
}

impl From<hugin::Error> for AppError {
    fn from(e: hugin::Error) -> Self {
        match e {
            hugin::Error::MenuNotFound => Self::MenuNotFound,
            hugin::Error::HttpError(_) => Self::InternalError,
            hugin::Error::ScrapeError { .. } => Self::InternalError,
            hugin::Error::InvalidMenuSlug => Self::BadRequest("invalid menu id".into()),
        }
    }
}

impl From<r2d2::Error> for AppError {
    fn from(e: r2d2::Error) -> Self {
        error!("r2d2 encountered an error: {}", e);
        Self::InternalError
    }
}

impl From<actix_web::error::BlockingError> for AppError {
    fn from(e: actix_web::error::BlockingError) -> Self {
        error!("blocking error: {}", e);
        Self::InternalError
    }
}

impl From<diesel::result::Error> for AppError {
    fn from(e: diesel::result::Error) -> Self {
        error!("diesel error: {}", e);
        Self::InternalError
    }
}

impl From<meilisearch_sdk::errors::Error> for AppError {
    fn from(e: meilisearch_sdk::errors::Error) -> Self {
        error!("meilisearch error: {e}");
        Self::InternalError
    }
}

pub type AppResult<T> = Result<T, AppError>;
