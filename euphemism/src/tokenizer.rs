const STOP_WORDS: &[&str] = &[
    "i", "på", "under", "över", "från", "ur", "bakom", "med", "bredvid", "vid", "till", "hos",
    "mellan", "framför", "ovanför",
];

/// Tokenize a value, preparing it for further processing.
///
/// ```
/// use euphemism::tokenizer::tokenize;
///
/// // These are real-world examples
/// assert_eq!(tokenize("Köttbullar med makaroner och ketchup"), "Köttbullar");
/// assert_eq!(tokenize("Köttbullar, stuvade makaroner och grönsaker"), "Köttbullar");
/// assert_eq!(tokenize("Taco´s nöt med tillbehör (skola)"), "Tacos nöt");
/// ```
pub fn tokenize(val: &str) -> String {
    let mut output = String::new();

    'words: for word in val.split_whitespace() {
        if STOP_WORDS.contains(&word) {
            break;
        }

        for c in word.chars() {
            if c == ',' {
                break 'words;
            }

            if c.is_alphabetic() {
                output.push(c)
            }
        }

        output.push(' ');
    }

    output.trim().to_owned()
}
