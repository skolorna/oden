use std::collections::{btree_map::Entry, BTreeMap};

use crate::tokenizer::{SeparatorKind, Token, TokenKind};

/// Extract deduplicated word proximities, i.e. there is an entry
/// for `A-B` but not `B-A`.
pub fn extract_word_pair_proximities<'a>(
    tokens: impl Iterator<Item = &'a Token<'a>>,
    max: usize,
) -> BTreeMap<(String, String), usize> {
    let mut words: Vec<_> = relative_token_offsets(tokens)
        .map(|(o, t)| (o, t.text().to_string()))
        .collect();
    words.sort_unstable_by(|(_, a), (_, b)| a.cmp(b));
    let mut proximities = BTreeMap::new();

    for (i, (loffset, lword)) in words.iter().enumerate() {
        for (roffset, rword) in &words[i..] {
            if lword == rword {
                continue;
            }

            let proximity = if loffset > roffset {
                loffset - roffset
            } else {
                roffset - loffset
            };

            if proximity > max {
                continue;
            }

            match proximities.entry((lword.to_string(), rword.to_string())) {
                Entry::Vacant(e) => {
                    e.insert(proximity);
                }
                Entry::Occupied(mut o) => {
                    if *o.get() > proximity {
                        o.insert(proximity);
                    }
                }
            }
        }
    }

    proximities
}

/// Calculate relative token offsets from separators and whatnot.
///
/// ```
/// use euphemism::tokenizer::analyze;
/// use euphemism::position::relative_token_offsets;
///
/// let analyzed = analyze("Fisk Björkeby serveras med kokt potatis");
/// let tokens = relative_token_offsets(analyzed);
/// let mut words = tokens.map(|(o, t)| (o, t.text().to_string()));
///
/// assert_eq!(words.next(), Some((0, "fisk".to_string())));
/// assert_eq!(words.next(), Some((1, "björkeby".to_string())));
/// assert_eq!(words.next(), Some((2, "server".to_string())));
/// assert_eq!(words.next(), Some((4, "kokt".to_string())));
/// assert_eq!(words.next(), Some((5, "potatis".to_string())));
/// assert_eq!(words.next(), None);
/// ```
pub fn relative_token_offsets<'a>(
    token_stream: impl Iterator<Item = &'a Token<'a>>,
) -> impl Iterator<Item = (usize, &'a Token<'a>)> {
    use TokenKind::*;

    token_stream
        .skip_while(|t| t.is_separator())
        .scan((0, None), |(offset, prev_kind), t| {
            match t.kind {
                Word | StopWord | Unknown => {
                    *offset += match *prev_kind {
                        Some(Separator(SeparatorKind::Hard)) => 8,
                        Some(_) => 1,
                        None => 0,
                    };
                    *prev_kind = Some(t.kind)
                }
                Separator(_) => {
                    if !matches!(prev_kind, Some(Separator(SeparatorKind::Hard))) {
                        *prev_kind = Some(t.kind)
                    }
                }
            }

            Some((*offset, t))
        })
        .filter(|(_, t)| t.is_word())
}
