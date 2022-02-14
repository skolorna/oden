use deunicode::deunicode_char;

use crate::tokenizer::{SeparatorKind, TokenKind};

/// Classify a separator.
///
/// ```
/// use euphemism::detection::classify_separator;
/// use euphemism::tokenizer::SeparatorKind;
///
/// assert_eq!(classify_separator(' '), Some(SeparatorKind::Soft));
/// assert_eq!(classify_separator('.'), Some(SeparatorKind::Hard));
/// assert_eq!(classify_separator('…'), Some(SeparatorKind::Hard));
/// assert_eq!(classify_separator('!'), Some(SeparatorKind::Hard));
/// assert_eq!(classify_separator('Å'), None);
/// ```
pub fn classify_separator(ch: char) -> Option<SeparatorKind> {
    match deunicode_char(ch)?.chars().next()? {
        '\u{00a0}' => None, // nbsp,
        ch if ch.is_whitespace() => Some(SeparatorKind::Soft),
        '-' | '_' | '\'' | ':' | '/' | '\\' | '@' | '"' | '+' | '~' | '=' | '^' | '*' | '#' => {
            Some(SeparatorKind::Soft)
        }
        '.' | ';' | ',' | '!' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' => {
            Some(SeparatorKind::Hard)
        }
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharCategory {
    Separator(SeparatorKind),
    Other,
}

impl From<CharCategory> for TokenKind {
    fn from(cat: CharCategory) -> Self {
        match cat {
            CharCategory::Separator(sep) => TokenKind::Separator(sep),
            CharCategory::Other => TokenKind::Word,
        }
    }
}

pub fn categorize_char(ch: char) -> CharCategory {
    if let Some(kind) = classify_separator(ch) {
        CharCategory::Separator(kind)
    } else {
        CharCategory::Other
    }
}
