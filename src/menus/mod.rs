pub mod id;
pub mod provider;
pub mod skolmaten;

use chrono::{Duration, Local, NaiveDate};
use serde::Serialize;
use skolmaten::days::SkolmatenMeal;

use self::{
    id::MenuID,
    provider::{Provider, ProviderInfo},
};
use crate::errors::{self, BadInputError, Error, RangeError, Result};

#[derive(Serialize, Debug)]
pub struct Meal {
    value: String,
}

impl From<SkolmatenMeal> for Meal {
    fn from(meal: SkolmatenMeal) -> Self {
        Self { value: meal.value }
    }
}

#[derive(Serialize, Debug)]
pub struct Day {
    /// Time zones aren't really relevant here.
    pub date: NaiveDate,
    pub meals: Vec<Meal>,
}

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
            Err(Error::BadInputError(BadInputError::RangeError(
                RangeError::DatesOutOfRange,
            )))
        } else {
            Ok(Self {
                menu_id,
                first,
                last,
            })
        }
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

    match query.menu_id.provider {
        Skolmaten => skolmaten::list_days(&query.menu_id.local_id, query.first, query.last).await,
    }
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
