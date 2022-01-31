use core::hash::Hash;
use std::collections::HashSet;

use itertools::Itertools;

pub type BigramSet = HashSet<(char, char)>;

/// Create a [HashSet] containing all [bigrams](https://en.wikipedia.org/wiki/Bigram).
/// ```
/// use euphemism::util::bigrams;
///
/// let set = bigrams("Hello");
///
/// assert!(set.contains(&('H', 'e')));
/// assert!(set.contains(&('e', 'l')));
/// assert!(set.contains(&('l', 'l')));
/// assert!(set.contains(&('l', 'o')));
/// ```
pub fn bigrams(val: &str) -> BigramSet {
    val.chars().tuple_windows().collect()
}

/// Calculate `|a ∩ b| / |a ∪ b|` of sets *a* and *b*.
///
/// ```
/// use std::collections::HashSet;
/// use euphemism::util::jaccard_index;
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
