use actix_web::{http::StatusCode, ResponseError};
use munin_lib::errors::MuninError;
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

impl From<MuninError> for AppError {
    fn from(e: MuninError) -> Self {
        match e {
            MuninError::MenuNotFound => Self::MenuNotFound,
            MuninError::HttpError(_) => Self::InternalError,
            MuninError::ScrapeError { .. } => Self::InternalError,
            MuninError::InvalidMenuSlug => Self::BadRequest("invalid menu id".into()),
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

pub type AppResult<T> = Result<T, AppError>;
