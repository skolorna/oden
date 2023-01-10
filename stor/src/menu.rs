use geo::Point;
use osm::OsmId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::{sqlite::SqliteRow, FromRow, Row};
use uuid::Uuid;

pub const UUID_NAMESPACE: Uuid = Uuid::from_bytes([
    0x88, 0xdc, 0x80, 0xe5, 0xf4, 0x7f, 0x46, 0x34, 0xb6, 0x33, 0x2c, 0xce, 0x5e, 0xf2, 0xcb, 0x11,
]);

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "db", derive(sqlx::Type), sqlx(rename_all = "lowercase"))]
pub enum Supplier {
    Skolmaten,
    Sodexo,
    Mpi,
    Kleins,
    Sabis,
    Matilda,
}

impl Supplier {
    pub fn iter() -> impl Iterator<Item = Self> {
        [
            Self::Skolmaten,
            Self::Sodexo,
            Self::Mpi,
            Self::Kleins,
            Self::Sabis,
            Self::Matilda,
        ]
        .into_iter()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Menu {
    pub id: Uuid,
    pub title: String,
    pub supplier: Supplier,
    pub supplier_reference: String,
    pub location: Option<Point>,
    pub osm_id: Option<OsmId>,
}

impl Menu {
    pub fn from_supplier(
        supplier: Supplier,
        supplier_reference: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        let supplier_reference = supplier_reference.into();
        let id = Uuid::new_v5(&UUID_NAMESPACE, supplier_reference.as_bytes());

        Self {
            id,
            title: title.into(),
            supplier,
            supplier_reference,
            location: None,
            osm_id: None,
        }
    }
}

#[cfg(feature = "db")]
impl FromRow<'_, SqliteRow> for Menu {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let location = {
            match (row.try_get("longitude")?, row.try_get("latitude")?) {
                (Some(longitude), Some(latitude)) => Some(Point::new(longitude, latitude)),
                _ => None,
            }
        };

        let osm_id = row
            .try_get::<Option<String>, _>("osm_id")?
            .map(|s| s.parse::<OsmId>())
            .transpose()
            .map_err(|e| sqlx::Error::Decode(e.into()))?;

        Ok(Self {
            id: row.try_get("id")?,
            title: row.try_get("title")?,
            supplier: row.try_get("supplier")?,
            supplier_reference: row.try_get("supplier_reference")?,
            osm_id,
            location,
        })
    }
}

#[cfg(feature = "db")]
#[cfg(test)]
mod tests {
    use sqlx::SqlitePool;
    use uuid::Uuid;

    use crate::menu::Supplier;

    use super::Menu;

    #[sqlx::test]
    async fn menu_from_row(pool: SqlitePool) -> sqlx::Result<()> {
        let mut conn = pool.acquire().await?;

        let id = Uuid::new_v4();
        let title = "School";
        let supplier = Supplier::Skolmaten;
        let supplier_reference = "12345";

        sqlx::query(
            "INSERT INTO menus (id, title, supplier, supplier_reference) VALUES ($1, $2, $3, $4)",
        )
        .bind(id)
        .bind(title)
        .bind(supplier)
        .bind(supplier_reference)
        .fetch_all(&mut conn)
        .await?;

        let menu = sqlx::query_as::<_, Menu>("SELECT * FROM menus")
            .fetch_one(&mut conn)
            .await?;

        assert_eq!(menu.id, id);
        assert_eq!(menu.title, title);
        assert_eq!(menu.supplier, supplier);
        assert_eq!(menu.supplier_reference, supplier_reference);

        Ok(())
    }
}
