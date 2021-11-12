#![no_main]
use libfuzzer_sys::fuzz_target;
use smaz::{compress, decompress};

fuzz_target!(|data: &[u8]| {
    let compressed = compress(data);
    assert_eq!(decompress(&compressed).unwrap(), data);
});
