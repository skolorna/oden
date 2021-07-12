pub mod day;
pub mod id;
pub mod mashie;
pub mod meal;
pub mod provider;
pub mod skolmaten;
pub mod sodexo;

use chrono::{Duration, Local, NaiveDate};
use serde::{Deserialize, Serialize};

use self::{
    day::Day,
    id::MenuID,
    meal::Meal,
    provider::{Provider, ProviderInfo},
};
use crate::errors::{self, BadInputError, Error, RangeError, Result};

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Menu {
    pub id: MenuID,
    pub title: String,
    pub provider: ProviderInfo,
}

impl Menu {
    pub fn new(id: MenuID, title: String) -> Self {
        Self {
            provider: id.provider.info(),
            id,
            title,
        }
    }
}

pub async fn list_menus() -> Result<Vec<Menu>> {
    let mut menus = vec![];

    let mut skolmaten_menus = skolmaten::list_menus().await?;
    let mut sodexo_menus = sodexo::list_menus().await?;

    menus.append(&mut skolmaten_menus);
    menus.append(&mut sodexo_menus);

    Ok(menus)
}

pub async fn query_menu(menu_id: &MenuID) -> Result<Menu> {
    use Provider::*;

    match menu_id.provider {
        Skolmaten => skolmaten::query_menu(&menu_id.local_id).await,
        Sodexo => sodexo::query_menu(&menu_id.local_id).await,
    }
}

pub async fn list_days(query: &ListDaysQuery) -> Result<Vec<Day>> {
    use Provider::*;

    match query.menu_id.provider {
        Skolmaten => skolmaten::list_days(&query.menu_id.local_id, query.first, query.last).await,
        Sodexo => sodexo::list_days(&query.menu_id.local_id, query.first, query.last).await,
    }
}

#[cfg(test)]
mod tests {
    use crate::util::is_sorted;

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

    #[actix_rt::test]
    async fn listing_days() {
        let menu_id = MenuID::new(Provider::Skolmaten, "4791333780717568".to_owned());
        let first = NaiveDate::from_ymd(2017, 12, 1);
        let last = NaiveDate::from_ymd(2018, 1, 31);
        let query = ListDaysQuery::new(menu_id, Some(first), Some(last)).unwrap();
        let days = list_days(&query).await.unwrap();

        assert_eq!(days.len(), 41);
        assert!(is_sorted(&days));

        let first_day = days.get(0).unwrap();
        assert_eq!(first_day.date, NaiveDate::from_ymd(2017, 12, 1));

        for day in days.iter() {
            assert!(day.date >= first);
            assert!(day.date <= last);
        }
    }
}
