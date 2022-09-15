use crate::{
    CACHE, DEFAULT, DOUBLE_DIFF, DOUBLE_DIFF_RANGE, DTM, DTM_HEADER_SIZE, DTM_MAGIC, RUN_LENGTH,
    SINGLE_DIFF, SINGLE_DIFF_RANGE,
};
use std::{
    error::Error,
    fmt::{self, Display},
    fs,
    path::Path,
};

// static mut C_CACHE: i32 = 0;
// static mut C_SINGLE_DIFF: i32 = 0;
// static mut C_DOUBLE_DIFF: i32 = 0;
// static mut C_RUN_LENGTH: i32 = 0;
// static mut C_RUN_COUNT: i32 = 0;
// static mut C_DEFAULT: i32 = 0;

/// Errors that may occur during DTM image encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EncodeError {
    /// The input buffer does not contain enough pixel data.
    InsufficientInputData,
    /// The output buffer is too small to fit the encoded image.
    InsufficientOutputBuffer,
    /// An IO error occurred while saving the image.
    IoError,
}

impl Error for EncodeError {}

impl Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncodeError::InsufficientInputData => {
                f.write_str("The input buffer does not contain enough pixel data.")
            }
            EncodeError::InsufficientOutputBuffer => {
                f.write_str("Output buffer is too small to fit the encoded image")
            }
            EncodeError::IoError => {
                f.write_str("There occurred an io error while saving the image.")
            }
        }
    }
}

impl DTM {
    /// Encodes a DTM image from a pixel slice into file.
    #[inline]
    pub fn encode_file<P: AsRef<Path>>(&self, path: P, pixels: &[u16]) -> Result<(), EncodeError> {
        let encoded = self.encode_alloc(pixels)?;

        match fs::write(path, encoded) {
            Ok(_) => Ok(()),
            Err(_) => return Err(EncodeError::IoError),
        }
    }

    /// Encodes a DTM image from a pixel slice into a newly allocated `Vec`.
    #[inline]
    pub fn encode_alloc(&self, pixels: &[u16]) -> Result<Vec<u8>, EncodeError> {
        let mut pixels = match pixels.get(..self.image_pixel_count()) {
            Some(pixels) => Pixels::new(self.width, self.height, self.channel_count, pixels),
            None => return Err(EncodeError::InsufficientInputData),
        };

        let mut data = vec![0; DTM_HEADER_SIZE + 3 * self.image_pixel_count()];
        let mut bytes = Bytes::new(&mut data[DTM_HEADER_SIZE..]);

        let mut channel_sizes = [0; 4];
        let mut total_size = DTM_HEADER_SIZE;

        for channel_size in &mut channel_sizes[0..self.channel_count] {
            encode(&mut bytes, &mut pixels);

            if bytes.index >= self.channel_size() {
                bytes.index = 0;
                pixels
                    .data
                    .iter()
                    .skip(pixels.channel)
                    .step_by(pixels.channel_count)
                    .for_each(|&pixel| {
                        bytes.data[bytes.index] = pixel as u8;
                        bytes.data[bytes.index + 1] = (pixel >> 8) as u8;
                        bytes.index += 2;
                    });
            };

            *channel_size = bytes.index;
            total_size += *channel_size;

            bytes = Bytes::new(&mut bytes.data[*channel_size..]);
            pixels.next_channel();
        }

        data[0..3].copy_from_slice(DTM_MAGIC);
        data[3] = self.pixel_size as u8;
        data[4..8].copy_from_slice(&(self.width as u32).to_be_bytes());
        data[8..12].copy_from_slice(&(self.height as u32).to_be_bytes());

        for i in 0..4 {
            data[12 + i * 4..16 + i * 4].copy_from_slice(&(channel_sizes[i] as u32).to_be_bytes());
        }

        data.truncate(total_size);

        Ok(data)
    }
}

fn encode(bytes: &mut Bytes, pixels: &mut Pixels) {
    while !pixels.is_empty() {
        let previous_pixel = pixels.previous();
        let pixel = pixels.current();

        if pixel == previous_pixel {
            bytes.run_length += 1;

            if bytes.run_length == 63 {
                finish_run(bytes, pixels);
            }
        } else {
            if bytes.run_length > 0 {
                finish_run(bytes, pixels);
            }

            let diff = pixel as i32 - pixels.paeth() as i32;

            if (-DOUBLE_DIFF_RANGE..DOUBLE_DIFF_RANGE).contains(&diff) {
                if let Some(previous_diff) = bytes.outstanding_diff {
                    bytes.double_diff(previous_diff, diff);
                    bytes.outstanding_diff = None;
                } else {
                    bytes.outstanding_diff = Some(diff);
                }
            } else {
                if let Some(previous_diff) = bytes.outstanding_diff {
                    bytes.single_diff(previous_diff);
                    bytes.outstanding_diff = None;
                }

                if (-SINGLE_DIFF_RANGE..SINGLE_DIFF_RANGE).contains(&diff) {
                    bytes.single_diff(diff);
                } else if pixel == bytes.pixel_cache[(pixel % 64) as usize] {
                    bytes.cache(pixel);
                } else {
                    bytes.default(pixel);
                }
            }
        }

        bytes.pixel_cache[(pixel % 64) as usize] = pixel;
        pixels.index += 1;
    }

    if bytes.run_length > 0 {
        finish_run(bytes, pixels);
    }

    if let Some(previous_diff) = bytes.outstanding_diff {
        bytes.single_diff(previous_diff);
    }

    /*
    unsafe {
        let size = pixels.width * pixels.height;
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
        println!(
            "Default: {} ({}%)",
            C_DEFAULT,
            C_DEFAULT as f32 / size as f32 * 100.0
        );
        println!(
            "Total: {} ({}%)",
            bytes.index,
            bytes.index as f32 / (2 * size) as f32 * 100.0
        );
        println!();

        assert_eq!(
            C_CACHE + C_SINGLE_DIFF + C_DOUBLE_DIFF + C_RUN_LENGTH + C_DEFAULT,
            size as i32
        );

        C_CACHE = 0;
        C_SINGLE_DIFF = 0;
        C_DOUBLE_DIFF = 0;
        C_RUN_LENGTH = 0;
        C_RUN_COUNT = 0;
        C_DEFAULT = 0;
    }
     */
}

