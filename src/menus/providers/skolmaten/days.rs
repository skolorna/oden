use chrono::NaiveDate;
use reqwest::Client;
use serde::Deserialize;

use crate::menus::{LocalDay, LocalMeal};

use super::{fetch::fetch_json, Station};

#[derive(Deserialize, Debug, Clone)]
pub(super) struct Meal {
    value: String,
    attributes: Vec<u32>,
}

impl Meal {
    fn new(value: String, attributes: Vec<u32>) -> Self {
        Self {
            value,
            attributes,
        }
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
    meals: Vec<Meal>,
}

impl Day {
    fn new(meals: Vec<Meal>, year: i32, month: u32, day: u32) -> Self {
        Self {
            year,
            month,
            day,
            meals,
        }
    }

    /// Maps `NaiveDate::from_ymd_opt` and creates a LocalDay; thus, `None` is returned on invalid dates such as *February 29, 2021*.
    fn into_local_day(self) -> Option<LocalDay> {
        NaiveDate::from_ymd_opt(self.year, self.month, self.day).map(|date| LocalDay {
            date,
            meals: self.meals.into_iter().map(|meal| meal.into()).collect(),
        })
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Week {
    year: u32,
    week_of_year: u8,
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
    station: Station,
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
    year: u32,
    week_of_year: u8,
    count: u8,
) -> reqwest::Result<MenuResponse> {
    let path = format!(
        "menu?station={}&year={}&weekOfYear={}&count={}",
        station_id, year, week_of_year, count
    );

    let res = fetch_json::<MenuResponse>(client, &path).await?;

    Ok(res)
}

pub(super) async fn list_days(client: &Client, station_id: u64) -> reqwest::Result<Vec<LocalDay>> {
    let res = raw_fetch_menu(client, station_id, 2021, 30, 2).await?;

    let days: Vec<LocalDay> = res
        .menu
        .weeks
        .into_iter()
        .flat_map(|week| week.days)
        .filter_map(|day| day.into_local_day())
        .collect();

    Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_day() {
        let meals: Vec<Meal> = vec![Meal::new("Fisk Björkeby".to_owned(), vec![1, 2, 3, 5, 8])];

        assert_eq!(Day::new(meals.clone(), 2020, 1, 1).into_local_day().unwrap().meals.len(), 1);
        assert!(Day::new(meals, 2021, 2, 29).into_local_day().is_none());
    }
}
