# Smaz

This is a fork of Dmitriy Sokolov's [Smaz](https://crates.io/crates/smaz) library.

## Generating a codebook

```rs
// Example of a build script

use std::collections::HashMap;
use indoc::formatdoc;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=sample.txt");

    let samples = include_str!("./sample.txt");

    let mut frequency: HashMap<&[u8], usize> = HashMap::new();

    for sample in samples.lines() {
        let b = sample.as_bytes();

        for window_size in 1..=b.len() {
            for window in b.windows(window_size) {
                *frequency.entry(window).or_insert(0) += 1;
            }
        }
    }

    let mut freq_vec = frequency.into_iter().collect::<Vec<_>>();
    freq_vec.sort_by_key(|(_, freq)| *freq);

    let highest_freq: Vec<_> = freq_vec
        .into_iter()
        .rev()
        .map(|(bytes, _)| format!("&{:?}", bytes))
        .take(254)
        .collect();
    let slice = highest_freq.join(", ");

    let out_dir = std::env::var_os("OUT_DIR").unwrap();
    let path = std::path::Path::new(&out_dir).join("codebook.rs");

    std::fs::write(
        &path,
        formatdoc! {r#"
            /// Compression codebook, used for compression
            pub static CODEBOOK: [&[u8]; 254] = [{}];
        "#, slice},
    )
    .unwrap();
}
```
