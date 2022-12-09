use std::time::Duration;

use anyhow::bail;
use meilisearch_sdk::{indexes::Index, tasks::Task, Client};
use serde::Serialize;
use tracing::info;

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
        .wait_for_completion(&index.client, None, Some(Duration::from_secs(10)))
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
            .wait_for_completion(&client, None, Some(std::time::Duration::from_secs(10)))
            .await?;
        match task {
            Task::Enqueued { .. } | Task::Processing { .. } => {
                bail!("timeout waiting for index creation")
            }
            Task::Failed { content } => {
                bail!(meilisearch_sdk::errors::Error::from(content.error))
            }
            Task::Succeeded { .. } => Ok(task.try_make_index(&client).unwrap()),
        }
    }
}
