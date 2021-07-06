use chrono::{Datelike, NaiveDate};
use reqwest::Client;
use serde::Deserialize;

use crate::{
    errors::{Error, NotFoundError, Result},
    menus::{LocalDay, LocalMeal, LocalMenu},
};

use super::{fetch::fetch, District, Station};

#[derive(Deserialize, Debug, Clone)]
pub(super) struct DetailedStation {
    district: District,
    #[serde(flatten)]
    station: Station,
}

impl DetailedStation {
    pub fn to_local_menu(&self) -> Option<LocalMenu> {
        self.station.to_local_menu(&self.district.name)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub(super) struct Meal {
    value: String,
    attributes: Vec<u32>,
}

impl Meal {
    fn new(value: String, attributes: Vec<u32>) -> Self {
        Self { value, attributes }
    }
}

impl Into<LocalMeal> for Meal {
    fn into(self) -> LocalMeal {
        LocalMeal { value: self.value }
    }
}

#[derive(Deserialize, Debug)]
pub(super) struct Day {
    year: i32,
    month: u32,
    day: u32,
    meals: Option<Vec<Meal>>,
}

impl Day {
    fn new(meals: Vec<Meal>, year: i32, month: u32, day: u32) -> Self {
        Self {
            year,
            month,
            day,
            meals: Some(meals),
        }
    }

    /// Maps `NaiveDate::from_ymd_opt` and creates a LocalDay; thus, `None` is returned on invalid dates such as *February 29, 2021*. Also, `None` is returned if `meals` is `None`.
    fn into_local_day(self) -> Option<LocalDay> {
        let date = NaiveDate::from_ymd_opt(self.year, self.month, self.day)?;
        let meals: Vec<LocalMeal> = self.meals?.into_iter().map(|meal| meal.into()).collect();

        if meals.is_empty() {
            None
        } else {
            Some(LocalDay { date, meals })
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Week {
    year: i32,
    week_of_year: u32,
    days: Vec<Day>,
}

#[derive(Deserialize, Debug)]
pub(super) struct Bulletin {
    text: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Menu {
    is_feedback_allowed: bool,
    weeks: Vec<Week>,
    station: DetailedStation,
    id: u64,
    bulletins: Vec<Bulletin>,
}

#[derive(Deserialize, Debug)]
pub(super) struct MenuResponse {
    menu: Menu,
}

async fn raw_fetch_menu(
    client: &Client,
    station_id: u64,
    year: i32,
    week_of_year: u32,
    count: u8,
) -> Result<MenuResponse> {
    let path = format!(
        "menu?station={}&year={}&weekOfYear={}&count={}",
        station_id, year, week_of_year, count
    );

    let res = fetch(client, &path).await?;

    if res.status() == 404 {
        Err(NotFoundError::MenuNotFoundError.into())
    } else {
        let res = res.json::<MenuResponse>().await?;
        Ok(res)
    }
}

pub(super) async fn list_days(client: &Client, station_id: u64) -> Result<Vec<LocalDay>> {
    let res = raw_fetch_menu(client, station_id, 2021, 27, 2).await?;

    let days: Vec<LocalDay> = res
        .menu
        .weeks
        .into_iter()
        .flat_map(|week| week.days)
        .filter_map(|day| day.into_local_day())
        .collect();

    Ok(days)
}

pub(super) async fn query_station(client: &Client, station_id: u64) -> Result<DetailedStation> {
    let now = chrono::offset::Utc::now();

    let res = raw_fetch_menu(client, station_id, now.year(), now.iso_week().week(), 1).await?;

    Ok(res.menu.station)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_day() {
        let meals: Vec<Meal> = vec![Meal::new("Fisk Björkeby".to_owned(), vec![1, 2, 3, 5, 8])];

        assert_eq!(
            Day::new(meals.clone(), 2020, 1, 1)
                .into_local_day()
                .unwrap()
                .meals
                .len(),
            1
        );
        assert!(Day::new(meals, 2021, 2, 29).into_local_day().is_none());
    }

    #[actix_rt::test]
    async fn query() {
        let client = Client::new();
        let station = query_station(&client, 6362776414978048).await.unwrap();

        assert_eq!(station.station.name, "Information - Sandvikens kommun");
        assert!(station.to_local_menu().is_none());
    }
}
