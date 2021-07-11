use chrono::NaiveDate;

use crate::errors::Result;

use super::{day::Day, mashie, provider::Provider, Menu};

const HOST: &str = "https://sodexo.mashie.com";

pub async fn list_menus() -> Result<Vec<Menu>> {
    let menus = mashie::list_menus(HOST)
        .await?
        .into_iter()
        .map(|m| m.into_menu(Provider::Sodexo))
        .collect();

    Ok(menus)
}

pub async fn query_menu(menu_id: &str) -> Result<Menu> {
    let menu = mashie::query_menu(HOST, menu_id)
        .await?
        .into_menu(Provider::Sodexo);

    Ok(menu)
}

pub async fn list_days(menu_id: &str, first: NaiveDate, last: NaiveDate) -> Result<Vec<Day>> {
    mashie::list_days(HOST, menu_id, first, last).await
}
