pub mod day;
pub mod id;
pub mod meal;
pub mod provider;
pub mod skolmaten;

use chrono::{Duration, Local, NaiveDate};
use serde::Serialize;

use self::{
    day::Day,
    id::MenuID,
    meal::Meal,
    provider::{Provider, ProviderInfo},
};
use crate::{
    errors::{self, BadInputError, Error, RangeError, Result},
    menus::day::dedup_dates,
};

pub struct ListDaysQuery {
    menu_id: MenuID,
    first: NaiveDate,
    last: NaiveDate,
}

impl ListDaysQuery {
    pub fn new(
        menu_id: MenuID,
        first: Option<NaiveDate>,
        last: Option<NaiveDate>,
    ) -> errors::Result<Self> {
        let first = first.unwrap_or_else(|| Local::now().date().naive_local());
        let last = last.unwrap_or_else(|| first + Duration::weeks(2));

        if first > last {
            return Err(Error::BadInputError(BadInputError::RangeError(
                RangeError::DatesOutOfRange,
            )));
        }

        if last - first > Duration::days(3650) {
            return Err(Error::BadInputError(BadInputError::RangeError(
                RangeError::DateSpanTooLong,
            )));
        }

        Ok(Self {
            menu_id,
            first,
            last,
        })
    }
}

#[derive(Serialize)]
pub struct Menu {
    id: MenuID,
    title: String,
    provider: ProviderInfo,
}

impl Menu {
    pub fn new(id: MenuID, title: &str, provider: Provider) -> Self {
        Self {
            id,
            title: title.to_owned(),
            provider: provider.info(),
        }
    }
}

pub async fn list_menus() -> Result<Vec<Menu>> {
    let menus = skolmaten::list_menus().await?;

    Ok(menus)
}

pub async fn query_menu(menu_id: &MenuID) -> Result<Menu> {
    use Provider::*;

    match menu_id.provider {
        Skolmaten => skolmaten::query_menu(&menu_id.local_id).await,
    }
}

pub async fn list_days(query: &ListDaysQuery) -> Result<Vec<Day>> {
    use Provider::*;

    let mut days = match query.menu_id.provider {
        Skolmaten => skolmaten::list_days(&query.menu_id.local_id, query.first, query.last).await?,
    };

    dedup_dates(&mut days);

    Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_days_query_validation() {
        let menu_id = MenuID::new(Provider::Skolmaten, "abc".to_owned());

        assert!(ListDaysQuery::new(
            menu_id.clone(),
            Some(NaiveDate::from_ymd(2020, 6, 1)),
            Some(NaiveDate::from_ymd(2020, 1, 1))
        )
        .is_err());

        assert!(ListDaysQuery::new(
            menu_id.clone(),
            None,
            Some(NaiveDate::from_ymd(1789, 7, 14))
        )
        .is_err());

        assert!(ListDaysQuery::new(
            menu_id.clone(),
            Some(NaiveDate::from_ymd(1789, 7, 14)),
            Some(NaiveDate::from_ymd(2000, 1, 1))
        )
        .is_err());

        assert!(ListDaysQuery::new(menu_id.clone(), None, None).is_ok());

        assert!(ListDaysQuery::new(
            menu_id.clone(),
            Some(NaiveDate::from_ymd(2020, 1, 1)),
            Some(NaiveDate::from_ymd(2020, 1, 1))
        )
        .is_ok());

        assert!(ListDaysQuery::new(menu_id, Some(NaiveDate::from_ymd(1789, 7, 14)), None).is_ok());
    }
}
