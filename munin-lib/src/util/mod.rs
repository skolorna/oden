use std::{collections::HashSet, hash::Hash};

use chrono::Weekday;

/// Remove *all* duplicates from a vector, regardless of position.
/// ```
/// use munin_lib::util::retain_unique;
///
/// let mut v = vec![1, 2, 3, 4, 5, 1];
/// retain_unique(&mut v);
/// assert_eq!(v, [1, 2, 3, 4, 5]);
/// ```
pub fn retain_unique<T: Eq + Hash + Clone>(v: &mut Vec<T>) {
    let mut seen = HashSet::new();
    v.retain(|e| seen.insert(e.clone()));
}

/// Check if a slice is sorted. **Deprecate when `slice::is_sorted` hits stable.**
/// ```
/// use munin_lib::util::is_sorted;
///
/// assert!(is_sorted(&[] as &[i32]));
/// assert!(is_sorted(&[1, 2, 2, 4, 5]));
/// assert!(!is_sorted(&[1, 0, 1, 2, 5]));
/// ```
pub fn is_sorted<T: Ord>(data: &[T]) -> bool {
    data.windows(2).all(|w| w[0] <= w[1])
}

pub fn last_path_segment(path: &str) -> Option<&str> {
    path.split('/')
        .filter(|s| !s.is_empty()) // If the url contains a trailing slash, the last segment will be "".
        .last()
}

/// Extract digits from a character iterator.
/// ```
/// use munin_lib::util::extract_digits;
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

/// Parse weekday (Swedish).
///
/// ```
/// use munin_lib::util::parse_weekday;
/// use chrono::Weekday;
///
/// assert_eq!(parse_weekday("Måndag"), Some(Weekday::Mon));
/// assert_eq!(parse_weekday("Lördag"), Some(Weekday::Sat));
/// assert_eq!(parse_weekday("Söndag"), Some(Weekday::Sun));
/// assert_eq!(parse_weekday("söndag"), None);
/// ```
pub fn parse_weekday(literal: &str) -> Option<Weekday> {
    match literal {
        "Måndag" => Some(Weekday::Mon),
        "Tisdag" => Some(Weekday::Tue),
        "Onsdag" => Some(Weekday::Wed),
        "Torsdag" => Some(Weekday::Thu),
        "Fredag" => Some(Weekday::Fri),
        "Lördag" => Some(Weekday::Sat),
        "Söndag" => Some(Weekday::Sun),
        _ => None,
    }
}
