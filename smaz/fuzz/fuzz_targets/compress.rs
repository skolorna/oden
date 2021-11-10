#![no_main]
use libfuzzer_sys::fuzz_target;
use smaz::compress;

fuzz_target!(|data: &[u8]| {
    compress(data);
});
