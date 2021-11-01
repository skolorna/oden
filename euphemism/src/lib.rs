use tokenizer::tokenize;
use util::{bigrams, BigramSet};

use std::{collections::HashSet, fmt::Display, hash::Hash};

pub mod tokenizer;
pub mod util;

pub fn create_clusters(data: &[&str]) -> Vec<Cluster> {
    let mut clusters = Vec::<Cluster>::new();

    for label in data {
        let sample = Sample::new(label);

        let best = clusters
            .iter_mut()
            .max_by(|a, b| a.score(&sample).partial_cmp(&b.score(&sample)).unwrap());

        match best {
            Some(cluster) if cluster.score(&sample) > 0.6 => {
                cluster.samples.push(sample);
            }
            _ => clusters.push(Cluster::with_samples(vec![sample])),
        };
    }

    clusters
}

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
}

#[derive(Debug)]
pub struct Cluster {
    samples: Vec<Sample>,
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

impl Display for Cluster {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.label().unwrap_or("UNNAMED"))?;

        for sample in &self.samples {
            writeln!(f, "- {}", sample.label)?;
        }

        Ok(())
    }
}
/// ```
/// use std::collections::HashSet;
/// use euphemism::jaccard_index;
///
/// let mut a = HashSet::new();
///
/// a.insert(1);
/// a.insert(2);
/// a.insert(3);
///
/// let mut b = HashSet::new();
///
/// b.insert(2);
/// b.insert(3);
/// b.insert(4);
///
/// assert_eq!(jaccard_index(&a, &b), 0.5);
/// ```
pub fn jaccard_index<T: Eq + Hash>(a: &HashSet<T>, b: &HashSet<T>) -> f32 {
    a.intersection(b).count() as f32 / a.union(b).count() as f32
}

#[cfg(test)]
mod tests {
    use crate::create_clusters;

    #[test]
    fn it_works() {
        let lines = include_str!("../sample.txt").lines();
        let clusters = create_clusters(&lines.collect::<Vec<_>>());

        for cluster in clusters {
            eprintln!("{}", cluster)
        }
    }
}
