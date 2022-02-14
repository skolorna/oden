use std::time::Instant;

use euphemism::{index::Index, search::Search};

fn main() {
    let recipes = include_str!("../../meals.txt").lines();
    let index_start = Instant::now();
    let mut index = Index::new();

    for recipe in recipes {
        index.insert(recipe);
    }

    println!("indexed in {:.02?}", index_start.elapsed());
    println!("index size: {}", index.documents().len());
    println!("no. words: {}", index.words().count());

    let search = Search::new(&index, "pasta med köttfärssås");

    let hits = search.execute();

    for _hit in hits {
        // print!("+");
    }
}
