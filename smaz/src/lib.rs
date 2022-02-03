#![doc = include_str!("../README.md")]
#![warn(
    unreachable_pub,
    missing_debug_implementations,
    missing_docs,
    clippy::pedantic
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
    &[101, 100],
    &[32, 109, 101],
    &[109, 101, 100],
    &[101, 100, 32],
    &[105, 115],
    &[104],
    &[116, 97],
    &[117],
    &[195, 182],
    &[182],
    &[101, 114],
    &[97, 116],
    &[102],
    &[98],
    &[115, 32],
    &[44],
    &[99, 104],
    &[44, 32],
    &[118],
    &[32, 111],
    &[111, 99],
    &[32, 115],
    &[116, 105],
    &[32, 112],
    &[165],
    &[104, 32],
    &[111, 99, 104],
    &[32, 111, 99],
    &[114, 195],
    &[111, 116],
    &[195, 164],
    &[164],
    &[116, 32],
    &[97, 116, 105],
    &[97, 115],
    &[121],
    &[116, 105, 115],
    &[116, 97, 116],
    &[111, 116, 97],
    &[116, 97, 116, 105],
    &[114, 97],
    &[105, 110],
    &[97, 32],
    &[32, 107],
    &[112, 111],
    &[103, 114],
    &[115, 116],
    &[112, 111, 116],
    &[112, 111, 116, 97, 116, 105],
    &[195, 165, 115],
    &[165, 115],
    &[110, 115],
    &[114, 195, 182],
    &[115, 195],
    &[32, 112, 111],
    &[110, 103],
    &[115, 195, 165],
    &[32, 112, 111, 116],
    &[115, 107],
    &[97, 114],
    &[112, 97],
    &[108, 105],
    &[107, 111],
    &[107, 116],
    &[114, 118],
    &[111, 114],
    &[115, 101],
    &[118, 101],
    &[108, 97],
    &[108, 108],
    &[75],
    &[97, 108],
    &[107, 97],
    &[114, 105],
    &[115, 97],
    &[115, 32, 109],
    &[111, 115],
    &[101, 114, 97],
    &[97, 107],
    &[32, 98],
    &[101, 116],
    &[107, 116, 32],
    &[32, 114],
    &[105, 110, 103],
    &[116, 116],
    &[32, 115, 101],
    &[115, 101, 114],
    &[114, 111],
    &[97, 100],
    &[118, 101, 114],
    &[116, 115],
    &[32, 103],
    &[114, 32],
    &[101, 114, 118],
    &[114, 118, 101],
    &[118, 101, 114, 97],
    &[114, 97, 115],
    &[97, 115, 32],
    &[115, 101, 114, 118],
    &[32, 115, 101, 114],
    &[105, 115, 107],
    &[106],
    &[101, 114, 97, 115],
    &[103, 114, 195],
    &[114, 97, 115, 32],
    &[115, 101, 114, 118, 101, 114, 97, 115],
    &[97, 115, 32, 109],
    &[101, 114, 97, 115, 32, 109],
    &[114, 97, 115, 32, 109, 101, 100],
    &[114, 97, 115, 32, 109, 101],
    &[115, 111],
    &[97, 110],
    &[115, 101, 114, 118, 101, 114, 97, 115, 32],
    &[115, 101, 114, 118, 101, 114, 97, 115, 32, 109],
    &[195, 182, 110],
    &[182, 110],
    &[114, 105, 115],
    &[108, 105, 110],
    &[97, 32, 109],
    &[32, 103, 114],
    &[115, 32, 111],
    &[115, 115],
    &[115, 44],
    &[115, 44, 32],
    &[111, 107],
    &[105, 108],
    &[112, 112],
    &[97, 32, 109, 101],
    &[195, 164, 114],
    &[164, 114],
    &[107, 32],
    &[107, 101],
    &[97, 32, 109, 101, 100],
    &[32, 103, 114, 195],
    &[116, 101],
    &[117, 108],
    &[109, 97],
    &[111, 110],
    &[195, 165, 115, 32],
    &[165, 115, 32],
    &[114, 121],
    &[32, 107, 111],
    &[108, 105, 110, 103],
    &[117, 114],
    &[114, 195, 182, 110],
    &[32, 102],
    &[83],
    &[115, 195, 165, 115, 32],
    &[115, 116, 97],
    &[98, 117],
    &[99, 107],
    &[195, 182, 100],
    &[182, 100],
    &[108, 195],
    &[98, 114],
    &[103, 32],
    &[107, 111, 107],
    &[101, 32],
    &[111, 112],
    &[114, 195, 182, 100],
    &[111, 107, 116],
    &[110, 101],
    &[102, 195],
    &[116, 32, 112],
    &[107, 108],
    &[115, 109],
    &[111, 107, 116, 32],
    &[111, 112, 112],
    &[112, 112, 97],
    &[107, 111, 107, 116],
    &[102, 102],
    &[109, 111],
    &[103, 114, 195, 182],
    &[32, 107, 111, 107],
    &[195, 182, 110, 115],
    &[182, 110, 115],
    &[107, 111, 107, 116, 32],
    &[101, 110],
    &[80],
    &[115, 97, 107],
    &[116, 32, 112, 111],
    &[114, 115],
    &[111, 112, 112, 97],
    &[32, 98, 114],
    &[107, 116, 32, 112],
    &[115, 111, 112],
    &[115, 107, 32],
    &[100, 32, 112],
    &[110, 115, 97],
    &[32, 107, 111, 107, 116],
    &[32, 118],
    &[107, 116, 32, 112, 111, 116, 97, 116],
    &[107, 116, 32, 112, 111, 116, 97],
    &[107, 116, 32, 112, 111, 116],
    &[107, 116, 32, 112, 111],
    &[32, 108],
    &[114, 195, 182, 110, 115],
    &[110, 115, 97, 107],
    &[100, 32, 107],
    &[32, 107, 111, 107, 116, 32],
    &[111, 115, 116],
    &[114, 101],
    &[195, 182, 116],
    &[182, 116],
    &[111, 107, 116, 32, 112],
    &[114, 116],
    &[111, 107, 116, 32, 112, 111, 116, 97, 116],
    &[111, 107, 116, 32, 112, 111, 116, 97],
    &[111, 107, 116, 32, 112, 111, 116],
    &[111, 107, 116, 32, 112, 111],
    &[116, 97, 32],
    &[107, 115],
    &[103, 114, 195, 182, 110],
    &[108, 32],
    &[97, 100, 32],
    &[108, 108, 97],
    &[111, 108],
    &[32, 107, 111, 107, 116, 32, 112],
    &[32, 107, 111, 107, 116, 32, 112, 111, 116, 97, 116],
    &[32, 107, 111, 107, 116, 32, 112, 111, 116, 97],
    &[32, 107, 111, 107, 116, 32, 112, 111, 116],
    &[32, 107, 111, 107, 116, 32, 112, 111],
    &[32, 103, 114, 195, 182],
    &[70],
    &[32, 99],
    &[100, 32, 115],
    &[97, 103],
    &[195, 182, 114],
    &[182, 114],
    &[97, 115, 116],
    &[98, 114, 195],
    &[32, 116],
    &[101, 100, 32, 112],
    &[101, 100, 32, 107],
];

