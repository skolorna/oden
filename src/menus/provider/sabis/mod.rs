use scraper::{Html, Selector};
use lazy_static::lazy_static;
use url::Url;

use crate::errors::Result;
use crate::menus::Menu;
use crate::menus::id::MenuID;
use crate::menus::provider::Provider;
use crate::util::last_path_segment;

lazy_static! {
	static ref S_RESTAURANT_ANCHOR: Selector = Selector::parse("li.restaurant-list-item a").unwrap();
}

pub async fn list_menus() -> Result<Vec<Menu>> {
    let html = reqwest::get("https://www.sabis.se/restauranger/").await?.text().await?;
		let doc = Html::parse_document(&html);

		let menus = doc.select(&S_RESTAURANT_ANCHOR).filter_map(|e| {
			let url = Url::parse(e.value().attr("href")?).ok()?;
			let title = e.text().collect::<_>();

			let local_id = last_path_segment(&url);

			debug_assert!(local_id.is_some());

			Some(Menu::new(
					MenuID::new(Provider::Sabis, local_id?.into()),
					title,
			))
		}).collect::<Vec<_>>();

		Ok(menus)
}

#[cfg(test)]
mod tests {
use super::*;

	#[actix_rt::test]
	async fn test_list_menus() {
		let menus = list_menus().await.unwrap();

		dbg!(&menus);
		assert!(menus.len() > 15);
	}
}
