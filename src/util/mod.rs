use std::{collections::HashSet, hash::Hash};

/// Remove *all* duplicates from a vector, regardless of position.
/// ```
/// use menu_proxy::util::assert_unique;
///
/// let mut v = vec![1, 2, 3, 4, 5, 1];
/// assert_unique(&mut v);
/// assert_eq!(v, [1, 2, 3, 4, 5]);
/// ```
pub fn assert_unique<T: Eq + Hash + Clone>(v: &mut Vec<T>) {
    let mut seen = HashSet::new();
    v.retain(|e| seen.insert(e.clone()));
}
