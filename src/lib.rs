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
    pub pixel_size: u32,
    pub channel_count: u32,
    pub width: u32,
    pub height: u32,
}

impl DTM {
    /// Returns the size of the decoded image in bytes .
    #[inline]
    pub fn image_size(&self) -> usize {
        (self.pixel_size * self.channel_count * self.width * self.height) as usize
    }

    /// Returns the size of a channel of the decoded image in bytes .
    #[inline]
    pub fn channel_size(&self) -> usize {
        (self.pixel_size * self.width * self.height) as usize
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
