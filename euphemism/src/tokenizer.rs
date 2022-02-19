use std::borrow::Cow;

use cow_utils::CowUtils;

use rust_stemmers::Stemmer;
use unicode_normalization::UnicodeNormalization;

use crate::detection::{categorize_char, classify, CharCategory};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeparatorKind {
    Soft,
    Hard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Word,
    Separator(SeparatorKind),
    StopWord,
    Unknown,
}

impl Default for TokenKind {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Token<'a> {
    pub word: Cow<'a, str>,
    pub kind: TokenKind,
}

impl<'a> Token<'a> {
    pub fn text(&'a self) -> &str {
        self.word.as_ref()
    }

    pub fn is_stop_word(&self) -> bool {
        matches!(self.kind, TokenKind::StopWord)
    }

    pub fn is_word(&self) -> bool {
        matches!(self.kind, TokenKind::Word)
    }

    pub fn is_separator(&self) -> bool {
        matches!(self.kind, TokenKind::Separator(_))
    }
}

/// A latin-based tokenizer.
///
/// ```
/// use euphemism::tokenizer::LatinTokenizer;
///
/// let mut s: Vec<String> = LatinTokenizer::new("Fisk Björkeby med hemlagat potatismos").map(|t| t.text().to_string()).collect();
///
/// assert_eq!(s, ["Fisk"," ", "Björkeby", " ", "med", " ", "hemlagat", " ", "potatismos"])
/// ```
#[derive(Debug)]
pub struct LatinTokenizer<'a> {
    inner: &'a str,
    /// [`CharCategory`] of the next character.
    next_category: CharCategory,
}

impl<'a> LatinTokenizer<'a> {
    pub fn new(s: &'a str) -> Self {
        let next_char = s.chars().next().unwrap();

        Self {
            inner: s,
            next_category: categorize_char(next_char),
        }
    }
}

impl<'a> Iterator for LatinTokenizer<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let token_category = self.next_category;
        let mut chars = self.inner.chars();
        let mut len = chars.next()?.len_utf8();

        for ch in chars {
            self.next_category = categorize_char(ch);

            if token_category != self.next_category {
                break;
            }

            len += ch.len_utf8();
        }

        let mut token = Token {
            word: Cow::Borrowed(&self.inner[0..len]),
            kind: TokenKind::Unknown,
        };

        token.kind = classify(&token);

        self.inner = &self.inner[len..];

        Some(token)
    }
}

pub struct TokenStream<'a> {
    inner: Box<dyn Iterator<Item = Token<'a>> + 'a>,
}

impl<'a> Iterator for TokenStream<'a> {
    type Item = Token<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

pub fn analyze(text: &str) -> TokenStream<'_> {
    let iter = LatinTokenizer::new(text)
        .map(|token| LowercaseFilter::default().filter(token))
        .map(|token| UnicodeNormalizationFilter::default().filter(token))
        .map(|token| StemmingFilter::default().filter(token));

    TokenStream {
        inner: Box::new(iter),
    }
}

pub trait Filter: Sync + Send {
    fn filter<'a>(&self, token: Token<'a>) -> Token<'a>;
}

#[derive(Default)]
struct LowercaseFilter;

impl Filter for LowercaseFilter {
    fn filter<'a>(&self, mut token: Token<'a>) -> Token<'a> {
        if let Cow::Owned(s) = token.word.cow_to_lowercase() {
            token.word = Cow::Owned(s);
        }

        token
    }
}

#[derive(Default)]
struct UnicodeNormalizationFilter;

impl Filter for UnicodeNormalizationFilter {
    fn filter<'a>(&self, mut token: Token<'a>) -> Token<'a> {
        token.word = Cow::Owned(token.word.nfc().to_string());

        token
    }
}

struct StemmingFilter {
    stemmer: Stemmer,
}

impl Default for StemmingFilter {
    fn default() -> Self {
        Self {
            stemmer: Stemmer::create(rust_stemmers::Algorithm::Swedish),
        }
    }
}

impl Filter for StemmingFilter {
    fn filter<'a>(&self, mut token: Token<'a>) -> Token<'a> {
        match token.kind {
            TokenKind::Word | TokenKind::StopWord | TokenKind::Unknown => {
                let stem = self.stemmer.stem(&token.word);
                token.word = Cow::Owned(stem.to_string());
            }
            TokenKind::Separator(_) => {} // only letters are stemmable
        }

        token
    }
}
