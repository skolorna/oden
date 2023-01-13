use std::path::{Path, PathBuf};

use futures::TryStreamExt;
use sqlx::{Acquire, PgConnection, PgPool};
use tokio::fs::File;

use crate::index::INSERTION_BATCH_SIZE;

#[derive(Debug, clap::Args)]
pub struct Args {
    #[clap(long)]
    menus: PathBuf,

    #[clap(long)]
    days: Option<PathBuf>,
}

async fn import_menus(conn: &mut PgConnection, path: &Path) -> anyhow::Result<()> {
    let mut menus = csv_async::AsyncDeserializer::from_reader(File::open(path).await?)
        .into_deserialize::<csv::Menu>();

    let mut txn = conn.begin().await?;

    while let Some(csv::Menu { id, title, slug }) = menus.try_next().await? {
        sqlx::query!(
            r#"
                INSERT INTO menus (id, title, supplier, supplier_reference)
                VALUES ($1, $2, $3, $4)
            "#,
            id,
            title,
            slug.0,
            slug.1
        )
        .execute(&mut txn)
        .await?;
    }

    Ok(txn.commit().await?)
}

async fn import_days(pool: &PgPool, path: &Path) -> anyhow::Result<()> {
    let mut days = csv_async::AsyncDeserializer::from_reader(File::open(path).await?)
        .into_deserialize::<csv::Day>();

    let pb = indicatif::ProgressBar::new_spinner()
        .with_style(
            indicatif::ProgressStyle::with_template("{spinner} {msg} ({pos} done)").unwrap(),
        )
        .with_message("importing days");

    let mut txn = pool.begin().await?;
    let mut uncommitted = 0usize;

    while let Some(csv::Day {
        menu_id,
        date,
        meals,
    }) = days.try_next().await?
    {
        sqlx::query!(
            "INSERT INTO days (menu_id, date, meals) VALUES ($1, $2, $3)",
            menu_id,
            date,
            meals,
        )
        .execute(&mut txn)
        .await?;

        uncommitted += 1;
        pb.inc(1);

        if uncommitted >= INSERTION_BATCH_SIZE {
            txn.commit().await?;
            uncommitted = 0;
            txn = pool.begin().await?;
        }
    }

    txn.commit().await?;
    pb.finish_and_clear();
    Ok(())
}

pub async fn import(opt: Args, pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query("PRAGMA journal_mode=WAL").execute(pool).await?;
    sqlx::query("PRAGMA busy_timeout=60000")
        .execute(pool)
        .await?;

    let mut conn = pool.acquire().await?;

    import_menus(&mut conn, &opt.menus).await?;

    if let Some(ref path) = opt.days {
        import_days(pool, path).await?;
    }

    Ok(())
}

mod csv {
    use std::str::FromStr;

    use serde::{de, Deserialize, Deserializer};
    use stor::{day::Meals, menu::Supplier};
    use time::Date;
    use uuid::Uuid;

    #[derive(Debug)]
    pub struct Slug(pub Supplier, pub String);

    impl FromStr for Slug {
        type Err = ();

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let (supplier, slug) = s.split_once('.').ok_or(())?;

            Ok(Self(supplier.parse().map_err(|_| ())?, slug.to_owned()))
        }
    }

    impl<'de> Deserialize<'de> for Slug {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            let s = String::deserialize(deserializer)?;
            s.parse().map_err(|_| de::Error::custom("invalid slug"))
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct Menu {
        pub id: Uuid,
        pub title: String,
        pub slug: Slug,
    }

    mod meals {
        use serde::{de, Deserialize, Deserializer};
        use stor::day::Meals;

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Meals, D::Error>
        where
            D: Deserializer<'de>,
        {
            #[derive(Debug, Deserialize)]
            struct Meal {
                value: String,
            }

            let json = String::deserialize(deserializer)?;

            let meals = serde_json::from_str::<Vec<Meal>>(&json)
                .map_err(de::Error::custom)?
                .into_iter()
                .map(|m| stor::Meal(m.value))
                .collect::<Vec<_>>();

            Ok(Meals::new(meals))
        }
    }

    #[derive(Debug, Deserialize)]
    pub struct Day {
        pub menu_id: Uuid,
        pub date: Date,
        #[serde(with = "meals")]
        pub meals: Meals,
    }
}
