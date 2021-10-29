use thiserror::Error;

#[derive(Debug, Error)]
pub enum ButlerError {
    #[error("menu not found")]
    MenuNotFound,

    #[error("{0}")]
    HttpError(#[from] reqwest::Error),

    #[error("something went wrong when scraping")]
    ScrapeError,

    #[error("invalid menu id")]
    InvalidMenuId,
}

pub type ButlerResult<T> = Result<T, ButlerError>;
