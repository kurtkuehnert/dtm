use crate::{
    CACHE, CACHE_END, DEFAULT, DOUBLE_DIFF, DOUBLE_DIFF_END, DOUBLE_DIFF_RANGE, MASK_3BIT,
    MASK_6BIT, RUN_LENGTH, RUN_LENGTH_END, SINGLE_DIFF, SINGLE_DIFF_END, SINGLE_DIFF_RANGE,
};

pub fn decode(width: u32, height: u32, input: &[u8]) -> Vec<u16> {
    let mut pixels = Vec::with_capacity(width as usize * height as usize);

    let mut pixel_cache = [0; 64];

    let mut index = 0;

    while index != input.len() {
        let byte = input[index];

        match byte {
            CACHE..=CACHE_END => {
                pixels.push(pixel_cache[(MASK_6BIT & byte) as usize]);
                index += 1;
            }
            SINGLE_DIFF..=SINGLE_DIFF_END => {
                let reference_pixel = *pixels.last().unwrap_or(&0) as i32;
                let diff = (MASK_6BIT & byte) as i32 - SINGLE_DIFF_RANGE;
                pixels.push((reference_pixel + diff) as u16);
                index += 1;
            }
            DOUBLE_DIFF..=DOUBLE_DIFF_END => {
                let reference_pixel = *pixels.last().unwrap_or(&0);
                let diff = (MASK_3BIT & (byte >> 3)) as i32 - DOUBLE_DIFF_RANGE;
                pixels.push((reference_pixel as i32 + diff) as u16);

                let previous_pixel = *pixels.last().unwrap_or(&0);
                pixel_cache[(previous_pixel % 64) as usize] = previous_pixel;

                let reference_pixel = previous_pixel;
                let diff = (MASK_3BIT & byte) as i32 - DOUBLE_DIFF_RANGE;
                pixels.push((reference_pixel as i32 + diff) as u16);
                index += 1;
            }
            RUN_LENGTH..=RUN_LENGTH_END => {
                let run_length = MASK_6BIT & byte + 1;
                let previous_pixel = *pixels.last().unwrap_or(&0);
                pixels.extend((0..run_length).map(|_| previous_pixel));
                index += 1;
            }
            DEFAULT => {
                pixels.push(((input[index + 1] as u16) << 8) + input[index + 2] as u16);
                index += 3;
            }
        };

        let previous_pixel = *pixels.last().unwrap_or(&0);
        pixel_cache[(previous_pixel % 64) as usize] = previous_pixel;
    }

    pixels
}