#[inline]
fn finish_run(bytes: &mut Bytes, pixels: &mut Pixels) {
    let mut run = true;

    if let Some(previous_diff) = bytes.outstanding_diff {
        if bytes.run_length == 1 {
            pixels.index -= 1;
            let diff = pixels.current() as i32 - pixels.paeth() as i32;
            pixels.index += 1;

            if (-DOUBLE_DIFF_RANGE..DOUBLE_DIFF_RANGE).contains(&previous_diff)
                && (-DOUBLE_DIFF_RANGE..DOUBLE_DIFF_RANGE).contains(&diff)
            {
                bytes.double_diff(previous_diff, diff);
                run = false;
            }
        }

        if run {
            bytes.single_diff(previous_diff);
        }

        bytes.outstanding_diff = None;
    }

    if run {
        bytes.run_length();
    }

    bytes.run_length = 0;
}

struct Bytes<'a> {
    data: &'a mut [u8],
    pixel_cache: [u16; 64],
    outstanding_diff: Option<i32>,
    run_length: u8,
    index: usize,
}

impl<'a> Bytes<'a> {
    #[inline]
    fn new(data: &'a mut [u8]) -> Self {
        Self {
            data,
            pixel_cache: [0; 64],
            outstanding_diff: None,
            run_length: 0,
            index: 0,
        }
    }

    #[inline]
    fn cache(&mut self, pixel: u16) {
        self.data[self.index] = CACHE | (pixel % 64) as u8;
        self.index += 1;
        // unsafe { C_CACHE += 1 };
    }

    #[inline]
    fn single_diff(&mut self, diff: i32) {
        self.data[self.index] = SINGLE_DIFF | (diff + SINGLE_DIFF_RANGE) as u8;
        self.index += 1;
        // unsafe { C_SINGLE_DIFF += 1 };
    }

    #[inline]
    fn double_diff(&mut self, previous_diff: i32, diff: i32) {
        self.data[self.index] = DOUBLE_DIFF
            | (((previous_diff + DOUBLE_DIFF_RANGE) as u8) << 3)
            | (diff + DOUBLE_DIFF_RANGE) as u8;
        self.index += 1;

        // unsafe { C_DOUBLE_DIFF += 2 };
    }

    #[inline]
    fn run_length(&mut self) {
        self.data[self.index] = RUN_LENGTH | self.run_length - 1;
        self.index += 1;
        // unsafe { C_RUN_LENGTH += self.run_length as i32 };
        // unsafe { C_RUN_COUNT += 1 };
    }

    #[inline]
    fn default(&mut self, pixel: u16) {
        self.data[self.index..self.index + 3].copy_from_slice(&[
            DEFAULT,
            pixel as u8,
            (pixel >> 8) as u8,
        ]);
        self.index += 3;
        // unsafe { C_DEFAULT += 1 };
    }
}

struct Pixels<'a> {
    width: usize,
    height: usize,
    channel_count: usize,
    data: &'a [u16],
    channel: usize,
    index: usize,
}

impl<'a> Pixels<'a> {
    #[inline]
    pub fn new(width: usize, height: usize, channel_count: usize, data: &'a [u16]) -> Self {
        Self {
            width,
            height,
            channel_count,
            data,
            channel: 0,
            index: 0,
        }
    }

    #[inline]
    fn get(&self, index: usize) -> u16 {
        self.data[index * self.channel_count + self.channel]
    }

    #[inline]
    fn current(&self) -> u16 {
        self.get(self.index)
    }

    #[inline]
    fn previous(&self) -> u16 {
        if self.index == 0 {
            0
        } else {
            self.get(self.index - 1)
        }
    }

    #[inline]
    fn paeth(&mut self) -> u16 {
        if self.index / self.width == 0 || self.index % self.width == 0 {
            self.previous()
        } else {
            let previous = self.get(self.index - 1);
            let above = self.get(self.index - self.width);
            let diagonal = self.get(self.index - self.width - 1);

            let p = previous.wrapping_add(above).wrapping_sub(diagonal);

            let diff_p_previous = p.abs_diff(previous);
            let diff_p_above = p.abs_diff(above);
            let diff_p_diagonal = p.abs_diff(diagonal);

            if diff_p_previous <= diff_p_above && diff_p_previous <= diff_p_diagonal {
                previous
            } else if diff_p_above <= diff_p_diagonal {
                above
            } else {
                diagonal
            }
        }
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.index == self.width * self.height
    }

    #[inline]
    fn next_channel(&mut self) {
        self.channel += 1;
        self.index = 0;
    }
}
