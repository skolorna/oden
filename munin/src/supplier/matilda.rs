use std::{borrow::Cow, fmt::Display, iter, str::FromStr};

use futures::{stream, StreamExt};
use reqwest::Client;
use select::{
    document::Document,
    node::Node,
    predicate::{Attr, Class, Name, Predicate},
};
use serde::{Deserialize, Serialize};
use stor::{menu::Supplier, Day, Meal, Menu};
use time::{Date, Duration, OffsetDateTime, Weekday};
use time_tz::OffsetDateTimeExt;
use tracing::{error, instrument, trace};

use crate::{util::parse_weekday, Error, Result};

use super::ListDays;

#[derive(Debug, Clone, Serialize)]
struct Region {
    #[serde(rename = "r")]
    id: u32,

    #[allow(dead_code)]
    #[serde(skip)]
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct Municipality<'r> {
    #[serde(rename = "m")]
    id: u32,

    #[serde(flatten)]
    region: &'r Region,

    #[allow(dead_code)]
    #[serde(skip)]
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct Part<'m> {
    #[serde(rename = "p")]
    id: u32,

    #[serde(flatten)]
    municipality: &'m Municipality<'m>,

    #[serde(skip)]
    name: String,
}

#[derive(Debug, Clone, Serialize)]
struct Customer<'p> {
    #[serde(rename = "c")]
    id: u32,

    #[serde(flatten)]
    part: &'p Part<'p>,

    #[serde(skip)]
    name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuQuery {
    #[serde(rename = "c", skip_serializing_if = "Option::is_none")]
    pub customer: Option<u32>,

    #[serde(rename = "p")]
    pub part: u32,

    #[serde(rename = "m")]
    pub municipality: u32,

    #[serde(rename = "r")]
    pub region: u32,
}

impl Display for MenuQuery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            serde_urlencoded::to_string(self).map_err(|e| {
                error!("{:?}", e);
                std::fmt::Error
            })?
        )
    }
}

impl FromStr for MenuQuery {
    type Err = serde::de::value::Error;

    fn from_str(s: &str) -> core::result::Result<Self, Self::Err> {
        serde_urlencoded::from_str(s)
    }
}

#[allow(clippy::module_name_repetitions)]
trait MatildaMenu {
    fn query(&self) -> MenuQuery;

    fn title(&self) -> Cow<str>;

    fn to_menu(&self) -> stor::Menu {
        Menu::from_supplier(Supplier::Matilda, self.query().to_string(), self.title())
    }
}

impl MatildaMenu for Part<'_> {
    fn title(&self) -> Cow<'_, str> {
        Cow::Borrowed(&self.name)
    }

    fn query(&self) -> MenuQuery {
        MenuQuery {
            customer: None,
            part: self.id,
            municipality: self.municipality.id,
            region: self.municipality.region.id,
        }
    }
}

impl MatildaMenu for Customer<'_> {
    fn title(&self) -> Cow<'_, str> {
        format!("{} ({})", self.name, self.part.name).into()
    }

    fn query(&self) -> MenuQuery {
        MenuQuery {
            customer: Some(self.id),
            part: self.part.id,
            municipality: self.part.municipality.id,
            region: self.part.municipality.region.id,
        }
    }
}

#[instrument(skip(client))]
async fn get_doc<T: Serialize + std::fmt::Debug>(client: &Client, query: &T) -> Result<Document> {
    let res = client
        .get("https://webmenu.foodit.se/")
        .query(query)
        .send()
        .await?;
    trace!("GET {}", res.url());
    let html = res.text().await?;
    Ok(Document::from(html.as_str()))
}

fn scrape_options<'a>(
    doc: &'a Document,
    select_id: &'a str,
) -> impl Iterator<Item = (u32, String)> + 'a {
    doc.find(Attr("id", select_id).child(Name("option")))
        .filter_map(|n| {
            let value = n.attr("value")?.parse().ok()?;
            let label = n.text();

            Some((value, label))
        })
}

