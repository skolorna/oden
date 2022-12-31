use serde::{Deserialize, Serialize};
use sqlx::{
    sqlite::{SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef},
    Decode,
};
#[cfg(feature = "db")]
use sqlx::{Encode, Sqlite};
use time::Date;

use crate::Meal;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Day {
    pub date: Date,
    pub meals: Meals,
}

impl Day {
    #[must_use]
    pub fn new(date: Date, meals: Vec<Meal>) -> Option<Self> {
        if meals.is_empty() {
            None
        } else {
            Some(Self {
                date,
                meals: Meals(meals),
            })
        }
    }

    #[must_use]
    pub const fn date(&self) -> &Date {
        &self.date
    }

    #[must_use]
    pub fn meals(&self) -> &Meals {
        &self.meals
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "db", derive(minicbor::Encode, minicbor::Decode))]
pub struct Meals(#[cfg_attr(feature = "db", n(0))] Vec<Meal>);

impl Meals {
    pub fn into_inner(self) -> Vec<Meal> {
        self.0
    }
}

#[cfg(feature = "db")]
impl Encode<'_, Sqlite> for Meals {
    fn encode_by_ref(&self, buf: &mut Vec<SqliteArgumentValue<'_>>) -> sqlx::encode::IsNull {
        let mut cbor = Vec::new();
        minicbor::encode(self, &mut cbor).unwrap();
        <Vec<u8> as sqlx::Encode<Sqlite>>::encode(cbor, buf)
    }
}

#[cfg(feature = "db")]
impl Decode<'_, Sqlite> for Meals {
    fn decode(value: SqliteValueRef) -> Result<Self, sqlx::error::BoxDynError> {
        let bytes = <&[u8] as Decode<Sqlite>>::decode(value)?;
        Ok(minicbor::decode(bytes)?)
    }
}

impl sqlx::Type<Sqlite> for Meals {
    fn type_info() -> SqliteTypeInfo {
        <&[u8] as sqlx::Type<Sqlite>>::type_info()
    }
}

#[cfg(feature = "db")]
#[cfg(test)]
mod tests {
    use osm::OsmId;
    use sqlx::{pool::PoolConnection, Sqlite};
    use time::macros::date;
    use uuid::Uuid;

    use crate::{menu::Supplier, Day, Meal, Menu};

    use super::Meals;

    #[sqlx::test]
    async fn from_row(mut conn: PoolConnection<Sqlite>) -> sqlx::Result<()> {
        let day = Day {
            date: date!(2022 - 12 - 09),
            meals: Meals(vec![
                Meal {
                    value: "Fisk Björkeby".to_owned(),
                },
                Meal {
                    value: "Fisk Bordelaise".to_owned(),
                },
            ]),
        };

        let menu = Menu {
            id: Uuid::new_v4(),
            title: "Skola".to_owned(),
            supplier: Supplier::Sodexo,
            supplier_reference: "69420".to_owned(),
            location: None,
            osm_id: Some(OsmId::Way(104245269)),
        };
        let osm_id = menu.osm_id.map(|id| id.to_string());

        sqlx::query!(
            "INSERT INTO menus (id, title, supplier, supplier_reference, osm_id) VALUES ($1, $2, $3, $4, $5)",
            menu.id,
            menu.title,
            menu.supplier,
            menu.supplier_reference,
            osm_id,
        )
        .execute(&mut conn)
        .await?;

        sqlx::query!(
            "INSERT INTO days (menu_id, date, meals) VALUES ($1, $2, $3)",
            menu.id,
            day.date,
            day.meals
        )
        .execute(&mut conn)
        .await?;

        let data: Day = sqlx::query_as("SELECT * FROM days WHERE menu_id = $1 AND date = $2")
            .bind(menu.id)
            .bind(day.date)
            .fetch_one(&mut conn)
            .await?;

        assert_eq!(data, day);

        Ok(())
    }
}
