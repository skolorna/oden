use geo::Point;
use osm::OsmId;
use serde::{Deserialize, Serialize};
#[cfg(feature = "db")]
use sqlx::{postgres::PgRow, FromRow, Row};
use strum::{EnumIter, EnumString};
use time::OffsetDateTime;
use uuid::Uuid;

pub const UUID_NAMESPACE: Uuid = Uuid::from_bytes([
    0x88, 0xdc, 0x80, 0xe5, 0xf4, 0x7f, 0x46, 0x34, 0xb6, 0x33, 0x2c, 0xce, 0x5e, 0xf2, 0xcb, 0x11,
]);

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, EnumIter, EnumString, strum::Display,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
#[cfg_attr(
    feature = "db",
    derive(sqlx::Type),
    sqlx(type_name = "supplier", rename_all = "lowercase") // postgres type defined in migrations
)]
pub enum Supplier {
    Skolmaten,
    Sodexo,
    Mpi,
    Kleins,
    Sabis,
    Matilda,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Menu {
    pub id: Uuid,
    pub title: String,
    pub supplier: Supplier,
    pub supplier_reference: String,
    pub location: Option<Point>,
    pub osm_id: Option<OsmId>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub created_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub checked_at: Option<OffsetDateTime>,
    pub consecutive_failures: i32,
}

/// A patch to a menu.
///
/// ```
/// # use geo::Point;
/// # use stor::menu::{Menu, Patch, Supplier};

/// let mut menu = Menu::from_supplier(Supplier::Skolmaten, "123", "title");
/// menu.location = Some(Point::new(1.0, 2.0));
///
/// menu.patch(Patch {
///     title: Some("new title".to_string()),
///     location: None,
///     osm_id: None,
/// });
///
/// assert_eq!(menu.title, "new title");
/// assert_eq!(menu.location, Some(Point::new(1.0, 2.0)));
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Patch {
    pub title: Option<String>,
    pub location: Option<Point>,
    pub osm_id: Option<OsmId>,
}

impl Patch {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

impl Menu {
    pub fn from_supplier(
        supplier: Supplier,
        supplier_reference: impl Into<String>,
        title: impl Into<String>,
    ) -> Self {
        let supplier_reference = supplier_reference.into();
        let id = Uuid::new_v5(
            &UUID_NAMESPACE,
            format!("{supplier}.{supplier_reference}").as_bytes(), // maintain compatibility with older versions
        );

        Self {
            id,
            title: title.into(),
            supplier,
            supplier_reference,
            location: None,
            osm_id: None,
            created_at: None,
            checked_at: None,
            consecutive_failures: 0,
        }
    }

    pub fn patch(&mut self, patch: Patch) {
        let Patch {
            title,
            location,
            osm_id,
        } = patch;

        if let Some(title) = title {
            self.title = title;
        }

        if let Some(location) = location {
            self.location = Some(location);
        }

        if let Some(osm_id) = osm_id {
            self.osm_id = Some(osm_id);
        }
    }
}

#[cfg(feature = "db")]
impl FromRow<'_, PgRow> for Menu {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
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
            created_at: row.try_get("created_at")?,
            checked_at: row.try_get("checked_at")?,
            consecutive_failures: row.try_get("consecutive_failures")?,
        })
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::menu::Supplier;

    use super::Menu;

    #[cfg(feature = "db")]
    #[sqlx::test]
    async fn menu_from_row(pool: sqlx::PgPool) -> sqlx::Result<()> {
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

    #[test]
    fn id_generation() {
        let menu = Menu::from_supplier(Supplier::Skolmaten, "123", "title");
        assert_eq!(
            menu.id,
            Uuid::parse_str("42b0b314-118c-5e1e-8fa7-c0ebdef301b5").unwrap()
        );
    }
}
