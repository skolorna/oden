use std::{borrow::Cow, collections::BTreeSet, time::Instant};

use fst::{IntoStreamer, Streamer};
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
        let words_fst = self.index.words();

        println!("build fst in {:.02?}", before.elapsed());

        for token in analyze(self.query) {
            let derivations = word_derivations(&token.word(), &words_fst);

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
pub struct DerivedWord {
    word: String,
    distance: u8,
}

pub fn word_derivations(query: &str, words: &fst::Set<Cow<'_, [u8]>>) -> Vec<DerivedWord> {
    let dfa = LEV_BUILDER.build_dfa(query);
    let mut stream = words.search_with_state(&dfa).into_stream();
    let mut derived_words = Vec::new();

    while let Some((word, state)) = stream.next() {
        let word = std::str::from_utf8(word).unwrap();
        let distance = dfa.distance(state).to_u8();
        // println!("{}", distance);
        derived_words.push(DerivedWord {
            word: word.to_string(),
            distance,
        });
    }

    derived_words

    // words.filter_map(move |word| {
    //     let mut state = dfa.initial_state();
    //     for &b in word.as_bytes() {
    //         state = dfa.transition(state, b);
    //     }
    //     let distance = dfa.distance(state).to_u8();

    //     if distance > MAX_TYPOS {
    //         return None;
    //     }

    //     if distance > 1 && word.chars().count() <= 8 {
    //         return None;
    //     }

    //     if distance > 0 && word.chars().count() <= 4 {
    //         return None;
    //     }

    //     Some(DerivedWord { word, distance })
    // })
}
