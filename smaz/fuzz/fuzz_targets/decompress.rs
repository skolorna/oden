#![no_main]
use libfuzzer_sys::fuzz_target;
use smaz::decompress;

fuzz_target!(|data: &[u8]| {
    decompress(data).ok();
});
