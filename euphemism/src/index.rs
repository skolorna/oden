use std::{
    borrow::Cow,
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet, HashMap},
    time::Instant,
};

use tracing::debug;

use crate::{position::extract_word_pair_proximities, tokenizer::analyze};

pub type DocId = u32;

pub struct Index<'a, T> {
    pub word_docids: HashMap<String, Vec<DocId>>,
    pub docid_words: HashMap<DocId, Vec<String>>,
    pub docid_word_pair_proximity: fst::Map<Vec<u8>>,
    pub documents: HashMap<DocId, T>,
    pub words_fst: fst::Set<Cow<'a, [u8]>>,
}

impl<T> std::fmt::Debug for Index<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Index")
            .field("num_docs", &self.docs().len())
            .field("dict_size", &self.words_fst().len())
            .finish()
    }
}

/// An immutable index.
#[allow(clippy::new_without_default)]
impl<'a, T> Index<'a, T> {
    #[must_use]
    pub fn docs(&self) -> &HashMap<DocId, T> {
        &self.documents
    }

    #[must_use]
    pub fn get_doc(&self, id: &DocId) -> Option<&T> {
        self.documents.get(id)
    }

    #[must_use]
    pub fn words_fst(&self) -> &fst::Set<Cow<'a, [u8]>> {
        &self.words_fst
    }

    #[must_use]
    pub fn word_pair_proximity(&self, doc: &DocId, a: &str, b: &str) -> Option<u64> {
        let key = if a > b {
            docid_word_pair_to_bytes(*doc, b, a)
        } else {
            docid_word_pair_to_bytes(*doc, a, b)
        };

        self.docid_word_pair_proximity.get(key).map(|v| v as _)
    }
}

/// Some kind of serialization for using FSTs with multiple levels of keys.
fn docid_word_pair_to_bytes(doc: DocId, a: &str, b: &str) -> Vec<u8> {
    const SEPARATOR: u8 = b' ';

    debug_assert!(a <= b);

    let mut bytes = Vec::with_capacity(std::mem::size_of::<DocId>() + a.len() + 1 + b.len());

    bytes.extend_from_slice(&doc.to_be_bytes());
    bytes.extend(a.as_bytes());
    bytes.push(SEPARATOR);
    bytes.extend(b.as_bytes());

    bytes
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct IndexBuilder<T> {
    pub documents: Vec<T>,
}

impl<T: ToString> Default for IndexBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[allow(clippy::new_without_default)]
impl<T: ToString> IndexBuilder<T> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            documents: Vec::new(),
        }
    }

    pub fn push(&mut self, doc: T) {
        self.documents.push(doc);
    }

    #[must_use]
    pub fn build(self) -> Index<'static, T> {
        let before = Instant::now();
        let mut word_docids = BTreeMap::<String, Vec<DocId>>::new(); // `BTreeSet` automatically sorts
        let mut docid_words = HashMap::<DocId, Vec<String>>::new();
        let mut docid_word_pair_proximity = BTreeMap::<Vec<u8>, u64>::new();
        let mut documents = HashMap::new();

        for (id, doc) in self.documents.into_iter().enumerate() {
            let id = id as u32;
            let doc_text = doc.to_string();
            let analyzed = analyze(&doc_text).collect::<Vec<_>>();
            let mut words = BTreeSet::new();

            let word_pair_proximities = extract_word_pair_proximities(analyzed.iter(), 16);

            for ((lword, rword), proximity) in word_pair_proximities {
                docid_word_pair_proximity
                    .insert(docid_word_pair_to_bytes(id, &lword, &rword), proximity as _);
            }

            for token in analyzed {
                words.insert(token.text().to_string());
            }

            for word in &words {
                word_docids.entry(word.clone()).or_default().push(id);
            }

            docid_words.insert(id, words.into_iter().collect());

            documents.insert(id, doc);
        }

        let mut freqs = word_docids
            .iter()
            .map(|(word, docs)| (word, docs.len()))
            .collect::<Vec<_>>();
        freqs.sort_by_key(|(_w, f)| Reverse(*f));
        for (w, f) in freqs
            .iter()
            .take(10)
            .map(|(w, f)| (w, *f as f32 / documents.len() as f32))
        {
            println!("\"{}\":\t{}", w, f);
        }

        let words_fst = fst::Set::from_iter(word_docids.keys().map(String::as_str))
            .unwrap()
            .map_data(Cow::Owned)
            .unwrap();
        let docid_word_pair_proximity_fst =
            fst::Map::from_iter(docid_word_pair_proximity.into_iter()).unwrap();

        let word_docids = word_docids.into_iter().collect::<HashMap<_, _>>();

        debug!("built index in {:.02?}", before.elapsed());
        dbg!(docid_word_pair_proximity_fst.len());

        Index {
            word_docids,
            docid_words,
            docid_word_pair_proximity: docid_word_pair_proximity_fst,
            documents,
            words_fst,
        }
    }
}
