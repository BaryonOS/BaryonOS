//! Crop coordinates refer to the displayed, EXIF-oriented image.
//! The optional host CLI uses an upstream PNG encoder; this crop core is no_std.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::vec::Vec;
use nieuw_os_image_codec::{Image, Pixels};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: usize,
    pub y: usize,
    pub width: usize,
    pub height: usize,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Crop {
    pub width: usize,
    pub height: usize,
    pub rgba: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InvalidRectangle,
    UnsupportedPrecision,
    WorkLimit,
    OutOfMemory,
    Codec(nieuw_os_image_codec::Error),
}

/// Preserve all RGBA8 sample bytes, including unassociated alpha.
/// Reject RGBA16 instead of silently quantizing. The additional budget covers
/// the full oriented temporary plus crop output, not the caller's decoded Image.
/// No source bytes, pixels or metadata are modified. No file is encoded here.
pub fn crop(image: &Image<'_>, rect: Rect, extra_bytes_budget: usize) -> Result<Crop, Error> {
    if rect.width == 0
        || rect.height == 0
        || rect
            .x
            .checked_add(rect.width)
            .is_none_or(|end| end > image.display_width)
        || rect
            .y
            .checked_add(rect.height)
            .is_none_or(|end| end > image.display_height)
    {
        return Err(Error::InvalidRectangle);
    }
    if !matches!(image.pixels, Pixels::Rgba8(_)) {
        return Err(Error::UnsupportedPrecision);
    }
    let display_len = image
        .display_width
        .checked_mul(image.display_height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(Error::WorkLimit)?;
    let output_len = rect
        .width
        .checked_mul(rect.height)
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(Error::WorkLimit)?;
    if display_len
        .checked_add(output_len)
        .is_none_or(|sum| sum > extra_bytes_budget)
    {
        return Err(Error::WorkLimit);
    }
    let mut display = Vec::new();
    display
        .try_reserve_exact(display_len)
        .map_err(|_| Error::OutOfMemory)?;
    display.resize(display_len, 0);
    image.rgba8_into(&mut display).map_err(Error::Codec)?;
    let mut output = Vec::new();
    output
        .try_reserve_exact(output_len)
        .map_err(|_| Error::OutOfMemory)?;
    let row_bytes = rect.width * 4;
    for row in rect.y..rect.y + rect.height {
        let start = (row * image.display_width + rect.x) * 4;
        output.extend_from_slice(&display[start..start + row_bytes]);
    }
    Ok(Crop {
        width: rect.width,
        height: rect.height,
        rgba: output,
    })
}
