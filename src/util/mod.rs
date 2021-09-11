use std::{collections::HashSet, hash::Hash};
use url::Url;

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

/// Check if a slice is sorted. **Deprecate when `slice::is_sorted` hits stable.**
/// ```
/// use menu_proxy::util::is_sorted;
///
/// assert!(is_sorted(&[] as &[i32]));
/// assert!(is_sorted(&[1, 2, 2, 4, 5]));
/// assert!(!is_sorted(&[1, 0, 1, 2, 5]));
/// ```
pub fn is_sorted<T: Ord>(data: &[T]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
}

pub fn last_path_segment(url: &Url) -> Option<&str> {
    url.path_segments()?
        .filter(|s| !s.is_empty()) // If the url contains a trailing slash, the last segment will be "".
        .last()
}

/// Extract digits from a character iterator.
/// ```
/// use menu_proxy::util::extract_digits;
///
/// assert_eq!(extract_digits("woah12there34".chars(), 10), 1234);
/// assert_eq!(extract_digits("abcdef".chars(), 16), 11259375);
/// ```
pub fn extract_digits<I>(chars: I, radix: u32) -> u32
where
    I: Iterator<Item = char>,
{
    let digits = chars.filter(|c| c.is_digit(radix)).collect::<String>();

    u32::from_str_radix(&digits, radix).unwrap()
}
