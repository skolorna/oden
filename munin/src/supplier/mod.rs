pub mod kleins;
pub mod matilda;
pub mod mpi;
pub mod skolmaten;
pub mod sodexo;

// impl Supplier {
//     #[must_use]
//     pub fn id(&self) -> String {
//         self.to_string()
//     }

//     #[must_use]
//     pub fn name(&self) -> String {
//         match *self {
//             Supplier::Skolmaten => "Skolmaten",
//             Supplier::Sodexo => "Sodexo",
//             Supplier::Mpi => "MPI",
//             Supplier::Kleins => "Klein's Kitchen",
//             Supplier::Sabis => "Sabis",
//             Supplier::Matilda => "Matilda",
//         }
//         .to_owned()
//     }

//     #[must_use]
//     pub fn info(&self) -> Info {
//         Info {
//             name: self.name(),
//             id: self.id(),
//         }
//     }

//     #[instrument(err, skip(client))]
//     pub async fn list_menus(&self, client: &Client) -> Result<Vec<Menu>> {
//         use Supplier::{Kleins, Matilda, Mpi, Sabis, Skolmaten, Sodexo};

//         debug!("listing menus");

//         match *self {
//             Skolmaten => skolmaten::list_menus(client).await,
//             Sodexo => sodexo::list_menus(client).await,
//             Mpi => mpi::list_menus(client).await,
//             Kleins => kleins::list_menus(client).await,
//             Sabis => sabis::list_menus().await,
//             Matilda => matilda::list_menus(client).await,
//         }
//     }

//     #[instrument(err, skip(client))]
//     pub async fn list_days(
//         &self,
//         client: &Client,
//         menu_slug: &str,
//         first: NaiveDate,
//         last: NaiveDate,
//     ) -> Result<Vec<Day>> {
//         use Supplier::{Kleins, Matilda, Mpi, Sabis, Skolmaten, Sodexo};

//         debug!("listing days");

//         match *self {
//             Skolmaten => {
//                 skolmaten::list_days(
//                     client,
//                     menu_slug.parse().map_err(|_| Error::InvalidMenuSlug)?,
//                     first,
//                     last,
//                 )
//                 .await
//             }
//             Sodexo => sodexo::list_days(client, menu_slug, first, last).await,
//             Mpi => mpi::list_days(client, menu_slug, first, last).await,
//             Kleins => kleins::list_days(client, menu_slug, first, last).await,
//             Sabis => sabis::list_days(client, menu_slug, first, last).await,
//             Matilda => {
//                 matilda::list_days(
//                     client,
//                     &menu_slug.parse().map_err(|_| Error::InvalidMenuSlug)?,
//                     first,
//                     last,
//                 )
//                 .await
//             }
//         }
//     }
// }
