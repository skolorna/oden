use std::time::Duration;

use anyhow::bail;
use meilisearch_sdk::{indexes::Index, tasks::Task, Client};
use osm::OsmId;
use serde::{Serialize, Serializer};
use sqlx::FromRow;
use stor::menu::Supplier;
use time::{Date, OffsetDateTime};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Serialize)]
struct Geo {
    lng: f64,
    lat: f64,
}

#[derive(Debug, FromRow)]
pub struct Menu {
    #[sqlx(flatten)]
    inner: stor::Menu,
    last_day: Option<Date>,
}

impl Serialize for Menu {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let Self { inner, last_day } = self;
        let stor::Menu {
            id,
            title,
            supplier,
            supplier_reference: _,
            location,
            osm_id,
            created_at,
            checked_at,
            consecutive_failures,
        } = inner;

        #[derive(Debug, Serialize)]
        struct Doc<'a> {
            id: Uuid,
            title: &'a str,
            #[serde(rename = "_geo", skip_serializing_if = "Option::is_none")]
            geo: Option<Geo>,
            last_day: Option<Date>,
            supplier: Supplier,
            osm_id: Option<OsmId>,
            #[serde(with = "time::serde::rfc3339::option")]
            created_at: Option<OffsetDateTime>,
            #[serde(with = "time::serde::rfc3339::option")]
            checked_at: Option<OffsetDateTime>,
            consecutive_failures: i32,
        }

        Doc {
            id: *id,
            title,
            geo: location.map(|p| Geo {
                lng: p.x(),
                lat: p.y(),
            }),
            last_day: *last_day,
            supplier: *supplier,
            osm_id: *osm_id,
            created_at: *created_at,
            checked_at: *checked_at,
            consecutive_failures: *consecutive_failures,
        }
        .serialize(serializer)
    }
}

pub async fn add_documents<T>(
    index: &Index,
    documents: &[T],
    primary_key: Option<&str>,
) -> anyhow::Result<()>
where
    T: Serialize,
{
    let task = index.add_documents(documents, primary_key).await?;

    info!(
        "queued {} documents for meilisearch indexing",
        documents.len()
    );

    match task
        .wait_for_completion(&index.client, None, Some(Duration::from_secs(30)))
        .await?
    {
        Task::Succeeded { content } => {
            info!(
                "indexed {} documents in {:.02} seconds",
                documents.len(),
                content.duration.as_secs_f64(),
            );

            Ok(())
        }
        Task::Failed { content } => bail!(meilisearch_sdk::errors::Error::from(content.error)),
        Task::Enqueued { .. } | Task::Processing { .. } => {
            bail!("timeout waiting for documents to be indexed")
        }
    }
}

pub async fn get_or_create_index(client: &Client, uid: impl AsRef<str>) -> anyhow::Result<Index> {
    let uid = uid.as_ref();

    if let Ok(index) = client.get_index(uid).await {
        Ok(index)
    } else {
        let task = client.create_index(uid, None).await?;
        let task = task
            .wait_for_completion(client, None, Some(std::time::Duration::from_secs(10)))
            .await?;
        match task {
            Task::Enqueued { .. } | Task::Processing { .. } => {
                bail!("timeout waiting for index creation")
            }
            Task::Failed { content } => {
                bail!(meilisearch_sdk::errors::Error::from(content.error))
            }
            Task::Succeeded { .. } => Ok(task.try_make_index(client).unwrap()),
        }
    }
}
