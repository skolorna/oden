use std::{collections::BTreeSet, time::Instant};

use levenshtein_automata::LevenshteinAutomatonBuilder as LevBuilder;
use once_cell::sync::Lazy;

use crate::{
    index::{DocId, Index},
    tokenizer::analyze,
};

pub struct Search<'a, T> {
    query: &'a str,
    index: &'a Index<T>,
}

impl<'a, T> Search<'a, T>
where
    T: ToString + std::fmt::Debug,
{
    pub fn new(index: &'a Index<T>, query: &'a str) -> Self {
        Self { query, index }
    }

    pub fn execute(&self) -> Vec<&'a T> {
        let before = Instant::now();
        let mut words = BTreeSet::new();
        let mut doc_ids = BTreeSet::<DocId>::new();

        for token in analyze(self.query) {
            let derivations = word_derivations(&token.word(), self.index.words());

            for derivation in derivations {
                words.insert(derivation.word.to_string());
            }
        }

        for word in words {
            if let Some(docs) = self.index.inverted_index.get(&word) {
                doc_ids.extend(docs.iter());
            }
        }

        let docs = doc_ids
            .iter()
            .filter_map(|id| self.index.get_doc(id))
            .collect();

        println!("search took {:.02?}", before.elapsed());

        docs
    }
}

const MAX_TYPOS: u8 = 2;

static LEV_BUILDER: Lazy<LevBuilder> = Lazy::new(|| LevBuilder::new(MAX_TYPOS, true));

#[derive(Debug)]
pub struct DerivedWord<'a> {
    word: &'a str,
    distance: u8,
}

pub fn word_derivations<'a>(
    query: &str,
    words: impl Iterator<Item = &'a str>,
) -> impl Iterator<Item = DerivedWord<'a>> {
    let dfa = LEV_BUILDER.build_dfa(query);

    words.filter_map(move |word| {
        let mut state = dfa.initial_state();
        for &b in word.as_bytes() {
            state = dfa.transition(state, b);
        }
        let distance = dfa.distance(state).to_u8();

        if distance > MAX_TYPOS {
            return None;
        }

        if distance > 1 && word.chars().count() <= 8 {
            return None;
        }

        if distance > 0 && word.chars().count() <= 4 {
            return None;
        }

        Some(DerivedWord { word, distance })
    })
}
