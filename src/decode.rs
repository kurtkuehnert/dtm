use crate::{
    CACHE, CACHE_END, DEFAULT, DOUBLE_DIFF, DOUBLE_DIFF_END, DOUBLE_DIFF_RANGE, DTM,
    DTM_HEADER_SIZE, DTM_MAGIC, MASK_3BIT, MASK_6BIT, RUN_LENGTH, RUN_LENGTH_END, SINGLE_DIFF,
    SINGLE_DIFF_END, SINGLE_DIFF_RANGE,
};
use std::{
    error::Error,
    fmt::{self, Display},
    fs,
    path::Path,
};

struct Header {
    descriptor: DTM,
    channel_sizes: [usize; 4],
    total_size: usize,
}

/// Errors that may occur during DTM image decoding.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DecodeError {
    /// The input buffer does not contain enough encoded data.
    InsufficientInputData,
    /// The encoded header contains an invalid magic value.
    ///
    /// First four encoded must contain `b"dtm"`.
    /// This usually indicates that the buffer does not contain a DTM image.
    InvalidMagic,
    /// The encoded header contains an invalid channels value.
    ///
    /// DTM supports 1 to 4 channels.
    /// Any other value can not be produced by a valid encoder.
    InvalidChannels,
    /// An IO error occurred while loading the image.
    IoError,
}

impl Error for DecodeError {}

impl Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::InsufficientInputData => {
                f.write_str("The input buffer does not contain enough encoded data.")
            }
            DecodeError::InvalidMagic => {
                f.write_str("The encoded header contains an invalid magic value.")
            }
            DecodeError::InvalidChannels => {
                f.write_str("The encoded header contains an invalid channels value. DTM supports 1 to 4 channels")
            }
            DecodeError::IoError => { f.write_str("There occurred an io error while loading the image.")}
        }
    }
}

impl DTM {
    /// Reads header from encoded DTM image.
    /// The returned header can be analyzed before proceeding parsing with [`DTM::decode_skip_header`].
    fn decode_header(encoded: &[u8]) -> Result<Header, DecodeError> {
        let header = if let Some(header) = encoded.get(..DTM_HEADER_SIZE) {
            header
        } else {
            return Err(DecodeError::InsufficientInputData);
        };

        if &header[0..3] != DTM_MAGIC {
            return Err(DecodeError::InvalidMagic);
        }

        let pixel_size = header[3] as u32;
        let width = u32::from_be_bytes(header[4..8].try_into().unwrap());
        let height = u32::from_be_bytes(header[8..12].try_into().unwrap());

        let mut channel_count = 0;
        let mut channel_sizes = [0; 4];
        let mut total_size = DTM_HEADER_SIZE;

        for i in 0..4 {
            let channel_size =
                u32::from_be_bytes(header[12 + i * 4..16 + i * 4].try_into().unwrap()) as usize;

            if channel_size == 0 {
                break;
            }

            channel_sizes[i] = channel_size;
            total_size += channel_size;
            channel_count += 1;
        }

        Ok(Header {
            descriptor: DTM {
                pixel_size,
                channel_count,
                width,
                height,
            },
            channel_sizes,
            total_size,
        })
    }

    /// Decodes a DTM image from a file into a newly allocated `Vec`.
    #[inline]
    pub fn decode_file<P: AsRef<Path>>(path: P) -> Result<(Self, Vec<u8>), DecodeError> {
        let encoded = match fs::read(path) {
            Ok(encoded) => encoded,
            Err(_) => return Err(DecodeError::IoError),
        };

        DTM::decode_alloc(&encoded)
    }

    /// Decodes a DTM image from a byte slice into the `decoded` slice.
    #[inline]
    pub fn decode(encoded: &[u8], decoded: &mut [u8]) -> Result<Self, DecodeError> {
        let Header {
            descriptor,
            channel_sizes,
            total_size,
        } = Self::decode_header(encoded)?;

        let mut encoded = match encoded.get(DTM_HEADER_SIZE..total_size) {
            Some(encoded) => Encoded::new(encoded),
            None => return Err(DecodeError::InsufficientInputData),
        };

        let mut decoded = Decoded::new(
            descriptor.width as usize,
            descriptor.height as usize,
            descriptor.channel_count as usize,
            decoded,
        );

        for &channel_size in &channel_sizes[0..descriptor.channel_count as usize] {
            encoded.next_channel(channel_size);

            if channel_size < descriptor.channel_size() {
                decode(&mut encoded, &mut decoded)?;
            } else if channel_size == descriptor.channel_size() {
                encoded.data[..channel_size]
                    .chunks_exact(2)
                    .for_each(|encoded| {
                        decoded.set(encoded[0] as u16 + ((encoded[1] as u16) << 8))
                    });
            } else {
                return Err(DecodeError::InvalidChannels);
            }

            decoded.next_channel();
        }

        Ok(descriptor)
    }

