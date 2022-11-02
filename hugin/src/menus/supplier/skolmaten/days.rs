use std::str::FromStr;

use chrono::{Datelike, NaiveDate};
use futures::stream::{self, StreamExt};
use reqwest::{Client, StatusCode};
use serde::Deserialize;
use tracing::{error, instrument};

use crate::{
    day::dedup_day_dates,
    errors::{Error, Result},
    menus::{Day, Meal},
};

use super::fetch::fetch;

#[derive(Deserialize, Debug, Clone)]
struct SkolmatenMeal {
    value: String,
}

impl SkolmatenMeal {
    fn normalize(self) -> Option<Meal> {
        Meal::from_str(&self.value).ok()
    }
}

#[derive(Deserialize, Debug)]
pub(super) struct SkolmatenDay {
    year: i32,
    month: u32,
    day: u32,
    meals: Option<Vec<SkolmatenMeal>>,
}

impl SkolmatenDay {
    /// Maps `NaiveDate::from_ymd_opt` and creates a [`Day`]; thus, `None`
    /// is returned on invalid dates such as *February 29, 2021*. Also,
    /// `None` is returned if `meals` is `None`.
    fn normalize(self) -> Option<Day> {
        let date = NaiveDate::from_ymd_opt(self.year, self.month, self.day)?;
        let meals: Vec<Meal> = self
            .meals?
            .into_iter()
            .filter_map(SkolmatenMeal::normalize)
            .collect();

        Day::new_opt(date, meals)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct Week {
    // year: i32,
    // week_of_year: u32,
    days: Vec<SkolmatenDay>,
}

// #[derive(Deserialize, Debug)]
// pub(super) struct Bulletin {
//     // text: String,
// }

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub(super) struct SkolmatenMenu {
    // is_feedback_allowed: bool,
    weeks: Vec<Week>,
    // station: DetailedStation,
    // id: u64,
    // bulletins: Vec<Bulletin>,
}

#[derive(Deserialize, Debug)]
pub(super) struct SkolmatenMenuResponse {
    menu: SkolmatenMenu,
}

impl SkolmatenMenuResponse {
    pub(crate) fn into_days_iter(self) -> impl Iterator<Item = Day> {
        self.menu
            .weeks
            .into_iter()
            .flat_map(|week| week.days)
            .filter_map(SkolmatenDay::normalize)
    }
}

#[derive(PartialEq, Debug)]
struct SkolmatenWeekSpan {
    year: i32,
    week_of_year: u32,
    count: u8,
}

#[instrument(skip(client))]
async fn raw_fetch_menu(
    client: &Client,
    station_id: u64,
    span: &SkolmatenWeekSpan,
) -> Result<SkolmatenMenuResponse> {
    let path = format!(
        "menu?station={}&year={}&weekOfYear={}&count={}",
        station_id, span.year, span.week_of_year, span.count
    );

    let res = fetch(client, &path).await?;
    let status = res.status();
    match res.json::<SkolmatenMenuResponse>().await {
        Ok(res) => Ok(res),
        Err(e) => {
            if status != StatusCode::NOT_FOUND {
                error!("{}", e);
            }

            Err(Error::MenuNotFound)
        }
    }
}

/// Generate a series of queries because the Skolmaten API cannot handle more than one year per request.
fn generate_week_spans(first: NaiveDate, last: NaiveDate) -> Vec<SkolmatenWeekSpan> {
    assert!(first <= last, "First must not be after last.");

    let mut spans: Vec<SkolmatenWeekSpan> = Vec::new();
    let mut segment_start = first;

    while last > segment_start {
        use std::convert::TryFrom;

        let year = segment_start.year();

        let segment_end = if last.year() == year {
            last
        } else {
            NaiveDate::from_ymd(year, 12, 31)
        };

        let diff = segment_end - segment_start;
        let weeks = u8::try_from((diff.num_days() + 6) / 7).unwrap(); // Round the number of weeks up, not down.

        let mut week_of_year = segment_start.iso_week().week();

        // skolmaten.se cannot handle week numbers above 52 for some reason.
        if week_of_year > 52 {
            week_of_year = 1;
        }

        spans.push(SkolmatenWeekSpan {
            year,
            week_of_year,
            count: weeks,
        });

        segment_start = NaiveDate::from_yo(year + 1, 1);
    }

    spans
}

/// List days of a particular Skolmaten menu.
#[instrument(skip(client), fields(%first, %last))]
pub(crate) async fn list_days(
    client: &Client,
    station_id: u64,
    first: NaiveDate,
    last: NaiveDate,
) -> Result<Vec<Day>> {
    let spans = generate_week_spans(first, last);

    let results = stream::iter(spans)
        .map(|span| {
            let client = &client;
            async move {
                raw_fetch_menu(client, station_id, &span)
                    .await
                    .map(SkolmatenMenuResponse::into_days_iter)
            }
        })
        .buffer_unordered(4)
        .collect::<Vec<_>>()
        .await;

    let days_2d = results.into_iter().collect::<Result<Vec<_>>>()?;

    let mut days = days_2d
        .into_iter()
        .flatten()
        .filter(|day| day.is_between(first, last))
        .collect::<Vec<_>>();

    days.sort();
    dedup_day_dates(&mut days);

    Ok(days)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn week_spans() {
        let week_53 = generate_week_spans(
            NaiveDate::from_ymd(2020, 8, 1),
            NaiveDate::from_ymd(2021, 3, 1),
        );
        assert_eq!(
            week_53,
            [
                SkolmatenWeekSpan {
                    year: 2020,
                    week_of_year: 31,
                    count: 22,
                },
                SkolmatenWeekSpan {
                    year: 2021,
                    week_of_year: 1, // Skolmaten.se doesn't care that 53 is the correct week number.
                    count: 9,
                }
            ]
        );
    }

    #[test]
    fn convert_day() {
        let meals: Vec<SkolmatenMeal> = vec![SkolmatenMeal {
            value: "Fisk Björkeby".to_owned(),
        }];

        assert_eq!(
            SkolmatenDay {
                meals: Some(meals.clone()),
                year: 2020,
                month: 1,
                day: 1,
            }
            .normalize()
            .unwrap()
            .meals()
            .len(),
            1
        );
        assert!(SkolmatenDay {
            meals: Some(meals),
            year: 2021,
            month: 2,
            day: 29
        }
        .normalize()
        .is_none());
    }
}
