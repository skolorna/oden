use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet, HashMap},
    time::Instant,
};

use tracing::{debug, instrument};

use crate::tokenizer::analyze;

pub type DocId = u32;
pub type Word = String;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Suggestion {
    distance: usize,
    value: Word,
}

pub struct Index<'a, T> {
    pub inverted_index: BTreeMap<Word, Vec<DocId>>,
    pub documents: HashMap<DocId, T>,
    pub words_fst: fst::Set<Cow<'a, [u8]>>,
}

impl<T> std::fmt::Debug for Index<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Index")
            // .field("inverted_index", &self.inverted_index)
            // .field("documents", &self.documents)
            // .field("words_fst", &self.words_fst)
            .field("num_docs", &self.documents().len())
            .field("dict_size", &self.words_fst().len())
            .finish()
    }
}

/// An immutable index.
#[allow(clippy::new_without_default)]
impl<'a, T> Index<'a, T> {
    pub fn documents(&self) -> &HashMap<DocId, T> {
        &self.documents
    }

    pub fn get_doc(&self, id: &DocId) -> Option<&T> {
        self.documents.get(id)
    }

    pub fn words_fst(&self) -> &fst::Set<Cow<'a, [u8]>> {
        &self.words_fst
    }
}

pub struct IndexBuilder<T> {
    pub documents: Vec<T>,
}

impl<T> std::fmt::Debug for IndexBuilder<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IndexBuilder")
            .field("num_docs", &self.documents.len())
            .finish()
    }
}

impl<T: ToString> IndexBuilder<T> {
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    pub fn push(&mut self, doc: T) {
        self.documents.push(doc);
    }

    #[instrument(level = "debug")]
    pub fn build(self) -> Index<'static, T> {
        let before = Instant::now();
        let mut inverted_index = BTreeMap::<Word, Vec<DocId>>::new();
        let mut documents = HashMap::new();

        for (id, doc) in self.documents.into_iter().enumerate() {
            let id = id as u32;
            let doc_text = doc.to_string();
            let tokens = analyze(&doc_text);
            let mut words = BTreeSet::new();

            for token in tokens {
                words.insert(token.word().to_string());
            }

            for word in words {
                let entry = inverted_index.entry(word.to_string()).or_default();

                entry.push(id);
            }

            documents.insert(id, doc);
        }

        let words_fst = fst::Set::from_iter(inverted_index.keys().map(|k| k.as_str()))
            .unwrap()
            .map_data(Cow::Owned)
            .unwrap();

        debug!("built index in {:.02?}", before.elapsed());

        Index {
            inverted_index,
            documents,
            words_fst,
        }
    }
}
