use std::env;

use euphemism::{index::IndexBuilder, search::Search};

fn main() {
    env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    let recipes = include_str!("../../meals.txt").lines();
    let mut index_builder = IndexBuilder::new();

    for recipe in recipes {
        index_builder.push(recipe);
    }

    let index = index_builder.build();

    let search = Search::new(&index, "Fisk Björkeby med kokt potatis", 100);
    // let search = Search::new(&index, "pasta med köttfärssås", 20);

    let hits = search.execute();

    for hit in hits {
        println!("{}", hit);
    }
}
