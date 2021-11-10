//! This is a fork of Dmitriy Sokolov's [Smaz](https://crates.io/crates/smaz) library.

#![warn(
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs
)]

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::str;

/// Compression codebook, used for compression
pub static CODEBOOK: [&[u8]; 254] = [
    &[32],
    &[115],
    &[97],
    &[116],
    &[114],
    &[101],
    &[111],
    &[105],
    &[107],
    &[195],
    &[108],
    &[100],
    &[110],
    &[109],
    &[103],
    &[112],
    &[99],
    &[100, 32],
    &[32, 109],
    &[109, 101],
    &[105, 115],
    &[101, 100],
    &[104],
    &[109, 101, 100],
    &[32, 109, 101],
    &[116, 97],
    &[101, 100, 32],
    &[32, 109, 101, 100],
    &[117],
    &[109, 101, 100, 32],
    &[32, 109, 101, 100, 32],
    &[182],
    &[195, 182],
    &[101, 114],
    &[97, 116],
    &[102],
    &[44],
    &[44, 32],
    &[98],
    &[115, 32],
    &[118],
    &[99, 104],
    &[32, 115],
    &[165],
    &[195, 165],
    &[116, 105],
    &[32, 111],
    &[111, 99],
    &[32, 112],
    &[111, 116],
    &[114, 195],
    &[104, 32],
    &[99, 104, 32],
    &[111, 99, 104],
    &[111, 99, 104, 32],
    &[32, 111, 99],
    &[32, 111, 99, 104],
    &[32, 111, 99, 104, 32],
    &[164],
    &[195, 164],
    &[116, 32],
    &[97, 116, 105],
    &[97, 115],
    &[121],
    &[116, 97, 116],
    &[116, 105, 115],
    &[97, 116, 105, 115],
    &[111, 116, 97],
    &[111, 116, 97, 116],
    &[116, 97, 116, 105],
    &[116, 97, 116, 105, 115],
    &[111, 116, 97, 116, 105],
    &[111, 116, 97, 116, 105, 115],
    &[114, 97],
    &[105, 110],
    &[103, 114],
    &[165, 115],
    &[195, 165, 115],
    &[112, 111],
    &[32, 107],
    &[112, 111, 116],
    &[112, 111, 116, 97],
    &[112, 111, 116, 97, 116],
    &[112, 111, 116, 97, 116, 105],
    &[115, 116],
    &[112, 111, 116, 97, 116, 105, 115],
    &[97, 32],
    &[110, 115],
    &[114, 195, 182],
    &[115, 195],
    &[110, 103],
    &[115, 195, 165],
    &[115, 195, 165, 115],
    &[115, 107],
    &[32, 112, 111],
    &[97, 114],
    &[32, 112, 111, 116],
    &[32, 112, 111, 116, 97],
    &[32, 112, 111, 116, 97, 116],
    &[32, 112, 111, 116, 97, 116, 105],
    &[32, 112, 111, 116, 97, 116, 105, 115],
    &[108, 105],
    &[112, 97],
    &[107, 111],
    &[114, 118],
    &[107, 116],
    &[115, 101],
    &[118, 101],
    &[108, 108],
    &[108, 97],
    &[111, 114],
    &[75],
    &[114, 105],
    &[107, 97],
    &[115, 32, 109],
    &[115, 32, 109, 101],
    &[115, 32, 109, 101, 100],
    &[115, 32, 109, 101, 100, 32],
    &[101, 116],
    &[101, 114, 97],
    &[115, 97],
    &[97, 107],
    &[97, 108],
    &[111, 115],
    &[105, 110, 103],
    &[32, 98],
    &[32, 114],
    &[116, 116],
    &[115, 101, 114],
    &[32, 115, 101],
    &[107, 116, 32],
    &[118, 101, 114],
    &[114, 97, 115],
    &[101, 114, 118],
    &[118, 101, 114, 97],
    &[114, 118, 101, 114],
    &[114, 118, 101],
    &[101, 114, 118, 101],
    &[101, 114, 118, 101, 114],
    &[97, 115, 32],
    &[114, 118, 101, 114, 97],
    &[101, 114, 118, 101, 114, 97],
    &[115, 101, 114, 118],
    &[115, 101, 114, 118, 101, 114],
    &[115, 101, 114, 118, 101],
    &[115, 101, 114, 118, 101, 114, 97],
    &[32, 115, 101, 114],
    &[32, 115, 101, 114, 118],
    &[32, 115, 101, 114, 118, 101],
    &[32, 115, 101, 114, 118, 101, 114],
    &[32, 115, 101, 114, 118, 101, 114, 97],
    &[116, 115],
    &[101, 114, 97, 115],
    &[118, 101, 114, 97, 115],
    &[114, 118, 101, 114, 97, 115],
    &[101, 114, 118, 101, 114, 97, 115],
    &[115, 101, 114, 118, 101, 114, 97, 115],
    &[32, 115, 101, 114, 118, 101, 114, 97, 115],
    &[114, 97, 115, 32],
    &[106],
    &[32, 103],
    &[101, 114, 97, 115, 32],
    &[118, 101, 114, 97, 115, 32],
    &[114, 118, 101, 114, 97, 115, 32],
    &[101, 114, 118, 101, 114, 97, 115, 32],
    &[115, 101, 114, 118, 101, 114, 97, 115, 32],
    &[32, 115, 101, 114, 118, 101, 114, 97, 115, 32],
    &[114, 111],
    &[97, 115, 32, 109],
    &[97, 115, 32, 109, 101],
    &[97, 115, 32, 109, 101, 100],
    &[97, 115, 32, 109, 101, 100, 32],
    &[105, 115, 107],
    &[114, 97, 115, 32, 109],
    &[114, 97, 115, 32, 109, 101],
    &[114, 97, 115, 32, 109, 101, 100],
    &[101, 114, 97, 115, 32, 109],
    &[118, 101, 114, 97, 115, 32, 109],
    &[114, 118, 101, 114, 97, 115, 32, 109],
    &[101, 114, 118, 101, 114, 97, 115, 32, 109],
    &[114, 97, 115, 32, 109, 101, 100, 32],
    &[101, 114, 97, 115, 32, 109, 101],
    &[101, 114, 97, 115, 32, 109, 101, 100],
    &[118, 101, 114, 97, 115, 32, 109, 101, 100],
    &[118, 101, 114, 97, 115, 32, 109, 101],
    &[114, 118, 101, 114, 97, 115, 32, 109, 101, 100],
    &[114, 118, 101, 114, 97, 115, 32, 109, 101],
    &[101, 114, 118, 101, 114, 97, 115, 32, 109, 101, 100],
    &[101, 114, 118, 101, 114, 97, 115, 32, 109, 101],
    &[101, 114, 97, 115, 32, 109, 101, 100, 32],
    &[118, 101, 114, 97, 115, 32, 109, 101, 100, 32],
    &[114, 118, 101, 114, 97, 115, 32, 109, 101, 100, 32],
    &[101, 114, 118, 101, 114, 97, 115, 32, 109, 101, 100, 32],
    &[115, 101, 114, 118, 101, 114, 97, 115, 32, 109],
    &[32, 115, 101, 114, 118, 101, 114, 97, 115, 32, 109],
    &[115, 101, 114, 118, 101, 114, 97, 115, 32, 109, 101, 100],
    &[115, 101, 114, 118, 101, 114, 97, 115, 32, 109, 101],
    &[32, 115, 101, 114, 118, 101, 114, 97, 115, 32, 109, 101, 100],
    &[32, 115, 101, 114, 118, 101, 114, 97, 115, 32, 109, 101],
    &[97, 110],
    &[115, 101, 114, 118, 101, 114, 97, 115, 32, 109, 101, 100, 32],
    &[
        32, 115, 101, 114, 118, 101, 114, 97, 115, 32, 109, 101, 100, 32,
    ],
    &[114, 32],
    &[97, 100],
    &[103, 114, 195],
    &[114, 105, 115],
    &[115, 44],
    &[108, 105, 110],
    &[107, 32],
    &[115, 44, 32],
    &[115, 111],
    &[182, 110],
    &[195, 182, 110],
    &[32, 103, 114],
    &[105, 108],
    &[107, 101],
    &[115, 115],
    &[97, 32, 109],
    &[111, 110],
    &[117, 108],
    &[112, 112],
    &[109, 97],
    &[115, 32, 111],
    &[115, 32, 111, 99],
    &[115, 32, 111, 99, 104],
    &[115, 32, 111, 99, 104, 32],
    &[111, 107],
    &[32, 103, 114, 195],
    &[108, 105, 110, 103],
    &[114, 121],
    &[97, 32, 109, 101],
    &[117, 114],
    &[116, 101],
    &[97, 32, 109, 101, 100],
    &[103, 32],
    &[97, 32, 109, 101, 100, 32],
    &[164, 114],
    &[195, 164, 114],
    &[32, 102],
    &[99, 107],
    &[98, 117],
    &[114, 195, 182, 110],
    &[195, 182, 100],
    &[182, 100],
    &[165, 115, 32],
    &[195, 165, 115, 32],
    &[32, 107, 111],
    &[108, 195],
    &[114, 195, 182, 100],
    &[102, 195],
    &[98, 114],
    &[115, 109],
    &[101, 110],
    &[110, 101],
];

