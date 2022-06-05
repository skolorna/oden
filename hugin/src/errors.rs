use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("menu not found")]
    MenuNotFound,

    #[error("{0}")]
    HttpError(#[from] reqwest::Error),

    #[error("something went wrong when scraping {context}")]
    ScrapeError { context: String },

    #[error("invalid menu id")]
    InvalidMenuSlug,
}

pub(crate) type Result<T> = core::result::Result<T, Error>;
