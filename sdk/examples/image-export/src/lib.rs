//! App-local, bounded 24-bit BMP export above the existing image decoder.
//! No file, network, surface or capability authority.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::vec::Vec;
use nieuw_os_image_codec::{Cancellation, Image, Limits};

pub const MAX_PIXELS: usize = 4 * 1024 * 1024;
pub const MAX_OUTPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_INPUT_BYTES: usize = 4 * 1024 * 1024;
const HEADER_BYTES: usize = 54;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidBackground,
    Dimensions,
    OutputLimit,
    OutOfMemory,
    Decode(nieuw_os_image_codec::Error),
}

/// This is only an admission envelope, not a hard decoder heap quota.
pub fn decode_limits() -> Limits {
    Limits {
        input_bytes: MAX_INPUT_BYTES,
        width: 16384,
        height: 16384,
        pixels: MAX_PIXELS,
        work_bytes: 64 * 1024 * 1024,
        metadata_bytes: 64 * 1024,
        png_chunks: 1024,
        jpeg_scans: 100,
    }
}

/// Convert supported PNG/JPEG bytes through the maintained decoder, then export.
/// The caller must explicitly choose a 24-bit RGB background for alpha flattening.
pub fn convert(input: &[u8], background: u32, output_limit: usize) -> Result<Vec<u8>, Error> {
    if background > 0xffffff {
        return Err(Error::InvalidBackground);
    }
    let image = nieuw_os_image_codec::decode(input, decode_limits(), &Cancellation::default())
        .map_err(Error::Decode)?;
    export(&image, background, output_limit)
}

/// Export display-oriented RGB24 BI_RGB, bottom-up, with zero row padding.
/// EXIF is baked into pixels; metadata and original bytes are not exported.
/// RGBA16 is rounded by the existing codec. ICC/gamma transforms remain unapplied.
pub fn export(image: &Image<'_>, background: u32, output_limit: usize) -> Result<Vec<u8>, Error> {
    if background > 0xffffff {
        return Err(Error::InvalidBackground);
    }
    let (width, height) = (image.display_width, image.display_height);
    if width == 0 || height == 0 || width > 16384 || height > 16384 {
        return Err(Error::Dimensions);
    }
    let pixels = width
        .checked_mul(height)
        .filter(|n| *n <= MAX_PIXELS)
        .ok_or(Error::Dimensions)?;
    let stride = width
        .checked_mul(3)
        .and_then(|n| n.checked_add(3))
        .map(|n| n & !3)
        .ok_or(Error::Dimensions)?;
    let data_bytes = stride.checked_mul(height).ok_or(Error::Dimensions)?;
    let total = HEADER_BYTES
        .checked_add(data_bytes)
        .ok_or(Error::Dimensions)?;
    if total > output_limit.min(MAX_OUTPUT_BYTES) {
        return Err(Error::OutputLimit);
    }
    let rgba_bytes = pixels.checked_mul(4).ok_or(Error::Dimensions)?;
    let mut rgba = Vec::new();
    rgba.try_reserve_exact(rgba_bytes)
        .map_err(|_| Error::OutOfMemory)?;
    rgba.resize(rgba_bytes, 0);
    image.rgba8_into(&mut rgba).map_err(Error::Decode)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(total)
        .map_err(|_| Error::OutOfMemory)?;
    output.resize(total, 0);
    output[..2].copy_from_slice(b"BM");
    output[2..6].copy_from_slice(&(total as u32).to_le_bytes());
    output[10..14].copy_from_slice(&(HEADER_BYTES as u32).to_le_bytes());
    output[14..18].copy_from_slice(&40u32.to_le_bytes());
    output[18..22].copy_from_slice(&(width as i32).to_le_bytes());
    output[22..26].copy_from_slice(&(height as i32).to_le_bytes());
    output[26..28].copy_from_slice(&1u16.to_le_bytes());
    output[28..30].copy_from_slice(&24u16.to_le_bytes());
    output[34..38].copy_from_slice(&(data_bytes as u32).to_le_bytes());
    for y in 0..height {
        for x in 0..width {
            let source = (y * width + x) * 4;
            let destination = HEADER_BYTES + (height - 1 - y) * stride + x * 3;
            let alpha = u32::from(rgba[source + 3]);
            for (offset, channel, shift) in [(0, 2, 0), (1, 1, 8), (2, 0, 16)] {
                let blended = (u32::from(rgba[source + channel]) * alpha
                    + ((background >> shift) & 255) * (255 - alpha)
                    + 127)
                    / 255;
                output[destination + offset] = blended as u8;
            }
        }
    }
    Ok(output)
}
