use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::{sqlite::SqliteValueRef, Decode, FromRow, Sqlite};
use sqlx::{
    sqlite::{SqliteRow, SqliteTypeInfo},
    Row, Type,
};
use uuid::Uuid;

#[cfg(feature = "db")]
pub mod db;

#[derive(Debug, Serialize, Deserialize)]
struct Coord {
    longitude: f64,
    latitude: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlobUuid(Uuid);

impl Decode<'_, Sqlite> for BlobUuid {
    fn decode(value: SqliteValueRef) -> Result<Self, sqlx::error::BoxDynError> {
        let bytes = <&[u8] as Decode<Sqlite>>::decode(value)?;

        Ok(Self(Uuid::from_slice(bytes)?))
    }
}

#[cfg(feature = "db")]
impl Type<Sqlite> for BlobUuid {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as Type<Sqlite>>::type_info()
    }

    fn compatible(ty: &SqliteTypeInfo) -> bool {
        <&[u8] as Type<Sqlite>>::compatible(ty)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Menu {
    id: BlobUuid,
    title: String,
    slug: String,
    location: Option<Coord>,
}

#[cfg(feature = "db")]
impl FromRow<'_, SqliteRow> for Menu {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let location = {
            match (row.try_get("longitude")?, row.try_get("latitude")?) {
                (Some(longitude), Some(latitude)) => Some(Coord {
                    longitude,
                    latitude,
                }),
                _ => None,
            }
        };

        Ok(Self {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            slug: row.try_get("slug")?,
            location,
        })
    }
}

impl Menu {
    pub const fn id(&self) -> &Uuid {
        &self.id.0
    }
}

#[cfg(feature = "db")]
#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use uuid::Uuid;

    use crate::Menu;

    #[sqlx::test]
    async fn menu_from_row(pool: SqlitePool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let id = Uuid::new_v4();
        let title = "School";
        let slug = "skolmaten.123";

        sqlx::query("INSERT INTO menus (id, title, slug) VALUES ($1, $2, $3)")
            .bind(&id.as_bytes()[..])
            .bind(title)
            .bind(slug)
            .fetch_all(&mut conn)
            .await?;

        let menu = sqlx::query_as::<_, Menu>("SELECT * FROM menus")
            .fetch_one(&mut conn)
            .await?;

        assert_eq!(*menu.id(), id);
        assert_eq!(menu.title, title);
        assert_eq!(menu.slug, slug);

        Ok(())
    }
}