lazy_static! {
    static ref LONGEST_CODE: usize = CODEBOOK.iter().map(|c| c.len()).max().unwrap();
    static ref CODEBOOK_MAP: HashMap<Vec<u8>, u8> = {
        let mut map: HashMap<Vec<u8>, u8> = HashMap::new();
        for (i, code) in CODEBOOK.iter().enumerate() {
            map.insert(code.to_vec(), i as u8);
        }
        map
    };
}

/// The error type for decompress operation.
///
/// Often this error occurs due to invalid data.
#[derive(Debug, Clone, Copy)]
pub struct DecompressError;

impl fmt::Display for DecompressError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid compressed data")
    }
}

impl Error for DecompressError {
    fn description(&self) -> &str {
        "invalid compressed data"
    }
}

fn flush_verbatim(verbatim: &[u8]) -> Vec<u8> {
    let mut chunk: Vec<u8> = Vec::new();
    if verbatim.len() > 1 {
        chunk.push(255);
        chunk.push((verbatim.len() - 1) as u8);
    } else {
        chunk.push(254);
    }
    for c in verbatim {
        chunk.push(*c)
    }
    chunk
}

/// Returns compressed data as a vector of bytes.
///
/// # Examples
///
/// ```
/// use smaz::compress;
///
/// let s = "Fisk Björkeby";
/// let compressed = compress(&s.as_bytes());
/// assert_eq!(vec![254, 70, 227, 0, 254, 66, 204, 34, 3, 9, 5, 30, 67], compressed);
/// ```
pub fn compress(input: &[u8]) -> Vec<u8> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len() / 2);
    let mut verbatim: Vec<u8> = Vec::new();
    let mut input_index = 0;

    while input_index < input.len() {
        let mut encoded = false;
        let max_len = LONGEST_CODE.min(input.len() - input_index);

        for i in (0..=max_len).rev() {
            let code = CODEBOOK_MAP.get(&input[input_index..input_index + i]);
            if let Some(v) = code {
                if !verbatim.is_empty() {
                    out.append(&mut flush_verbatim(&verbatim));
                    verbatim.clear();
                }
                out.push(*v);
                input_index += i;
                encoded = true;
                break;
            }
        }

        if !encoded {
            verbatim.push(input[input_index]);
            input_index += 1;

            if verbatim.len() == 256 {
                out.append(&mut flush_verbatim(&verbatim));
                verbatim.clear();
            }
        }
    }

    if !verbatim.is_empty() {
        out.append(&mut flush_verbatim(&verbatim));
    }
    out
}

