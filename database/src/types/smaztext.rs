use std::fmt::Display;

use diesel::{
    backend::Backend,
    sql_types,
    types::{FromSql, ToSql},
};
use munin_lib::menus::meal::Meal;
use serde::{Deserialize, Serialize};

/// A text type that is compressed with [Smaz](https://github.com/antirez/smaz)
/// in the database.
#[derive(
    Debug,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    AsExpression,
    FromSqlRow,
    Serialize,
    Deserialize,
)]
#[sql_type = "diesel::sql_types::Binary"]
pub struct SmazText(String);

impl Display for SmazText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<DB> ToSql<sql_types::Binary, DB> for SmazText
where
    DB: Backend,
    Vec<u8>: ToSql<sql_types::Binary, DB>,
{
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, DB>,
    ) -> diesel::serialize::Result {
        let compressed = smaz::compress(self.0.as_bytes());
        compressed.to_sql(out)
    }
}

impl<DB> FromSql<sql_types::Binary, DB> for SmazText
where
    DB: Backend,
    Vec<u8>: FromSql<sql_types::Binary, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        let compressed = Vec::<u8>::from_sql(bytes)?;
        let plain = smaz::decompress(&compressed)?;
        let s = String::from_utf8(plain)?;
        Ok(Self(s))
    }
}

impl From<String> for SmazText {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<SmazText> for Vec<Meal> {
    fn from(t: SmazText) -> Self {
        let lines = t.0.lines();

        lines
            .map(|l| Meal {
                value: l.to_owned(),
            })
            .collect()
    }
}
