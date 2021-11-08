use libshoco_sys::size_t;

pub fn compress(data: &str) -> Vec<u8> {
    if data.is_empty() {
        return vec![];
    }

    unsafe {
        let mut buf: Vec<u8> = vec![0; data.len() * 4];
        let len = libshoco_sys::shoco_compress(
            data.as_ptr().cast(),
            data.len() as size_t,
            buf.as_mut_ptr().cast(),
            buf.len() as size_t,
        ) as usize;
        buf[..len].to_vec()
    }
}

pub fn decompress(bytes: &[u8]) -> Vec<u8> {
    unsafe {
        let mut buf: Vec<u8> = vec![0; bytes.len() * 2];
        let len = libshoco_sys::shoco_decompress(
            bytes.as_ptr().cast(),
            bytes.len() as size_t,
            buf.as_mut_ptr().cast(),
            buf.len() as size_t,
        ) as usize;
        buf[..len].to_vec()
    }
}

#[cfg(test)]
mod tests {
    use crate::{compress, decompress};

    #[test]
    fn it_works() {
        let plain = "Fisk Björkeby";
        let compressed = compress(plain);
        assert_eq!(
            compressed,
            [70, 206, 15, 66, 106, 164, 114, 107, 101, 98, 121]
        );
        let decompressed = decompress(&compressed);
        assert_eq!(decompressed, plain.as_bytes());
    }
}