/// Returns decompressed data as a vector of bytes.
///
/// # Errors
///
/// If the compressed data is invalid or encoded incorrectly, then an error
/// is returned [`DecompressError`](struct.DecompressError.html).
///
/// # Examples
///
/// ```
/// use std::str;
/// use smaz::decompress;
///
/// let v = vec![254, 70, 227, 0, 254, 66, 204, 34, 3, 9, 5, 30, 67];
/// let decompressed = decompress(&v).unwrap();
/// let origin = str::from_utf8(&decompressed).unwrap();
/// assert_eq!("Fisk Björkeby", origin);
/// ```
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, DecompressError> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len() * 3);
    let mut i: usize = 0;

    while i < input.len() {
        if input[i] == 254 {
            if i + 1 > input.len() {
                return Err(DecompressError);
            }
            out.push(input[i + 1]);
            i += 2;
        } else if input[i] == 255 {
            if i + input[i + 1] as usize + 2 >= input.len() {
                return Err(DecompressError);
            }
            for j in 0..=input[i + 1] {
                out.push(input[i + 2 + j as usize])
            }
            i += 3 + input[i + 1] as usize
        } else {
            for c in CODEBOOK[input[i] as usize].iter() {
                out.push(*c);
            }

            i += 1;
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use crate::compress;

    #[test]
    pub fn unicode() {
        let plaintext = "Fisk Björkeby";
        assert!(compress(plaintext.as_bytes()).len() < plaintext.len());
    }

    #[test]
    pub fn long() {
        assert_eq!(compress(b" serveras med ").len(), 1);
    }
}