#[instrument(skip(client))]
async fn list_regions(client: &Client) -> Result<Vec<Region>> {
    #[derive(Debug, Serialize)]
    struct Query {}

    let doc = get_doc(client, &Query {}).await?;
    let regions = scrape_options(&doc, "RegionList")
        .map(|(id, name)| Region { id, name })
        .collect();

    Ok(regions)
}

#[instrument(skip(client))]
async fn list_municipalities<'r>(
    client: &Client,
    region: &'r Region,
) -> Result<Vec<Municipality<'r>>> {
    let doc = get_doc(client, &region).await?;
    let municipalities = scrape_options(&doc, "MunicipalityList")
        .map(|(id, name)| Municipality { id, region, name })
        .collect();

    Ok(municipalities)
}

#[instrument(skip(client))]
async fn list_parts<'m>(
    client: &Client,
    municipality: &'m Municipality<'m>,
) -> Result<Vec<Part<'m>>> {
    let doc = get_doc(client, municipality).await?;
    let parts = scrape_options(&doc, "PartList")
        .map(|(id, name)| Part {
            id,
            municipality,
            name,
        })
        .collect();

    Ok(parts)
}

#[instrument(skip(client))]
async fn list_customers<'p>(client: &Client, part: &'p Part<'p>) -> Result<Vec<Customer<'p>>> {
    let doc = get_doc(client, part).await?;
    let customers = scrape_options(&doc, "CustomerList")
        .map(|(id, name)| Customer { id, part, name })
        .collect();

    Ok(customers)
}

const CONCURRENT_REQUESTS: usize = 16;

#[instrument(skip(client))]
async fn menus_in_part<'p>(client: &Client, part: Part<'p>) -> Result<Vec<Menu>> {
    let customers = list_customers(client, &part).await?;

    if customers.is_empty() {
        Ok(vec![part.to_menu()])
    } else {
        Ok(customers.iter().map(MatildaMenu::to_menu).collect())
    }
}

#[instrument(skip(client))]
async fn menus_in_municipality<'m>(
    client: &Client,
    municipality: &'m Municipality<'m>,
) -> Result<Vec<Menu>> {
    let mut menus = vec![];
    let parts = list_parts(client, municipality).await?;

    let mut menus_stream = stream::iter(parts)
        .map(|p| menus_in_part(client, p))
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some(result) = menus_stream.next().await {
        menus.append(&mut result?);
    }

    Ok(menus)
}

#[instrument(skip(client))]
async fn menus_in_region(client: &Client, region: &Region) -> Result<Vec<Menu>> {
    let mut menus = vec![];
    let municipalities = list_municipalities(client, region).await?;

    let mut parts_stream = stream::iter(municipalities.iter())
        .map(|m| menus_in_municipality(client, m))
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some(result) = parts_stream.next().await {
        menus.append(&mut result?);
    }

    Ok(menus)
}

#[instrument(err)]
pub async fn list_menus(client: &Client) -> Result<Vec<Menu>> {
    let mut menus = vec![];

    let regions = list_regions(client).await?;

    let mut menus_stream = stream::iter(regions.iter())
        .map(|region| menus_in_region(client, region))
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some(result) = menus_stream.next().await {
        menus.append(&mut result?);
    }

    Ok(menus)
}

#[derive(Debug, Serialize)]
enum View {
    Week,
}

#[derive(Debug, Serialize)]
struct ListDaysQuery<'m> {
    #[serde(flatten)]
    menu: &'m MenuQuery,

    #[serde(rename = "v")]
    view: View,

    #[serde(rename = "w")]
    week_offset: i64,
}

