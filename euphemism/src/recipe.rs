use reqwest::Client;
use select::{document::Document, node::Node, predicate::Name};
use thiserror::Error;

#[derive(Error, Debug)]
#[allow(clippy::module_name_repetitions)]
pub enum RecipeError {
    #[error("{0}")]
    HttpError(#[from] reqwest::Error),
}

#[derive(Debug)]
pub struct Recipe {
    title: String,
}

impl Recipe {
    fn from_node_checked(node: &Node) -> Option<Self> {
        let title = node.attr(":title")?;
        let title = serde_json::from_str(title).unwrap();

        // let attrs: HashMap<&str, &str> = node.attrs().collect();

        // dbg!(attrs);

        Some(Self { title })
    }
}

async fn get_doc(client: &Client, page: Option<u32>) -> Result<Document, RecipeError> {
    let html = client
        .get("https://recept.se/kategori/huvudratt")
        .query(&[("page", page)])
        .send()
        .await?
        .text()
        .await?;
    let doc = Document::from(html.as_str());

    Ok(doc)
}

async fn recipes_on_page(client: &Client, page: u32) -> Result<Vec<Recipe>, RecipeError> {
    let doc = get_doc(client, Some(page)).await?;
    let cards = doc.find(Name("recipe-card"));

    Ok(cards
        .filter_map(|n| Recipe::from_node_checked(&n))
        .collect())
}

/// Scrape recipes.
///
/// ```
/// use euphemism::recipe::scrape_recipes;
///
/// # tokio_test::block_on(async {
/// let recipes = scrape_recipes().await.unwrap();
///
/// println!("{:?}", &recipes);
///
/// assert!(recipes.len() > 100);
/// # })
/// ```
///
/// # Errors
///
/// Failing to connect or unexpected HTML will result in an error.
pub async fn scrape_recipes() -> Result<Vec<Recipe>, RecipeError> {
    let client = Client::new();
    let mut recipes: Vec<Recipe> = Vec::new();
    let mut page = 1;

    loop {
        // println!("scraping page {}", page);
        let mut res = recipes_on_page(&client, page).await?;

        // println!("{}", recipes.len());

        if res.is_empty() {
            for recipe in &recipes {
                println!("{}", recipe.title);
            }

            return Ok(recipes);
        }

        recipes.append(&mut res);
        page += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn single_page() {
        let recipes = recipes_on_page(&Client::new(), 1).await.unwrap();

        assert!(recipes.len() > 10, "{:?}", recipes);
    }

    #[tokio::test]
    async fn all_pages() {
        let recipes = scrape_recipes().await.unwrap();

        assert!(recipes.len() > 100, "{:?}", recipes);
    }
}
