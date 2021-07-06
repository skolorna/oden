use chrono::{Datelike, NaiveDate};
use reqwest::Client;
use serde::Deserialize;

use crate::{
    errors::{NotFoundError, Result},
    menus::{Day, Meal, Menu},
};

use super::{fetch::fetch, District, Station};

#[derive(Deserialize, Debug, Clone)]
pub(super) struct DetailedStation {
    district: District,
    #[serde(flatten)]
    station: Station,
}

impl DetailedStation {
    pub fn to_menu(&self) -> Option<Menu> {
        self.station.to_menu(&self.district.name)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SkolmatenMeal {
    pub value: String,
    pub attributes: Vec<u32>,
}

#[derive(Deserialize, Debug)]
pub(super) struct SkolmatenDay {
    year: i32,
    month: u32,
    day: u32,
    meals: Option<Vec<SkolmatenMeal>>,
}

impl SkolmatenDay {
    /// Maps `NaiveDate::from_ymd_opt` and creates a Day; thus, `None` is returned on invalid dates such as *February 29, 2021*. Also, `None` is returned if `meals` is `None`.
    fn into_day(self) -> Option<Day> {
        let date = NaiveDate::from_ymd_opt(self.year, self.month, self.day)?;
        let meals: Vec<Meal> = self.meals?.into_iter().map(|meal| meal.into()).collect();

        if meals.is_empty() {
            None
        } else {
            Some(Day { date, meals })
        }
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Week {
    year: i32,
    week_of_year: u32,
    days: Vec<SkolmatenDay>,
}

#[derive(Deserialize, Debug)]
pub(super) struct Bulletin {
    text: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct SkolmatenMenu {
    is_feedback_allowed: bool,
    weeks: Vec<Week>,
    station: DetailedStation,
    id: u64,
    bulletins: Vec<Bulletin>,
}

#[derive(Deserialize, Debug)]
pub(super) struct SkolmatenMenuResponse {
    menu: SkolmatenMenu,
}

async fn raw_fetch_menu(
    client: &Client,
    station_id: u64,
    year: i32,
    week_of_year: u32,
    count: u8,
) -> Result<SkolmatenMenuResponse> {
    let path = format!(
        "menu?station={}&year={}&weekOfYear={}&count={}",
        station_id, year, week_of_year, count
    );

    let res = fetch(client, &path).await?;

    if res.status() == 404 {
        Err(NotFoundError::MenuNotFoundError.into())
    } else {
        let res = res.json::<SkolmatenMenuResponse>().await?;
        Ok(res)
    }
}

pub(super) async fn list_days(client: &Client, station_id: u64) -> Result<Vec<Day>> {
    let res = raw_fetch_menu(client, station_id, 2021, 27, 2).await?;

    let days: Vec<Day> = res
        .menu
        .weeks
        .into_iter()
        .flat_map(|week| week.days)
        .filter_map(|day| day.into_day())
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
        let meals: Vec<SkolmatenMeal> = vec![SkolmatenMeal {
            value: "Fisk Björkeby".to_owned(),
            attributes: vec![1, 2, 3, 5, 8],
        }];

        assert_eq!(
            SkolmatenDay {
                meals: Some(meals.clone()),
                year: 2020,
                month: 1,
                day: 1,
            }
            .into_day()
            .unwrap()
            .meals
            .len(),
            1
        );
        assert!(SkolmatenDay {
            meals: Some(meals),
            year: 2021,
            month: 2,
            day: 29
        }
        .into_day()
        .is_none());
    }

    #[actix_rt::test]
    async fn query() {
        let client = Client::new();
        let station = query_station(&client, 6362776414978048).await.unwrap();

        assert_eq!(station.station.name, "Information - Sandvikens kommun");
        assert!(station.to_menu().is_none());
    }
}
