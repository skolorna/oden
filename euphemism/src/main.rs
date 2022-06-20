use std::time::Instant;

use euphemism::{index::IndexBuilder, search::Search};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

fn main() {
    // env::set_var("RUST_LOG", "debug");
    tracing_subscriber::fmt::init();

    let meals = include_str!("../../meals.txt");
    let mut index_builder = IndexBuilder::new();

    for meal in meals.lines() {
        index_builder.push(meal);
    }

    let index = index_builder.build();

    let sample = meals.lines().take(1000).collect::<Vec<_>>();

    let before = Instant::now();

    let searches = sample
        .par_iter()
        .map(|meal| (meal, Search::new(&index, meal, 1).execute()))
        .collect::<Vec<_>>();

    for (meal, results) in searches {
        print!("{meal}\t");
        println!("{}", results[0]);
    }

    eprintln!("searched in {:?}", before.elapsed());
}
