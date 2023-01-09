use std::path::Path;

use async_compression::tokio::bufread::{ZstdDecoder, ZstdEncoder};
use futures_util::TryStreamExt;
use reqwest::IntoUrl;
use sqlx::SqlitePool;
use tokio::{
    fs::OpenOptions,
    io::{self, AsyncBufRead, AsyncRead},
};
use tokio_util::io::StreamReader;

pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

const COMPRESSION_LEVEL: async_compression::Level = async_compression::Level::Precise(9);

pub fn compress(uncompressed: impl AsyncBufRead) -> impl AsyncRead {
    ZstdEncoder::with_quality(uncompressed, COMPRESSION_LEVEL)
}

pub fn decompress(compressed: impl AsyncBufRead) -> impl AsyncRead {
    ZstdDecoder::new(compressed)
}

pub async fn download(
    url: impl IntoUrl,
    destination: impl AsRef<Path>,
) -> anyhow::Result<SqlitePool> {
    let path = destination.as_ref();
    let mut destination = OpenOptions::new().create_new(true).open(path).await?;

    let res = reqwest::get(url).await?;
    let read = StreamReader::new(
        res.bytes_stream()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e)),
    );

    io::copy(&mut decompress(read), &mut destination).await?;

    let url = format!("sqlite://{}", path.display());
    let pool = SqlitePool::connect(&url).await?;

    Ok(pool)
}
