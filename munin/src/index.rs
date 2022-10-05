use std::fs::create_dir_all;
use std::io::Write;
use std::{
    collections::{BTreeMap, BinaryHeap},
    fs::{self, File},
    path::{Path, PathBuf},
};

use chrono::{DateTime, Duration, TimeZone, Utc};
use clap::Parser;
use futures::{future, stream, Stream, StreamExt};
use git2::{build::RepoBuilder, Cred, RemoteCallbacks, Repository};
use hugin::menus::list_menus;
use serde::Serialize;
use tempfile::tempdir;
use tracing::{debug, info, instrument, warn};
use uuid::Uuid;

#[derive(Parser, Debug)]
pub struct Args {
    #[clap(long, default_value = "90")]
    days: u32,

    #[clap(long, env)]
    repo: String,

    #[clap(long, env)]
    ssh_key_path: PathBuf,

    #[clap(long, default_value = "50")]
    concurrent: usize,
}

impl Args {
    pub fn clone_repo(&self, dest: &Path) -> anyhow::Result<Dataset> {
        let mut cbs = RemoteCallbacks::new();
        cbs.credentials(|_url, username_from_url, _allowed_types| {
            Cred::ssh_key(username_from_url.unwrap(), None, &self.ssh_key_path, None)
        });

        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(cbs);

        let mut builder = RepoBuilder::new();
        builder.fetch_options(fo);

        let repo = builder.clone(&self.repo, dest)?;

        Ok(Dataset(repo))
    }
}

#[derive(Debug, Serialize)]
pub struct Menu {
    id: Uuid,
    slug: hugin::MenuSlug,
    title: String,
    last_update: Option<DateTime<Utc>>,
}

impl From<hugin::Menu> for Menu {
    fn from(m: hugin::Menu) -> Self {
        Self {
            id: m.get_uuid(),
            slug: m.slug,
            title: m.title,
            last_update: None,
        }
    }
}

pub struct Dataset(pub Repository);

impl Dataset {
    fn menus_path(&self) -> PathBuf {
        self.0.workdir().unwrap().join("menus.csv")
    }

    fn days_path(&self, menu: Uuid) -> PathBuf {
        self.0
            .workdir()
            .unwrap()
            .join("days")
            .join(format!("{menu}.csv"))
    }

    pub fn get_menus(&self) -> anyhow::Result<BTreeMap<Uuid, Menu>> {
        let path = self.menus_path();

        if !path.exists() {
            warn!(?path, "menu index does not exist");
            return Ok(BTreeMap::new());
        }

        let file = File::open(self.menus_path())?;

        todo!();
    }

    pub fn insert_menus(
        &self,
        new: impl IntoIterator<Item = Menu>,
    ) -> anyhow::Result<BTreeMap<Uuid, Menu>> {
        let path = self.menus_path();

        let mut menus = self.get_menus()?;

        for menu in new {
            // don't overwrite last_update
            menus
                .entry(menu.id)
                .and_modify(|e| {
                    e.title = menu.title.clone();
                    e.slug = menu.slug.clone();
                })
                .or_insert(menu);
        }

        let mut wtr = csv::Writer::from_path(path)?;

        for (_, menu) in &menus {
            wtr.serialize(menu)?;
        }

        wtr.flush()?;

        Ok(menus)
    }

    pub fn get_days(&self, menu: Uuid) -> anyhow::Result<BinaryHeap<hugin::Day>> {
        let path = self.days_path(menu);

        if !path.exists() {
            warn!(?path, %menu, "list of days does not exist");
            return Ok(BinaryHeap::new());
        }

        todo!();

        // let mut reader = csv::Reader::from_path(path)
    }

    pub fn insert_days(
        &self,
        menu: Uuid,
        new: impl IntoIterator<Item = hugin::Day>,
    ) -> anyhow::Result<()> {
        let path = self.days_path(menu);

        let mut days = self.get_days(menu)?;
        days.extend(new);

        create_dir_all(path.parent().unwrap())?;
        let mut wtr = csv::WriterBuilder::new().from_path(path)?;

        wtr.write_record(&["date", "meals"])?;

        for day in days.into_iter_sorted() {
            let hugin::Day { date, meals } = day;

            let meals = meals
                .into_iter()
                .map(|hugin::Meal { value }| value)
                .intersperse("\\n".to_string())
                .collect::<String>();

            wtr.write_field(date.to_string())?;
            wtr.write_field(meals)?;
            wtr.write_record(None::<&[u8]>)?;

            // wtr.write_record(&[date, meals])?;
        }

        Ok(())
    }
}

pub async fn index(args: Args) -> anyhow::Result<()> {
    debug!(?args, "indexing");

    let menus = list_menus(4).await?;

    info!("got {} menus", menus.len());

    // let dir = tempdir()?;

    let ds = args.clone_repo("../x".as_ref())?;

    let menus = ds.insert_menus(menus.into_iter().map(Into::into))?;

    let mut stream = list_days(menus.values(), args);

    while let Some((menu, res)) = stream.next().await {
        let days = match res {
            Ok(days) => days,
            Err(e) => {
                warn!(%menu, "failed to list days: {e}");
                continue;
            }
        };

        ds.insert_days(menu, days)?;
    }

    // list_days(menu_slug, first, last)

    Ok(())
}

fn list_days<'a>(
    menus: impl IntoIterator<Item = &'a Menu> + 'a,
    args: Args,
) -> impl Stream<Item = (Uuid, Result<Vec<hugin::Day>, hugin::Error>)> + 'a {
    let utc = Utc::now().naive_utc().date();
    let first = chrono_tz::Europe::Stockholm
        .from_utc_date(&utc)
        .naive_local();
    let last = first + Duration::days(90);

    stream::iter(menus)
        .map(move |m| async move { (m.id, hugin::menus::list_days(&m.slug, first, last).await) })
        .buffer_unordered(args.concurrent)
}
