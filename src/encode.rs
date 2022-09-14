use crate::{
    paeth, CACHE, DEFAULT, DOUBLE_DIFF, DOUBLE_DIFF_RANGE, RUN_LENGTH, SINGLE_DIFF,
    SINGLE_DIFF_RANGE,
};

static mut C_CACHE: i32 = 0;
static mut C_SINGLE_DIFF: i32 = 0;
static mut C_DOUBLE_DIFF: i32 = 0;
static mut C_RUN_LENGTH: i32 = 0;
static mut C_RUN_COUNT: i32 = 0;
static mut C_DEFAULT: i32 = 0;

pub fn encode(width: u32, height: u32, pixels: &[u16]) -> Vec<u8> {
    let size = width as usize * height as usize;
    let mut output = Vec::with_capacity(size);
    let mut pixel_cache = [0; 64];
    let mut previous_pixel = 0;
    let mut run_length: u8 = 0;
    let mut outstanding_diff = None;

    for index in 0..pixels.len() {
        let pixel = pixels[index];

        encode_pixel(
            &mut output,
            pixels,
            &mut pixel_cache,
            &mut outstanding_diff,
            &mut run_length,
            index,
            previous_pixel,
            pixel,
            width as usize,
        );

        previous_pixel = pixel;
        pixel_cache[(pixel % 64) as usize] = pixel;
    }

    if run_length > 0 {
        finish_run(
            pixels,
            &mut output,
            &mut outstanding_diff,
            &mut run_length,
            pixels.len(),
            width as usize,
        );
    }

    if let Some(previous_diff) = outstanding_diff {
        single_diff(&mut output, previous_diff);
    }

    unsafe {
        println!(
            "Default: {} ({}%)",
            C_DEFAULT,
            (C_DEFAULT) as f32 / size as f32 * 100.0
        );
        println!(
            "Cache: {} ({}%)",
            C_CACHE,
            (C_CACHE) as f32 / size as f32 * 100.0
        );
        println!(
            "Single Diff: {} ({}%)",
            C_SINGLE_DIFF,
            C_SINGLE_DIFF as f32 / size as f32 * 100.0
        );
        println!(
            "Double Diff: {} ({}%)",
            C_DOUBLE_DIFF,
            C_DOUBLE_DIFF as f32 / size as f32 * 100.0
        );
        println!(
            "Run length: {} ({}%), with an average size of {}",
            C_RUN_LENGTH,
            C_RUN_LENGTH as f32 / size as f32 * 100.0,
            C_RUN_LENGTH as f32 / C_RUN_COUNT as f32
        );

        assert_eq!(
            C_CACHE + C_SINGLE_DIFF + C_DOUBLE_DIFF + C_RUN_LENGTH + C_DEFAULT,
            size as i32
        );

        C_DEFAULT = 0;
        C_CACHE = 0;
        C_SINGLE_DIFF = 0;
        C_DOUBLE_DIFF = 0;
        C_RUN_LENGTH = 0;
        C_RUN_COUNT = 0;
    }

    output
}

fn encode_pixel(
    mut output: &mut Vec<u8>,
    pixels: &[u16],
    pixel_cache: &mut [u16; 64],
    mut outstanding_diff: &mut Option<i32>,
    mut run_length: &mut u8,
    index: usize,
    previous_pixel: u16,
    pixel: u16,
    width: usize,
) {
    if pixel == previous_pixel {
        *run_length += 1;

        if *run_length == 63 {
            finish_run(pixels, output, outstanding_diff, run_length, index, width);
        }
    } else {
        if *run_length > 0 {
            finish_run(pixels, output, outstanding_diff, run_length, index, width);
        }

        let reference_pixel = paeth(pixels, index, width);
        let diff = pixel as i32 - reference_pixel as i32;

        if (-DOUBLE_DIFF_RANGE..DOUBLE_DIFF_RANGE).contains(&diff) {
            if let &mut Some(previous_diff) = outstanding_diff {
                double_diff(output, previous_diff, diff);
                *outstanding_diff = None;
            } else {
                *outstanding_diff = Some(diff);
            }
        } else {
            if let &mut Some(previous_diff) = outstanding_diff {
                single_diff(output, previous_diff);
                *outstanding_diff = None;
            }

            if (-SINGLE_DIFF_RANGE..SINGLE_DIFF_RANGE).contains(&diff) {
                single_diff(output, diff);
            } else if pixel == pixel_cache[(pixel % 64) as usize] {
                cache(output, pixel);
            } else {
                default(output, pixel);
            }
        }
    }
}

fn finish_run(
    pixels: &[u16],
    output: &mut Vec<u8>,
    outstanding_diff: &mut Option<i32>,
    length: &mut u8,
    index: usize,
    width: usize,
) {
    let mut run = true;

    if let &mut Some(previous_diff) = outstanding_diff {
        if *length == 1 {
            let pixel = pixels[index - 1];
            let reference_pixel = paeth(pixels, index - 1, width);
            let diff = pixel as i32 - reference_pixel as i32;

            if (-DOUBLE_DIFF_RANGE..DOUBLE_DIFF_RANGE).contains(&previous_diff)
                && (-DOUBLE_DIFF_RANGE..DOUBLE_DIFF_RANGE).contains(&diff)
            {
                double_diff(output, previous_diff, diff);
                run = false;
            }
        }

        if run {
            single_diff(output, previous_diff);
        }

        *outstanding_diff = None;
    }

    if run {
        run_length(output, *length);
    }

    *length = 0;
}

#[inline]
fn cache(output: &mut Vec<u8>, pixel: u16) {
    output.push(CACHE | (pixel % 64) as u8);
    unsafe { C_CACHE += 1 };
}

#[inline]
fn single_diff(output: &mut Vec<u8>, diff: i32) {
    output.push(SINGLE_DIFF | (diff + SINGLE_DIFF_RANGE) as u8);
    unsafe { C_SINGLE_DIFF += 1 };
}

#[inline]
fn double_diff(output: &mut Vec<u8>, previous_diff: i32, diff: i32) {
    output.push(
        DOUBLE_DIFF
            | ((((previous_diff + DOUBLE_DIFF_RANGE) as u8) << 3)
                | (diff + DOUBLE_DIFF_RANGE) as u8),
    );

    unsafe { C_DOUBLE_DIFF += 2 };
}

#[inline]
fn run_length(output: &mut Vec<u8>, length: u8) {
    output.push(RUN_LENGTH | length - 1);
    unsafe { C_RUN_LENGTH += length as i32 };
    unsafe { C_RUN_COUNT += 1 };
}

#[inline]
fn default(output: &mut Vec<u8>, pixel: u16) {
    output.extend_from_slice(&[DEFAULT, (pixel >> 8) as u8, pixel as u8]);
    unsafe { C_DEFAULT += 1 };
}
