use std::{borrow::Cow, fmt::Display, str::FromStr};

use chrono::{Datelike, IsoWeek, NaiveDate, TimeZone, Utc, Weekday};
use chrono_tz::Europe::Stockholm;
use futures::{stream, StreamExt};
use reqwest::Client;
use select::{
    document::Document,
    node::Node,
    predicate::{Attr, Class, Name, Predicate},
};
use serde::{Deserialize, Serialize};
use tracing::{error, instrument, trace};

use crate::{
    errors::{MuninError, MuninResult},
    util::parse_weekday,
    Day, Meal, Menu, MenuSlug,
};

use super::Supplier;

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

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_urlencoded::from_str(s)
    }
}

#[allow(clippy::module_name_repetitions)]
trait MatildaMenu {
    fn query(&self) -> MenuQuery;

    fn title(&self) -> Cow<str>;
}

impl<T: MatildaMenu> From<T> for Menu {
    fn from(i: T) -> Self {
        Self {
            slug: MenuSlug {
                supplier: Supplier::Matilda,
                local_id: i.query().to_string(),
            },
            title: i.title().into(),
        }
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
async fn get_doc<T: Serialize + std::fmt::Debug>(
    client: &Client,
    query: &T,
) -> MuninResult<Document> {
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
async fn list_regions(client: &Client) -> MuninResult<Vec<Region>> {
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
) -> MuninResult<Vec<Municipality<'r>>> {
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
) -> MuninResult<Vec<Part<'m>>> {
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
async fn list_customers<'p>(client: &Client, part: &'p Part<'p>) -> MuninResult<Vec<Customer<'p>>> {
    let doc = get_doc(client, part).await?;
    let customers = scrape_options(&doc, "CustomerList")
        .map(|(id, name)| Customer { id, part, name })
        .collect();

    Ok(customers)
}

const CONCURRENT_REQUESTS: usize = 16;

#[instrument]
async fn menus_in_part<'p>(client: &Client, part: Part<'p>) -> MuninResult<Vec<Menu>> {
    let customers = list_customers(client, &part).await?;

    if customers.is_empty() {
        Ok(vec![part.into()])
    } else {
        Ok(customers.into_iter().map(Into::into).collect())
    }
}

#[instrument(skip(client))]
async fn menus_in_municipality<'m>(
    client: &Client,
    municipality: &'m Municipality<'m>,
) -> MuninResult<Vec<Menu>> {
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
async fn menus_in_region(client: &Client, region: &Region) -> MuninResult<Vec<Menu>> {
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

#[instrument]
pub async fn list_menus() -> MuninResult<Vec<Menu>> {
    let client = Client::new();
    let mut menus = vec![];

    let regions = list_regions(&client).await?;

    let mut menus_stream = stream::iter(regions.iter())
        .map(|region| menus_in_region(&client, region))
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

fn parse_day_node_opt(node: &Node, year: i32, week_num: u32) -> Option<Day> {
    let meals = node
        .find(Class("meal-text"))
        .filter_map(|n| Meal::from_str(&n.text()).ok())
        .collect();

    let date = node.find(Class("date-container")).next()?.text();
    let mut date_parts = date.split_whitespace();
    let weekday = date_parts.next().and_then(parse_weekday)?;
    let date = NaiveDate::from_isoywd_opt(year, week_num, weekday)?;

    Day::new_opt(date, meals)
}

async fn days_by_week(
    client: &Client,
    menu: &MenuQuery,
    week_offset: i64,
) -> MuninResult<Vec<Day>> {
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
        .ok_or(MuninError::ScrapeError {
            context: "couldn't get year".into(),
        })?;
    let week_num = doc
        .find(Attr("id", "WeekPageWeekNo"))
        .next()
        .and_then(|n| n.attr("value")?.parse().ok())
        .ok_or(MuninError::ScrapeError {
            context: "couldn't get week number".into(),
        })?;

    let days = doc
        .find(Name("li").and(Class("li-menu")))
        .filter_map(|n| parse_day_node_opt(&n, year, week_num))
        .collect();

    Ok(days)
}

/// List days.
#[instrument]
pub async fn list_days(
    menu: &MenuQuery,
    first: NaiveDate,
    last: NaiveDate,
) -> MuninResult<Vec<Day>> {
    let utc = Utc::now().naive_utc();
    let now = Stockholm.from_utc_datetime(&utc).date().naive_local();

    let client = Client::new();

    let offsets = stream::iter(week_offsets(
        now.iso_week(),
        first.iso_week(),
        last.iso_week(),
    ));
    let mut days_stream = offsets
        .map(|o| days_by_week(&client, menu, o))
        .buffer_unordered(CONCURRENT_REQUESTS);

    let mut days = vec![];

    while let Some(result) = days_stream.next().await {
        days.append(&mut result?);
    }

    Ok(days)
}

/// Calculate week offsets.
pub(super) fn week_offsets(
    around: IsoWeek,
    first: IsoWeek,
    last: IsoWeek,
) -> impl Iterator<Item = i64> {
    let first = NaiveDate::from_isoywd(first.year(), first.week(), Weekday::Mon);
    let last = NaiveDate::from_isoywd(last.year(), last.week(), Weekday::Mon);
    let around = NaiveDate::from_isoywd(around.year(), around.week(), Weekday::Mon);

    first
        .iter_weeks()
        .take_while(move |d| *d <= last)
        .map(move |d| (d - around).num_weeks())
}

#[cfg(test)]
mod tests {

    use chrono::{Datelike, Duration, NaiveDate, Utc};

    use crate::menus::supplier::matilda::{list_days, week_offsets, MenuQuery};

    use super::list_menus;

    #[tokio::test]
    async fn should_list_menus() {
        let menus = list_menus().await.unwrap();

        assert!(menus.len() > 500);
    }

    #[tokio::test]
    async fn should_list_days() {
        let menu = MenuQuery {
            customer: Some(10242),
            part: 1594,
            municipality: 2161,
            region: 21,
        };
        let first = Utc::now() - Duration::weeks(1);

        let days = list_days(
            &menu,
            first.date().naive_utc(),
            (first + Duration::days(30)).date().naive_local(),
        )
        .await
        .unwrap();

        assert!(days.len() >= 7);
    }

    #[test]
    fn calc_week_offsets() {
        let around = NaiveDate::from_ymd(2022, 1, 4).iso_week();
        let first = NaiveDate::from_ymd(2021, 12, 12).iso_week();
        let last = NaiveDate::from_ymd(2022, 1, 12).iso_week();

        let offsets: Vec<i64> = week_offsets(around, first, last).collect();

        assert_eq!(offsets, &[-4, -3, -2, -1, 0, 1]);
    }
}
