use std::{
    cmp::Ordering,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll}, fmt::Display,
};

use async_compression::tokio::{bufread::ZstdDecoder, write::ZstdEncoder};
use futures_core::Stream;
use futures_util::{join, pin_mut, StreamExt, TryStreamExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use time::Date;
use tokio::{
    io::{AsyncBufRead, AsyncWrite},
    time::Instant,
};
use uuid::Uuid;

use crate::{
    meal::{self, Key},
    merge::MergeStreamExt,
    Error, Result,
};

pub fn read<'a>(
    reader: impl AsyncBufRead + Unpin + Send + 'a,
) -> impl Stream<Item = csv_async::Result<Record>> + 'a {
    csv_async::AsyncDeserializer::from_reader(ZstdDecoder::new(reader)).into_deserialize()
}

pub struct Writer<W: AsyncWrite + Unpin> {
    inner: csv_async::AsyncSerializer<ZstdEncoder<W>>,
}

impl<W: AsyncWrite + Unpin> Writer<W> {
    pub fn new(writer: W) -> Self {
        let level = async_compression::Level::Precise(9);

        Self {
            inner: csv_async::AsyncSerializer::from_writer(ZstdEncoder::with_quality(
                writer, level,
            )),
        }
    }

    pub async fn write(&mut self, record: &Record) -> Result<()> {        
        self.inner.serialize(record).await.map_err(Into::into)
    }

    pub async fn flush(&mut self) -> Result<()> {
        self.inner.flush().await?;
        Ok(())
    }
}

pub async fn write<E>(
    writer: impl AsyncWrite + Unpin,
    mut records: impl Stream<Item = Result<Record, E>> + Unpin,
) -> Result<()>
where
    Error: From<E>,
{
    let mut writer = Writer::new(writer);

    while let Some(record) = records.try_next().await? {
        writer.write(&record).await?;
    }

    writer.flush().await?;

    Ok(())
}

pub async fn insert<W: AsyncWrite + Unpin>(
    writer: &mut Writer<W>,
    existing: impl AsyncBufRead + Unpin + Send,
    new: impl Stream<Item = Record> + Unpin,
) -> Result<()> {
    let existing = read(existing).map(|res| res.unwrap());
    let mut records = existing.merge_by(new, |n, e| n.key <= e.key);

    let mut k = Key::MIN;

    while let Some(record) = records.next().await {
        todo!("don't mix old and new meals");

        if k < record.key {
            k = record.key;

            println!("woho");
        } else {
            println!("oops, unordered");
        }
    }

    todo!()
}

#[derive(Debug, Deserialize, PartialEq, Eq)]
pub struct Record {
    #[serde(flatten)]
    pub key: meal::Key,
    pub value: String,
}

impl Serialize for Record {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Map<'a> {
            menu: Uuid,
            date: Date,
            i: usize,
            value: &'a str,
        }

        let Key { menu, date, i } = self.key;

        Map {
            menu,
            date,
            i,
            value: &self.value,
        }
        .serialize(serializer)
    }
}
