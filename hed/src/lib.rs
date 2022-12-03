use std::convert::Infallible;

use serde::Serialize;
use time::Date;
use uuid::Uuid;

pub mod archive;
mod merge;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("csv error: {0}")]
    Csv(#[from] csv_async::Error),

    #[error("io error: {0}")]
    Io(#[from] tokio::io::Error)
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

pub struct Day {
    pub menu: Uuid,
    pub date: Date,
    pub meals: Vec<String>,
}

pub mod meal {
    use std::fmt::Display;

    use serde::{Deserialize, Serialize};
    use time::Date;
    use uuid::Uuid;

    #[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct Key {
        pub menu: Uuid,
        pub date: Date,
        pub i: usize,
    }

    impl Key {
        pub const MIN: Self = Self {
            menu: Uuid::nil(),
            date: Date::MIN,
            i: 0,
        };
    }

    impl Display for Key {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}:{}", self.menu, self.date, self.i)
    }
    }
}