    /// Decodes a DTM image from a byte slice into a newly allocated `Vec`.
    #[inline]
    pub fn decode_alloc(encoded: &[u8]) -> Result<(Self, Vec<u8>), DecodeError> {
        let header = Self::decode_header(encoded)?;
        let mut decoded = vec![0; header.descriptor.image_size()];
        let descriptor = Self::decode(encoded, &mut decoded)?;

        Ok((descriptor, decoded))
    }
}

fn decode(encoded: &mut Encoded, decoded: &mut Decoded) -> Result<(), DecodeError> {
    while !encoded.is_empty() {
        let byte = encoded.next();

        match byte {
            CACHE..=CACHE_END => {
                let index = MASK_6BIT & byte;
                decoded.set(decoded.cache[index as usize]);
            }
            SINGLE_DIFF..=SINGLE_DIFF_END => {
                let diff = (MASK_6BIT & byte) as i32 - SINGLE_DIFF_RANGE;
                let pixel = (decoded.paeth() as i32 + diff) as u16;
                decoded.set(pixel);
            }
            DOUBLE_DIFF..=DOUBLE_DIFF_END => {
                let diff = (MASK_3BIT & (byte >> 3)) as i32 - DOUBLE_DIFF_RANGE;
                let pixel = (decoded.paeth() as i32 + diff) as u16;
                decoded.set(pixel);

                let diff = (MASK_3BIT & byte) as i32 - DOUBLE_DIFF_RANGE;
                let pixel = (decoded.paeth() as i32 + diff) as u16;
                decoded.set(pixel);
            }
            RUN_LENGTH..=RUN_LENGTH_END => {
                let run_length = (MASK_6BIT & byte + 1) as usize;
                let pixel = decoded.previous();
                (0..run_length).for_each(|_| decoded.set(pixel));
            }
            DEFAULT => {
                decoded.set(encoded.next() as u16 + ((encoded.next() as u16) << 8));
            }
        }
    }

    if decoded.is_empty() {
        Ok(())
    } else {
        Err(DecodeError::InsufficientInputData)
    }
}

struct Encoded<'a> {
    data: &'a [u8],
    index: usize,
    channel_size: usize,
}

impl<'a> Encoded<'a> {
    #[inline]
    fn new(data: &'a [u8]) -> Self {
        Self {
            data,
            index: 0,
            channel_size: 0,
        }
    }

    #[inline]
    fn next(&mut self) -> u8 {
        let byte = self.data[self.index];
        self.index += 1;
        byte
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.index == self.channel_size
    }

    #[inline]
    fn next_channel(&mut self, channel_size: usize) {
        self.data = &self.data[self.channel_size..];
        self.channel_size = channel_size;
        self.index = 0;
    }
}

struct Decoded<'a> {
    width: usize,
    height: usize,
    channel_count: usize,
    data: &'a mut [u8],
    cache: [u16; 64],
    channel: usize,
    index: usize,
}

impl<'a> Decoded<'a> {
    #[inline]
    pub fn new(width: usize, height: usize, channel_count: usize, data: &'a mut [u8]) -> Self {
        Self {
            width,
            height,
            channel_count,
            data,
            cache: [0; 64],
            channel: 0,
            index: 0,
        }
    }

    #[inline]
    fn get(&self, index: usize) -> u16 {
        let index = (index * self.channel_count + self.channel) << 1;
        u16::from_le_bytes(self.data[index..index + 2].try_into().unwrap())
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
    fn set(&mut self, pixel: u16) {
        let index = (self.index * self.channel_count + self.channel) << 1;
        self.data[index..index + 2].copy_from_slice(&pixel.to_le_bytes());
        self.cache[pixel as usize % 64] = pixel;
        self.index += 1;
    }

    #[inline]
    fn is_empty(&self) -> bool {
        self.index == self.width * self.height
    }

    #[inline]
    fn next_channel(&mut self) {
        self.cache = [0; 64];
        self.channel += 1;
        self.index = 0;
    }
}
