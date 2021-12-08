use tokenizer::tokenize;
use util::{bigrams, jaccard_index, BigramSet};

pub mod tokenizer;
pub mod util;

#[derive(Debug, PartialEq, Eq)]
pub struct Sample {
    label: String,
    tokenized: String,
    shingle: BigramSet,
}

impl Sample {
    pub fn new(val: &str) -> Self {
        let tokenized = tokenize(val);

        Self {
            label: val.to_owned(),
            shingle: bigrams(&tokenized),
            tokenized,
        }
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

#[derive(Debug)]
pub struct Cluster {
    pub samples: Vec<Sample>,
}

impl Cluster {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
        }
    }

    pub fn with_samples(samples: Vec<Sample>) -> Self {
        Self { samples }
    }

    pub fn shingles_iter(&self) -> impl Iterator<Item = &BigramSet> {
        self.samples.iter().map(|s| &s.shingle)
    }

    pub fn shingle(&self) -> Option<&BigramSet> {
        self.shingles_iter().next()
    }

    pub fn score(&self, sample: &Sample) -> f32 {
        self.shingle()
            .map(|s| jaccard_index(s, &sample.shingle))
            .unwrap_or(0.0)
    }

    pub fn label(&self) -> Option<&str> {
        Some(&self.samples.get(0)?.label)
    }
}

impl Default for Cluster {
    fn default() -> Self {
        Self { samples: vec![] }
    }
}
