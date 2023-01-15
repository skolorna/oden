use std::io::Cursor;

use async_zip::read::stream::ZipFileReader;
use futures::TryStreamExt;
use geo::Point;
use milli::{
    documents::{DocumentsBatchBuilder, DocumentsBatchReader},
    heed::EnvOpenOptions,
    update::{IndexDocuments, IndexDocumentsConfig, IndexerConfig},
};
use osm::OsmId;
use reqwest::{
    header::{self, HeaderMap},
    Client, IntoUrl,
};
use serde::Deserialize;
use tempdir::TempDir;
use tokio::{io::AsyncRead, task};
use tokio_util::io::StreamReader;

mod github {
    use reqwest::Client;
    use serde::Deserialize;
    use time::OffsetDateTime;

    #[derive(Debug, Deserialize)]
    pub struct Artifact {
        pub size_in_bytes: u64,
        pub archive_download_url: String,
        pub name: String,
        #[serde(with = "time::serde::rfc3339")]
        pub created_at: OffsetDateTime,
    }

    pub async fn list_artifacts(client: &Client, repo: &str) -> reqwest::Result<Vec<Artifact>> {
        #[derive(Debug, Deserialize)]
        struct Artifacts {
            artifacts: Vec<Artifact>,
        }

        Ok(client
            .get(format!(
                "https://api.github.com/repos/{repo}/actions/artifacts"
            ))
            .send()
            .await?
            .json::<Artifacts>()
            .await?
            .artifacts)
    }
}

async fn download_zip(
    client: &Client,
    url: impl IntoUrl,
) -> reqwest::Result<ZipFileReader<impl AsyncRead + Unpin>> {
    let stream = client
        .get(url)
        .send()
        .await?
        .bytes_stream()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e));

    Ok(ZipFileReader::new(StreamReader::new(stream)))
}

async fn download_latest_artifact(
    pat: &str,
) -> reqwest::Result<ZipFileReader<impl AsyncRead + Unpin>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::AUTHORIZATION,
        format!("Bearer {pat}").parse().unwrap(),
    );
    let client = Client::builder()
        .user_agent(crate::USER_AGENT)
        .default_headers(headers)
        .build()?;

    let artifacts = github::list_artifacts(&client, "akeamc/osm").await?;
    let latest = artifacts.into_iter().max_by_key(|a| a.created_at).unwrap();

    download_zip(&client, latest.archive_download_url).await
}

async fn append_csv(index: milli::Index, csv: impl AsyncRead + Unpin + Send) -> anyhow::Result<()> {
    let mut builder = DocumentsBatchBuilder::new(Cursor::new(Vec::new()));
    let mut records =
        csv_async::AsyncDeserializer::from_reader(csv).into_deserialize::<osm::Record>();

    while let Some(record) = records.try_next().await? {
        task::block_in_place(|| builder.append_json_object(&record_to_milli_obj(record)))?;
    }

    task::spawn_blocking(move || -> anyhow::Result<()> {
        let mut wtxn = index.write_txn()?;
        let buffer = builder.into_inner()?;
        let indexer_config = IndexerConfig::default();
        let builder = IndexDocuments::new(
            &mut wtxn,
            &index,
            &indexer_config,
            IndexDocumentsConfig::default(),
            |_| (),
            || false,
        )?;

        let (builder, res) = builder.add_documents(DocumentsBatchReader::from_reader(buffer)?)?;
        res?;
        builder.execute()?;
        wtxn.commit()?;

        Ok(())
    })
    .await??;

    Ok(())
}

pub struct Index {
    pub dir: TempDir,
    pub inner: milli::Index,
}

pub async fn build_index(gh_pat: &str) -> anyhow::Result<Index> {
    let dir = TempDir::new("munin-milli-db")?;

    let index = {
        let mut options = EnvOpenOptions::new();
        options.map_size(100 * 1024 * 1024); // 100 MiB
        milli::Index::new(options, dir.path())?
    };

    let mut zip = download_latest_artifact(gh_pat).await?;
    let csv = zip.entry_reader().await?.unwrap();

    append_csv(index.clone(), csv).await?;

    Ok(Index { inner: index, dir })
}

#[derive(Debug)]
pub struct Hit {
    pub name: String,
    pub id: OsmId,
    pub coordinates: Point,
}

pub fn parse_obkv(fields_ids_map: &milli::FieldsIdsMap, obkv: obkv::KvReader<'_, u16>) -> Hit {
    #[derive(Debug, Deserialize)]
    struct Geo {
        lat: f64,
        lon: f64,
    }

    impl From<Geo> for Point {
        fn from(Geo { lon, lat }: Geo) -> Self {
            Self::new(lon, lat)
        }
    }

    let mut name = None;
    let mut id = None;
    let mut coordinates: Option<Point> = None;

    for (key, value) in obkv.iter() {
        match fields_ids_map.name(key).expect("missing field name") {
            "name" => name = Some(serde_json::from_slice::<String>(value).unwrap()),
            "id" => id = Some(serde_json::from_slice::<OsmId>(value).unwrap()),
            "_geo" => coordinates = Some(serde_json::from_slice::<Geo>(value).unwrap().into()),
            _ => continue,
        }
    }

    Hit {
        name: name.unwrap(),
        id: id.unwrap(),
        coordinates: coordinates.unwrap(),
    }
}

fn record_to_milli_obj(record: osm::Record) -> milli::Object {
    use serde_json::{Map, Value};

    let osm::Record {
        name,
        osm_id,
        location,
        latitude,
        longitude,
    } = record;

    let mut map = Map::new();

    map.insert("id".to_owned(), Value::String(osm_id.to_string()));
    map.insert("name".to_owned(), Value::String(name));
    map.insert(
        "location".to_owned(),
        Value::Array(location.into_iter().map(Value::String).collect()),
    );

    let geo = {
        let mut coordinates = serde_json::Map::new();

        coordinates.insert(
            "lat".to_owned(),
            Value::Number(serde_json::Number::from_f64(latitude).unwrap()),
        );
        coordinates.insert(
            "lon".to_owned(),
            Value::Number(serde_json::Number::from_f64(longitude).unwrap()),
        );

        coordinates
    };

    map.insert("_geo".to_owned(), Value::Object(geo));

    map
}
