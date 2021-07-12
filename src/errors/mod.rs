use std::num::ParseIntError;

use actix_web::{http::StatusCode, ResponseError};
use thiserror::Error;

use crate::menus::id::ParseMenuIDError;

#[derive(Error, Debug)]
pub enum InternalError {
    #[error("http request failed")]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Error, Debug)]
pub enum RangeError {
    #[error("dates out of range")]
    DatesOutOfRange,

    #[error("dates are too far apart")]
    DateSpanTooLong,
}

#[derive(Error, Debug)]
pub enum BadInputError {
    #[error("{0}")]
    ParseIntError(#[from] ParseIntError),

    #[error("{0}")]
    ParseMenuIDError(#[from] ParseMenuIDError),

    #[error("{0}")]
    RangeError(#[from] RangeError),
}

#[derive(Error, Debug)]
pub enum NotFoundError {
    #[error("menu not found")]
    MenuNotFoundError,
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("internal error")]
    InternalError,

    #[error("http request failed: {0}")]
    ReqwestError(#[from] reqwest::Error),

    #[error("{0}")]
    BadInputError(#[from] BadInputError),

    #[error("{0}")]
    NotFoundError(#[from] NotFoundError),
}

impl ResponseError for Error {
    fn status_code(&self) -> StatusCode {
        match *self {
            Error::NotFoundError(_) => StatusCode::NOT_FOUND,
            Error::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            Error::BadInputError(_) => StatusCode::BAD_REQUEST,
            Error::ReqwestError(ref e) => {
                eprintln!("{}", e);
                StatusCode::BAD_GATEWAY
            },
        }
    }
}

pub type Result<T> = core::result::Result<T, Error>;
