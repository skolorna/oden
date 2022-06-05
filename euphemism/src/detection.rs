use deunicode::deunicode_char;

use crate::tokenizer::{SeparatorKind, Token, TokenKind};

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
#[must_use]
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

#[must_use]
pub fn categorize_char(ch: char) -> CharCategory {
    if let Some(kind) = classify_separator(ch) {
        CharCategory::Separator(kind)
    } else {
        CharCategory::Other
    }
}

const STOP_WORDS: &[&str] = &[
    "i", "på", "under", "över", "från", "ur", "bakom", "med", "bredvid", "vid", "till", "hos",
    "mellan", "framför", "ovanför",
];

#[must_use]
pub fn classify(token: &Token<'_>) -> TokenKind {
    if STOP_WORDS.contains(&token.text()) {
        return TokenKind::StopWord;
    }

    if let Some(kind) = token.text().chars().find_map(classify_separator) {
        return TokenKind::Separator(kind);
    }

    TokenKind::Word
}
