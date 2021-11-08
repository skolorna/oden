use std::fmt::Display;

use diesel::{
    backend::Backend,
    sql_types,
    types::{FromSql, ToSql},
};
use serde::{Deserialize, Serialize};

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
pub struct ShocoText(String);

impl Display for ShocoText {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<DB> ToSql<sql_types::Binary, DB> for ShocoText
where
    DB: Backend,
    Vec<u8>: ToSql<sql_types::Binary, DB>,
{
    fn to_sql<W: std::io::Write>(
        &self,
        out: &mut diesel::serialize::Output<W, DB>,
    ) -> diesel::serialize::Result {
        let compressed = shoco::compress(self.0.as_str());
        compressed.to_sql(out)
    }
}

impl<DB> FromSql<sql_types::Binary, DB> for ShocoText
where
    DB: Backend,
    Vec<u8>: FromSql<sql_types::Binary, DB>,
{
    fn from_sql(bytes: Option<&DB::RawValue>) -> diesel::deserialize::Result<Self> {
        let compressed = Vec::<u8>::from_sql(bytes)?;
        let plain = shoco::decompress(&compressed);
        let s = String::from_utf8(plain)?;
        Ok(Self(s))
    }
}

impl From<String> for ShocoText {
    fn from(s: String) -> Self {
        Self(s)
    }
}
