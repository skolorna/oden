use std::{collections::HashSet, hash::Hash};

use time::Weekday;

/// Remove *all* duplicates from a vector, even if it's not sorted.
pub fn retain_unique<T: Eq + Hash + Clone>(v: &mut Vec<T>) {
    let mut seen = HashSet::new();
    v.retain(|e| seen.insert(e.clone()));
}

/// Check if a slice is sorted. **Deprecate when `slice::is_sorted` hits stable.**
pub fn is_sorted<T: Ord>(data: &[T]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
}

pub fn last_path_segment(path: &str) -> Option<&str> {
    path.split('/')
        .filter(|s| !s.is_empty()) // If the url contains a trailing slash, the last segment will be "".
        .last()
}

/// Parse weekday (Swedish).
pub fn parse_weekday(literal: &str) -> Option<Weekday> {
    match literal {
        "Måndag" => Some(Weekday::Monday),
        "Tisdag" => Some(Weekday::Tuesday),
        "Onsdag" => Some(Weekday::Wednesday),
        "Torsdag" => Some(Weekday::Thursday),
        "Fredag" => Some(Weekday::Friday),
        "Lördag" => Some(Weekday::Saturday),
        "Söndag" => Some(Weekday::Sunday),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn retain_unique() {
        let mut v = vec![1, 2, 3, 4, 5, 1];
        super::retain_unique(&mut v);
        assert_eq!(v, [1, 2, 3, 4, 5]);
    }
}
