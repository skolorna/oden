use std::{
    borrow::Cow,
    collections::{BTreeSet, HashMap},
    time::Instant, any::Any,
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
    /// Construct a new search query.
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
        let mut terms = BTreeSet::new();
        let mut docids = Vec::new();
        let words_fst = self.index.words_fst();

        for token in analyze(self.query).filter(|t| t.is_word()) {
            let derivations = word_derivations(token.text(), words_fst);

            terms.extend(derivations.into_iter());
        }

        debug!(
            "derived {} terms after {:.02?}",
            terms.len(),
            before.elapsed()
        );

        for term in terms {
            println!("{:?}", term);

            if let Some(docs) = self.index.word_docids.get(&term.word) {
                docids.extend(docs.iter().map(|d| (*d, term.typos)));
            }
        }

        let mut docid_typos = HashMap::<DocId, usize>::new();

        dbg!(docids.len());

        for (doc, typos) in docids {
            *docid_typos.entry(doc).or_default() += typos as usize;
        }

        let mut doc_frequencies: Vec<_> = docid_typos.into_iter().collect();
        let i = doc_frequencies.len().saturating_sub(self.limit + 1);

        let candidates = if i >= doc_frequencies.len() {
            doc_frequencies.as_mut()
        } else {
            let (_, _, top) = doc_frequencies.select_nth_unstable_by_key(i, |(_id, freq)| *freq);

            top
        };

        candidates.sort_by_key(|(_id, freq)| *freq);

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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct DerivedWord {
    word: String,
    typos: u8,
}

pub fn word_derivations(query: &str, words_fst: &fst::Set<Cow<'_, [u8]>>) -> Vec<DerivedWord> {
    let dfa = LEV_BUILDER.build_dfa(query);
    let mut stream = words_fst.search_with_state(&dfa).into_stream();
    let mut derived_words = Vec::new();

    while let Some((word, state)) = stream.next() {
        let word = match std::str::from_utf8(word) {
            Ok(w) => w,
            Err(_e) => continue,
        };
        let typos = dfa.distance(state).to_u8();
        let char_count = word.chars().count();

        if typos > 1 && char_count <= 8 {
            continue;
        }

        if typos > 0 && char_count <= 4 {
            continue;
        }

        derived_words.push(DerivedWord {
            word: word.to_owned(),
            typos,
        });
    }

    derived_words
}
