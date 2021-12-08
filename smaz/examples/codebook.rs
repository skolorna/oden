use std::{cmp::Ordering, collections::HashMap};

trait ContainsSlice<T>: PartialEq<[T]> {
    fn contains_slice(self: &'_ Self, slice: &'_ [T]) -> bool;
}

impl<T, Item: PartialEq<T>> ContainsSlice<T> for [Item] {
    fn contains_slice(self: &'_ [Item], slice: &'_ [T]) -> bool {
        let len = slice.len();
        if len == 0 {
            return true;
        }
        self.windows(len).any(move |sub_slice| sub_slice == slice)
    }
}

fn main() {
    let dict = include_str!("../../meals.txt");

    let mut frequencies: HashMap<Vec<u8>, usize> = HashMap::new();

    for line in dict.lines() {
        let line = line.as_bytes();

        for window_size in 1..=line.len() {
            for window in line.windows(window_size) {
                *frequencies.entry(window.to_vec()).or_insert(0) += 1;
            }
        }
    }

    let mut frequencies = frequencies.into_iter().collect::<Vec<_>>();
    frequencies.sort_by(|(va, fa), (vb, fb)| {
        let f_ord = fb.cmp(fa);
        match f_ord {
            Ordering::Equal => vb.len().cmp(&va.len()),
            _ => f_ord,
        }
    }); // Sort from highest to lowest

    let mut out: Vec<(Vec<u8>, usize)> = Vec::with_capacity(254);
    let mut i = 0;

    while out.len() < 254 {
        let (val, freq) = &frequencies[i];

        if out
            .iter()
            .find(|(b, f)| val.contains_slice(b) && *freq as f32 / *f as f32 > 0.99)
            .is_none()
        {
            out.push((val.to_vec(), *freq))
        }

        i += 1;
    }

    println!("static CODEBOOK: [&[u8]; 254] = [");

    for (val, _) in out {
        println!("  &{:?},", val);
    }

    println!("];");
}
