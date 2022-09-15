// pub use encode;

pub use decode::DecodeError;

pub mod decode;
pub mod encode;

pub const DTM_HEADER_SIZE: usize = 28;
pub const DTM_MAGIC: &[u8] = "dtm".as_bytes();

/// The descriptor of a DTM image.
///
/// This value is parsed from the image header during decoding or is specified for encoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DTM {
    pub pixel_size: usize,
    pub channel_count: usize,
    pub width: usize,
    pub height: usize,
}

impl DTM {
    /// Returns the number of pixels of the decoded image.
    #[inline]
    pub fn image_pixel_count(&self) -> usize {
        self.channel_count * self.width * self.height
    }

    /// Returns the size of a channel of the decoded image.
    #[inline]
    pub fn channel_size(&self) -> usize {
        self.pixel_size * self.width * self.height
    }
}

pub(crate) const CACHE: u8 = 0b00000000;
pub(crate) const CACHE_END: u8 = 0b00111111;
pub(crate) const SINGLE_DIFF: u8 = 0b01000000;
pub(crate) const SINGLE_DIFF_END: u8 = 0b01111111;
pub(crate) const DOUBLE_DIFF: u8 = 0b10000000;
pub(crate) const DOUBLE_DIFF_END: u8 = 0b10111111;
pub(crate) const RUN_LENGTH: u8 = 0b11000000;
pub(crate) const RUN_LENGTH_END: u8 = 0b11111110;
pub(crate) const DEFAULT: u8 = 0b11111111;

pub(crate) const MASK_6BIT: u8 = 0b00111111;
pub(crate) const MASK_3BIT: u8 = 0b00000111;

pub(crate) const SINGLE_DIFF_RANGE: i32 = 32;
pub(crate) const DOUBLE_DIFF_RANGE: i32 = 4;
