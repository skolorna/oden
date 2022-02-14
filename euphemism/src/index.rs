use std::collections::{BTreeMap, HashMap};

use crate::tokenizer::analyze;

pub type DocId = u32;
pub type Word = String;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Suggestion {
    distance: usize,
    value: Word,
}

pub struct Index<T> {
    pub inverted_index: BTreeMap<Word, Vec<DocId>>,
    pub documents: HashMap<DocId, T>,
    pub next_id: DocId,
}

#[allow(clippy::new_without_default)]
impl<T> Index<T>
where
    T: ToString + std::fmt::Debug,
{
    pub fn new() -> Self {
        Self {
            inverted_index: BTreeMap::new(),
            documents: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn insert(&mut self, doc: T) {
        let doc_text = doc.to_string();
        let tokens = analyze(&doc_text);
        let id = self.next_id;

        for token in tokens {
            let word = token.word();

            let docs = self.inverted_index.entry(word.to_string()).or_default();

            docs.push(id);
        }

        self.documents.insert(id, doc);

        self.next_id += 1;
    }

    pub fn documents(&self) -> &HashMap<DocId, T> {
        &self.documents
    }

    pub fn get_doc(&self, id: &DocId) -> Option<&T> {
        self.documents.get(id)
    }

    pub fn words(&self) -> impl Iterator<Item = &'_ str> {
        self.inverted_index.keys().map(|k| k.as_str())
    }
}
