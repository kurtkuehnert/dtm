pub fn compress(input: &[u8]) -> Vec<u8> {
    let compressed = lz4_flex::compress_prepend_size(input);

    compressed
}

pub fn decompress(input: &[u8]) -> Vec<u8> {
    let decompressed = lz4_flex::decompress_size_prepended(input).unwrap();

    decompressed
}
