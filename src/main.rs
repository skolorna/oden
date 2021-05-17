use crate::api::skolmaten::crawl::crawl_schools;

pub mod api;

fn main() {
    println!("Gathering schools ...");

    let schools = crawl_schools().unwrap();

    for school in schools {
        println!("{}", school.name);
    }
}