lazy_static! {
    static ref LONGEST_CODE: usize = CODEBOOK.iter().map(|c| c.len()).max().unwrap();
    static ref CODEBOOK_MAP: HashMap<Vec<u8>, u8> = {
        let mut map: HashMap<Vec<u8>, u8> = HashMap::new();
        #[allow(clippy::cast_possible_truncation)]
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

/// `verbatim` must not be longer than 256.
fn flush_verbatim(verbatim: &[u8]) -> Vec<u8> {
    let mut chunk: Vec<u8> = Vec::new();
    if verbatim.len() > 1 {
        chunk.push(255);
        #[allow(clippy::cast_possible_truncation)]
        chunk.push((verbatim.len() - 1) as u8);
    } else {
        chunk.push(254);
    }
    for c in verbatim {
        chunk.push(*c);
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
/// assert_eq!(vec![243, 120, 0, 254, 66, 121, 247, 151, 33, 55], compressed);
/// ```
#[must_use]
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
/// let v = vec![243, 120, 0, 254, 66, 121, 247, 151, 33, 55];
/// let decompressed = decompress(&v).unwrap();
/// let origin = str::from_utf8(&decompressed).unwrap();
/// assert_eq!("Fisk Björkeby", origin);
/// ```
pub fn decompress(input: &[u8]) -> Result<Vec<u8>, DecompressError> {
    let mut out: Vec<u8> = Vec::with_capacity(input.len() * 3);
    let mut i: usize = 0;

    while i < input.len() {
        if input[i] == 254 {
            out.push(*input.get(i + 1).ok_or(DecompressError)?);
            i += 2;
        } else if input[i] == 255 {
            if i + *input.get(i + 1).ok_or(DecompressError)? as usize + 2 >= input.len() {
                return Err(DecompressError);
            }
            for j in 0..=input[i + 1] {
                out.push(input[i + 2 + j as usize]);
            }
            i += 3 + input[i + 1] as usize;
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
    use crate::{compress, decompress};

    #[test]
    fn unicode() {
        let plaintext = "Fisk Björkeby";
        assert!(compress(plaintext.as_bytes()).len() < plaintext.len());
    }

    #[test]
    fn old_fuzz_failures() {
        assert!(decompress(&[254]).is_err());
        assert!(decompress(&[255]).is_err());
    }
}
