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
