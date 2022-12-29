use time::Weekday;

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
