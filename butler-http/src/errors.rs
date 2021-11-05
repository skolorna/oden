use actix_web::{http::StatusCode, ResponseError};
use butler_lib::errors::ButlerError;
use thiserror::Error;

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

impl From<ButlerError> for AppError {
    fn from(e: ButlerError) -> Self {
        match e {
            ButlerError::MenuNotFound => Self::MenuNotFound,
            ButlerError::HttpError(_) => Self::InternalError,
            ButlerError::ScrapeError { .. } => Self::InternalError,
            ButlerError::InvalidMenuId => Self::BadRequest("invalid menu id".into()),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;
