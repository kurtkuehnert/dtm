use crate::{
    paeth, CACHE, CACHE_END, DEFAULT, DOUBLE_DIFF, DOUBLE_DIFF_END, DOUBLE_DIFF_RANGE, MASK_3BIT,
    MASK_6BIT, RUN_LENGTH, RUN_LENGTH_END, SINGLE_DIFF, SINGLE_DIFF_END, SINGLE_DIFF_RANGE,
};

pub fn decode(width: u32, height: u32, input: &[u8]) -> Vec<u16> {
    let mut pixels = Vec::with_capacity(width as usize * height as usize);
    let mut pixel_cache = [0; 64];
    let mut index = 0;

    while index != input.len() {
        index += decode_byte(input, &mut pixels, &mut pixel_cache, index, width as usize);
    }

    pixels
}

#[inline]
fn decode_byte(
    input: &[u8],
    mut pixels: &mut Vec<u16>,
    pixel_cache: &mut [u16; 64],
    index: usize,
    width: usize,
) -> usize {
    let byte = input[index];

    match byte {
        CACHE..=CACHE_END => {
            let pixel = pixel_cache[(MASK_6BIT & byte) as usize];
            pixels.push(pixel);

            1
        }
        SINGLE_DIFF..=SINGLE_DIFF_END => {
            let reference_pixel = paeth(&pixels, pixels.len(), width) as i32;
            let diff = (MASK_6BIT & byte) as i32 - SINGLE_DIFF_RANGE;
            let pixel = (reference_pixel + diff) as u16;
            pixel_cache[pixel as usize % 64] = pixel;
            pixels.push(pixel);

            1
        }
        DOUBLE_DIFF..=DOUBLE_DIFF_END => {
            let reference_pixel = paeth(&pixels, pixels.len(), width) as i32;
            let diff = (MASK_3BIT & (byte >> 3)) as i32 - DOUBLE_DIFF_RANGE;
            let pixel = (reference_pixel + diff) as u16;
            pixel_cache[pixel as usize % 64] = pixel;
            pixels.push(pixel);

            let reference_pixel = paeth(&pixels, pixels.len(), width) as i32;
            let diff = (MASK_3BIT & byte) as i32 - DOUBLE_DIFF_RANGE;
            let pixel = (reference_pixel + diff) as u16;
            pixel_cache[pixel as usize % 64] = pixel;
            pixels.push(pixel);

            1
        }
        RUN_LENGTH..=RUN_LENGTH_END => {
            let run_length = MASK_6BIT & byte + 1;
            let previous_pixel = *pixels.last().unwrap_or(&0);
            pixels.extend((0..run_length).map(|_| previous_pixel));

            1
        }
        DEFAULT => {
            let pixel = ((input[index + 1] as u16) << 8) + input[index + 2] as u16;
            pixel_cache[pixel as usize % 64] = pixel;
            pixels.push(pixel);

            3
        }
    }
}
