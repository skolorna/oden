use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
    time::Instant,
};

use fst::{IntoStreamer, Streamer};
use levenshtein_automata::LevenshteinAutomatonBuilder as LevBuilder;
use once_cell::sync::Lazy;
use tracing::{debug, instrument};

use crate::{
    index::{DocId, Index},
    tokenizer::analyze,
};

#[derive(Debug)]
pub struct Search<'a, T> {
    query: &'a str,
    limit: usize,
    index: &'a Index<'a, T>,
}

impl<'a, T> Search<'a, T>
where
    T: ToString + std::fmt::Debug,
{
    pub fn new(index: &'a Index<T>, query: &'a str, limit: usize) -> Self {
        Self {
            query,
            index,
            limit,
        }
    }

    #[instrument(level = "debug")]
    pub fn execute(&self) -> Vec<&'a T> {
        let before = Instant::now();
        let mut words = BTreeSet::new();
        let mut doc_ids = Vec::new();
        let words_fst = self.index.words_fst();

        for token in analyze(self.query).filter(|t| t.is_word()) {
            let derivations = word_derivations(&token.word(), words_fst);

            words.extend(derivations.into_iter().map(|d| d.word));
        }

        debug!(
            "derived {} words after {:.02?}",
            words.len(),
            before.elapsed()
        );

        for word in words {
            println!("{}", word);

            if let Some(docs) = self.index.inverted_index.get(&word) {
                doc_ids.extend(docs.iter());
            }
        }

        let mut doc_frequencies = HashMap::<DocId, usize>::new();

        for doc in doc_ids {
            *doc_frequencies.entry(doc).or_default() += 1;
        }

        let mut doc_frequencies: Vec<_> = doc_frequencies.into_iter().collect();
        let i = doc_frequencies.len().saturating_sub(self.limit + 1);

        let candidates = if i >= doc_frequencies.len() {
            &mut []
        } else {
            let (_, _, top) = doc_frequencies.select_nth_unstable_by_key(i, |(_id, freq)| *freq);

            top
        };

        debug!(
            "found {} candidates after {:.02?}",
            candidates.len(),
            before.elapsed()
        );

        let docs = candidates
            .iter_mut()
            .filter_map(|(id, _freq)| self.index.get_doc(id))
            .collect();

        debug!("search took {:.02?}", before.elapsed());

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

pub fn word_derivations(query: &str, words_fst: &fst::Set<Cow<'_, [u8]>>) -> Vec<DerivedWord> {
    let dfa = LEV_BUILDER.build_dfa(query);
    let mut stream = words_fst.search_with_state(&dfa).into_stream();
    let mut derived_words = Vec::new();

    while let Some((word, state)) = stream.next() {
        let word = std::str::from_utf8(word).unwrap();
        let distance = dfa.distance(state).to_u8();
        let char_count = word.chars().count();

        if distance > 1 && char_count <= 8 {
            continue;
        }

        if distance > 0 && char_count <= 4 {
            continue;
        }

        derived_words.push(DerivedWord {
            word: word.to_owned(),
            distance,
        });
    }

    derived_words
}