fn parse_day_node_opt(node: &Node, year: i32, week_num: u8) -> Option<Day> {
    let meals = node
        .find(Class("meal-text"))
        .filter_map(|n| Meal::from_str(&n.text()).ok())
        .collect();

    let date = node.find(Class("date-container")).next()?.text();
    let mut date_parts = date.split_whitespace();
    let weekday = date_parts.next().and_then(parse_weekday)?;
    let date = Date::from_iso_week_date(year, week_num, weekday).ok()?;

    Day::new(date, meals)
}

#[instrument(skip(client))]
async fn days_by_week(client: &Client, menu: &MenuQuery, week_offset: i64) -> Result<Vec<Day>> {
    let q = ListDaysQuery {
        menu,
        view: View::Week,
        week_offset,
    };

    let doc = get_doc(client, &q).await?;

    let year = doc
        .find(Attr("id", "Year"))
        .next()
        .and_then(|n| n.attr("value")?.parse().ok())
        .ok_or_else(|| Error::scrape_error("couldn't get year"))?;
    let week_num = doc
        .find(Attr("id", "WeekPageWeekNo"))
        .next()
        .and_then(|n| n.attr("value")?.parse().ok())
        .ok_or_else(|| Error::scrape_error("couldn't get week number"))?;

    let days = doc
        .find(Name("li").and(Class("li-menu")))
        .filter_map(|n| parse_day_node_opt(&n, year, week_num))
        .collect();

    Ok(days)
}

/// List days.
#[instrument(fields(%first, %last))]
pub async fn list_days(
    client: &Client,
    menu: &MenuQuery,
    first: Date,
    last: Date,
) -> Result<ListDays> {
    let today = OffsetDateTime::now_utc().to_timezone(crate::TZ).date();

    let offsets = stream::iter(week_offsets(today, first, last).unwrap());
    let mut days_stream = offsets
        .map(|o| days_by_week(client, menu, o))
        .buffer_unordered(CONCURRENT_REQUESTS);

    let mut days = vec![];

    while let Some(result) = days_stream.next().await {
        days.append(&mut result?);
    }

    Ok(ListDays { menu: None, days })
}

fn rewind_to_weekday(mut date: Date, weekday: Weekday) -> Option<Date> {
    while date.weekday() != weekday {
        date = date.previous_day()?;
    }

    Some(date)
}

/// Calculate week offsets.
pub(super) fn week_offsets(
    around: Date,
    first: Date,
    last: Date,
) -> Option<impl Iterator<Item = i64>> {
    let mut first = rewind_to_weekday(first, Weekday::Monday)?;
    let last = rewind_to_weekday(last, Weekday::Monday)?;
    let around = rewind_to_weekday(around, Weekday::Monday)?;

    Some(iter::from_fn(move || {
        let this = first;
        first += Duration::weeks(1);

        if this <= last {
            Some((this - around).whole_weeks())
        } else {
            None
        }
    }))
}

#[cfg(test)]
mod tests {
    use reqwest::Client;
    use time::{macros::date, Duration, OffsetDateTime};

    use crate::supplier::ListDays;

    use super::MenuQuery;

    #[tokio::test]
    async fn list_menus() {
        let menus = super::list_menus(&Client::new()).await.unwrap();

        assert!(menus.len() > 500);
    }

    #[tokio::test]
    async fn list_days() {
        let menu = MenuQuery {
            customer: Some(10242),
            part: 1594,
            municipality: 2161,
            region: 21,
        };
        let first = OffsetDateTime::now_utc() - Duration::weeks(1);

        let ListDays { days, .. } = super::list_days(
            &Client::new(),
            &menu,
            first.date(),
            (first + Duration::days(30)).date(),
        )
        .await
        .unwrap();

        assert!(days.len() >= 7);
    }

    #[test]
    fn calc_week_offsets() {
        let around = date!(2022 - 01 - 04);
        let first = date!(2021 - 12 - 12);
        let last = date!(2022 - 01 - 12);

        let offsets: Vec<i64> = super::week_offsets(around, first, last).unwrap().collect();

        assert_eq!(offsets, &[-4, -3, -2, -1, 0, 1]);
    }
}
