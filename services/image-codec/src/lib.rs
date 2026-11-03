//! Native image decode component, never a file/network/display authority.
//!
//! Limits reject known oversized work before pixel decode. The maintained
//! engines still contain infallible allocations: the caller MUST execute them
//! in a private, bounded arena/process. OOM there terminates that decoder; this
//! API does not promise recovery from an upstream allocation failure or panic.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::{boxed::Box, sync::Arc, vec::Vec};
use core::sync::atomic::{AtomicBool, Ordering};
use zune_core::{
    bit_depth::ByteEndian, bytestream::ZCursor, colorspace::ColorSpace, options::DecoderOptions,
};
use zune_inflate::{DeflateDecoder, DeflateOptions};
use zune_jpeg::JpegDecoder;
use zune_png::PngDecoder;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    InvalidLimits,
    InputLimit,
    Dimensions,
    WorkLimit,
    MetadataLimit,
    InvalidData,
    UnknownFormat,
    UnsupportedAnimation,
    UnsupportedColor,
    UnsupportedPrecision,
    Cancelled,
    OutOfMemory,
    OutputSize,
}

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub input_bytes: usize,
    pub width: usize,
    pub height: usize,
    pub pixels: usize,
    /// Conservative admission estimate, not a replacement for the arena cap.
    pub work_bytes: usize,
    pub metadata_bytes: usize,
    pub png_chunks: usize,
    pub jpeg_scans: usize,
}

impl Limits {
    fn validate(self) -> Result<Self, Error> {
        if self.input_bytes == 0
            || self.width == 0
            || self.height == 0
            || self.width > 16384
            || self.height > 16384
            || self.pixels == 0
            || self.work_bytes == 0
            || self.metadata_bytes == 0
            || self.png_chunks == 0
            || self.jpeg_scans == 0
            || self.jpeg_scans > 256
        {
            return Err(Error::InvalidLimits);
        }
        Ok(self)
    }

    fn admit(
        self,
        w: usize,
        h: usize,
        input: usize,
        bytes_per_pixel: usize,
    ) -> Result<usize, Error> {
        let pixels = w.checked_mul(h).ok_or(Error::Dimensions)?;
        if w == 0 || h == 0 || w > self.width || h > self.height || pixels > self.pixels {
            return Err(Error::Dimensions);
        }
        let work = pixels
            .checked_mul(bytes_per_pixel)
            .and_then(|x| input.checked_mul(4)?.checked_add(x))
            .and_then(|x| self.metadata_bytes.checked_mul(4)?.checked_add(x))
            .and_then(|x| x.checked_add(131072))
            .ok_or(Error::WorkLimit)?;
        if work > self.work_bytes {
            return Err(Error::WorkLimit);
        }
        Ok(pixels)
    }
}

#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    fn check(&self) -> Result<(), Error> {
        if self.is_cancelled() {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug)]
pub enum Pixels {
    Rgba8(Vec<u8>),
    /// Native integer samples: full PNG16 precision and unassociated alpha.
    Rgba16(Vec<u16>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Format {
    Png,
    Jpeg,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Orientation {
    Normal,
    MirrorHorizontal,
    Rotate180,
    MirrorVertical,
    Transpose,
    Rotate90,
    Transverse,
    Rotate270,
}

impl Orientation {
    fn from_exif(value: u16) -> Option<Self> {
        Some(match value {
            1 => Self::Normal,
            2 => Self::MirrorHorizontal,
            3 => Self::Rotate180,
            4 => Self::MirrorVertical,
            5 => Self::Transpose,
            6 => Self::Rotate90,
            7 => Self::Transverse,
            8 => Self::Rotate270,
            _ => return None,
        })
    }

    fn display_dimensions(self, width: usize, height: usize) -> (usize, usize) {
        match self {
            Self::Transpose | Self::Rotate90 | Self::Transverse | Self::Rotate270 => {
                (height, width)
            }
            Self::Normal | Self::MirrorHorizontal | Self::Rotate180 | Self::MirrorVertical => {
                (width, height)
            }
        }
    }

    fn display_position(self, x: usize, y: usize, width: usize, height: usize) -> (usize, usize) {
        match self {
            Self::Normal => (x, y),
            Self::MirrorHorizontal => (width - 1 - x, y),
            Self::Rotate180 => (width - 1 - x, height - 1 - y),
            Self::MirrorVertical => (x, height - 1 - y),
            Self::Transpose => (y, x),
            Self::Rotate90 => (height - 1 - y, x),
            Self::Transverse => (height - 1 - y, width - 1 - x),
            Self::Rotate270 => (y, width - 1 - x),
        }
    }
}

impl Format {
    pub const fn media_type(self) -> &'static str {
        match self {
            Self::Png => "image/png",
            Self::Jpeg => "image/jpeg",
        }
    }
}

/// Detects a supported image format from its signature only.
///
/// A match is a routing hint, not proof that the remaining bytes form a valid
/// image. `decode` still validates the complete input before returning pixels.
pub fn detect_format(input: &[u8]) -> Option<Format> {
    if input.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(Format::Png)
    } else if input.starts_with(b"\xff\xd8") {
        Some(Format::Jpeg)
    } else {
        None
    }
}

#[derive(Debug)]
pub struct Image<'a> {
    /// The original immutable object bytes, including all metadata. No re-encode.
    pub source: &'a [u8],
    pub format: Format,
    /// Dimensions in the decoded pixel order, before applying EXIF orientation.
    pub width: usize,
    pub height: usize,
    /// Dimensions returned by `rgba8_into` after applying a valid EXIF orientation.
    pub display_width: usize,
    pub display_height: usize,
    /// Valid JPEG EXIF orientation from IFD0; absent or malformed values are ignored.
    pub orientation: Option<Orientation>,
    /// Raw decoder samples remain in source order; use `rgba8_into` for display pixels.
    pub pixels: Pixels,
    pub has_exif: bool,
    pub has_icc: bool,
    /// Raw decoder samples have no metadata transforms; `rgba8_into` applies
    /// JPEG EXIF orientation only. ICC/gamma transforms remain unapplied.
    pub metadata_transforms_applied: bool,
    /// ICC color management remains unapplied.
    pub icc_color_transform_applied: bool,
}

impl Image<'_> {
    /// Produce display-oriented RGBA8 pixels; full-precision samples and source stay unchanged.
    pub fn rgba8_into(&self, out: &mut [u8]) -> Result<(), Error> {
        let orientation = self.orientation.unwrap_or(Orientation::Normal);
        let (expected_width, expected_height) =
            orientation.display_dimensions(self.width, self.height);
        if (self.display_width, self.display_height) != (expected_width, expected_height) {
            return Err(Error::OutputSize);
        }
        let source_pixels = self
            .width
            .checked_mul(self.height)
            .ok_or(Error::OutputSize)?;
        let required = self
            .display_width
            .checked_mul(self.display_height)
            .and_then(|x| x.checked_mul(4))
            .ok_or(Error::OutputSize)?;
        let source_required = source_pixels.checked_mul(4).ok_or(Error::OutputSize)?;
        if out.len() != required || required != source_required {
            return Err(Error::OutputSize);
        }
        match &self.pixels {
            Pixels::Rgba8(v) => {
                if v.len() != source_required {
                    return Err(Error::OutputSize);
                }
                for y in 0..self.height {
                    for x in 0..self.width {
                        let (display_x, display_y) =
                            orientation.display_position(x, y, self.width, self.height);
                        let source = (y * self.width + x) * 4;
                        let destination = (display_y * self.display_width + display_x) * 4;
                        out[destination..destination + 4].copy_from_slice(&v[source..source + 4]);
                    }
                }
            }
            Pixels::Rgba16(v) => {
                if v.len() != source_required {
                    return Err(Error::OutputSize);
                }
                for y in 0..self.height {
                    for x in 0..self.width {
                        let (display_x, display_y) =
                            orientation.display_position(x, y, self.width, self.height);
                        let source = (y * self.width + x) * 4;
                        let destination = (display_y * self.display_width + display_x) * 4;
                        for component in 0..4 {
                            out[destination + component] =
                                ((u32::from(v[source + component]) + 128) / 257) as u8;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

/// FNV-1a checksum of raw decoder-order samples, NOT a security/integrity primitive.
/// PNG16 uses each sample's big-endian bytes, retaining all sixteen bits.
pub fn pixel_hash(image: &Image<'_>) -> u64 {
    fn byte(hash: u64, value: u8) -> u64 {
        (hash ^ u64::from(value)).wrapping_mul(0x100000001b3)
    }
    let mut hash = 0xcbf29ce484222325;
    match &image.pixels {
        Pixels::Rgba8(values) => {
            for value in values {
                hash = byte(hash, *value);
            }
        }
        Pixels::Rgba16(values) => {
            for value in values {
                for value in value.to_be_bytes() {
                    hash = byte(hash, value);
                }
            }
        }
    }
    hash
}

fn zeros<T: Default + Clone>(length: usize) -> Result<Vec<T>, Error> {
    let mut result = Vec::new();
    result
        .try_reserve_exact(length)
        .map_err(|_| Error::OutOfMemory)?;
    result.resize(length, T::default());
    Ok(result)
}

fn options(limits: Limits) -> DecoderOptions {
    DecoderOptions::new_safe()
        .set_strict_mode(true)
        .set_use_unsafe(false)
        .set_max_width(limits.width)
        .set_max_height(limits.height)
        .set_byte_endian(ByteEndian::BE)
        .png_set_confirm_crc(true)
        .jpeg_set_max_scans(limits.jpeg_scans)
}

fn be32(bytes: &[u8]) -> usize {
    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
}

fn tiff_u16(bytes: &[u8], offset: usize, little_endian: bool) -> Option<u16> {
    let value = bytes.get(offset..offset.checked_add(2)?)?;
    let pair = [value[0], value[1]];
    Some(if little_endian {
        u16::from_le_bytes(pair)
    } else {
        u16::from_be_bytes(pair)
    })
}

fn tiff_u32(bytes: &[u8], offset: usize, little_endian: bool) -> Option<u32> {
    let value = bytes.get(offset..offset.checked_add(4)?)?;
    let word = [value[0], value[1], value[2], value[3]];
    Some(if little_endian {
        u32::from_le_bytes(word)
    } else {
        u32::from_be_bytes(word)
    })
}

fn exif_orientation(payload: &[u8]) -> Option<Orientation> {
    let tiff = payload.strip_prefix(b"Exif\0\0")?;
    let little_endian = match tiff.get(..2)? {
        b"II" => true,
        b"MM" => false,
        _ => return None,
    };
    if tiff_u16(tiff, 2, little_endian)? != 42 {
        return None;
    }
    let ifd_offset = usize::try_from(tiff_u32(tiff, 4, little_endian)?).ok()?;
    if ifd_offset < 8 {
        return None;
    }
    let entry_count = usize::from(tiff_u16(tiff, ifd_offset, little_endian)?);
    let entries_start = ifd_offset.checked_add(2)?;
    let entries_end = entries_start.checked_add(entry_count.checked_mul(12)?)?;
    let ifd_end = entries_end.checked_add(4)?;
    if ifd_end > tiff.len() {
        return None;
    }
    for index in 0..entry_count {
        let entry = entries_start.checked_add(index.checked_mul(12)?)?;
        if tiff_u16(tiff, entry, little_endian)? != 0x0112 {
            continue;
        }
        if tiff_u16(tiff, entry.checked_add(2)?, little_endian)? != 3
            || tiff_u32(tiff, entry.checked_add(4)?, little_endian)? != 1
        {
            return None;
        }
        return Orientation::from_exif(tiff_u16(tiff, entry.checked_add(8)?, little_endian)?);
    }
    None
}

fn jpeg_exif_orientation(
    input: &[u8],
    metadata_limit: usize,
    cancel: &Cancellation,
) -> Result<Option<Orientation>, Error> {
    if !input.starts_with(b"\xff\xd8") {
        return Ok(None);
    }
    let mut offset = 2usize;
    loop {
        cancel.check()?;
        if input.get(offset) != Some(&0xff) {
            return Ok(None);
        }
        while input.get(offset) == Some(&0xff) {
            let Some(next) = offset.checked_add(1) else {
                return Ok(None);
            };
            offset = next;
        }
        let Some(&marker) = input.get(offset) else {
            return Ok(None);
        };
        let Some(next) = offset.checked_add(1) else {
            return Ok(None);
        };
        offset = next;
        if marker == 0xda || marker == 0xd9 {
            return Ok(None);
        }
        if marker == 0x01 || (0xd0..=0xd7).contains(&marker) {
            continue;
        }
        let Some(length_end) = offset.checked_add(2) else {
            return Ok(None);
        };
        let Some(length_bytes) = input.get(offset..length_end) else {
            return Ok(None);
        };
        let length = usize::from(u16::from_be_bytes([length_bytes[0], length_bytes[1]]));
        if length < 2 {
            return Ok(None);
        }
        let Some(segment_end) = offset.checked_add(length) else {
            return Ok(None);
        };
        let Some(payload) = input.get(length_end..segment_end) else {
            return Ok(None);
        };
        if marker == 0xe1 && payload.starts_with(b"Exif\0\0") {
            if payload.len() > metadata_limit {
                return Err(Error::MetadataLimit);
            }
            if let Some(orientation) = exif_orientation(payload) {
                return Ok(Some(orientation));
            }
        }
        offset = segment_end;
    }
}

/// Resource-envelope check only: PNG reconstruction, CRC, filters, Adam7 and
/// palette/alpha decoding remain exclusively in the maintained engine.
fn png_envelope(input: &[u8], limits: Limits, cancel: &Cancellation) -> Result<(), Error> {
    let mut offset = 8usize;
    let mut count = 0usize;
    let mut metadata = 0usize;
    let mut expanded = 0usize;
    let mut have_header = false;
    while offset < input.len() {
        cancel.check()?;
        count = count.checked_add(1).ok_or(Error::InputLimit)?;
        if count > limits.png_chunks {
            return Err(Error::InputLimit);
        }
        let header = input
            .get(offset..offset.checked_add(8).ok_or(Error::InvalidData)?)
            .ok_or(Error::InvalidData)?;
        let length = be32(header);
        let end = offset
            .checked_add(12)
            .and_then(|x| x.checked_add(length))
            .ok_or(Error::InvalidData)?;
        let data = input
            .get(offset + 8..end.checked_sub(4).ok_or(Error::InvalidData)?)
            .ok_or(Error::InvalidData)?;
        if end > input.len() {
            return Err(Error::InvalidData);
        }
        let kind = &header[4..8];
        if count == 1 {
            if kind != b"IHDR" || data.len() != 13 {
                return Err(Error::InvalidData);
            }
            limits.admit(be32(data), be32(&data[4..]), input.len(), 64)?;
            have_header = true;
        }
        if kind == b"acTL" || kind == b"fcTL" || kind == b"fdAT" {
            return Err(Error::UnsupportedAnimation); // Explicit open APNG profile.
        }
        if kind != b"IDAT" {
            metadata = metadata.checked_add(length).ok_or(Error::MetadataLimit)?;
            if metadata > limits.metadata_bytes {
                return Err(Error::MetadataLimit);
            }
        }
        if kind == b"iCCP" || kind == b"zTXt" {
            let separator = data
                .iter()
                .take(80)
                .position(|x| *x == 0)
                .filter(|x| *x > 0 && *x <= 79)
                .ok_or(Error::InvalidData)?;
            if data.get(separator + 1) != Some(&0) {
                return Err(Error::InvalidData);
            }
            let compressed = data.get(separator + 2..).ok_or(Error::InvalidData)?;
            let remaining = limits
                .metadata_bytes
                .checked_sub(expanded)
                .filter(|x| *x != 0)
                .ok_or(Error::MetadataLimit)?;
            let opts = DeflateOptions::default()
                .set_limit(remaining)
                .set_size_hint(remaining.min(4096))
                .set_confirm_checksum(true);
            let decoded = DeflateDecoder::new_with_options(compressed, opts)
                .decode_zlib()
                .map_err(|error| match error.error {
                    zune_inflate::errors::DecodeErrorStatus::OutputLimitExceeded(_, _) => {
                        Error::MetadataLimit
                    }
                    _ => Error::InvalidData,
                })?;
            expanded = expanded
                .checked_add(decoded.len())
                .ok_or(Error::MetadataLimit)?;
            if expanded > limits.metadata_bytes {
                return Err(Error::MetadataLimit);
            }
        }
        offset = end;
        if kind == b"IEND" {
            return if have_header && data.is_empty() && offset == input.len() {
                Ok(())
            } else {
                Err(Error::InvalidData)
            };
        }
    }
    Err(Error::InvalidData)
}

pub fn decode<'a>(
    input: &'a [u8],
    limits: Limits,
    cancel: &Cancellation,
) -> Result<Image<'a>, Error> {
    let limits = limits.validate()?;
    cancel.check()?;
    if input.len() > limits.input_bytes {
        return Err(Error::InputLimit);
    }
    match detect_format(input) {
        Some(Format::Png) => png(input, limits, cancel),
        Some(Format::Jpeg) => jpeg(input, limits, cancel),
        None => Err(Error::UnknownFormat),
    }
}

// Keep format-specific stack frames out of the dispatcher's JPEG call chain.
#[inline(never)]
fn png<'a>(input: &'a [u8], limits: Limits, cancel: &Cancellation) -> Result<Image<'a>, Error> {
    png_envelope(input, limits, cancel)?;
    let mut decoder = PngDecoder::new_with_options(ZCursor::new(input), options(limits));
    decoder.decode_headers().map_err(|_| Error::InvalidData)?;
    let (w, h) = decoder.dimensions().ok_or(Error::InvalidData)?;
    let pixels = limits.admit(w, h, input.len(), 64)?;
    let color = decoder.colorspace().ok_or(Error::InvalidData)?;
    let components = color.num_components();
    if !matches!(
        color,
        ColorSpace::RGB | ColorSpace::RGBA | ColorSpace::Luma | ColorSpace::LumaA
    ) {
        return Err(Error::UnsupportedColor);
    }
    let depth = decoder.depth().ok_or(Error::InvalidData)?;
    let bytes_per_sample = match depth {
        zune_core::bit_depth::BitDepth::Eight => 1,
        zune_core::bit_depth::BitDepth::Sixteen => 2,
        _ => return Err(Error::UnsupportedPrecision),
    };
    let expected = pixels
        .checked_mul(components)
        .and_then(|x| x.checked_mul(bytes_per_sample))
        .ok_or(Error::WorkLimit)?;
    if decoder.output_buffer_size() != Some(expected) {
        return Err(Error::InvalidData);
    }
    let info = decoder.info().ok_or(Error::InvalidData)?;
    let has_exif = info.exif.is_some();
    let has_icc = info.icc_profile.is_some();
    let mut raw = zeros::<u8>(expected)?;
    cancel.check()?;
    // decode_into byte output avoids upstream decode()'s mutable u16 alias cast.
    decoder
        .decode_into(&mut raw)
        .map_err(|_| Error::InvalidData)?;
    cancel.check()?;
    let output = if bytes_per_sample == 2 {
        let mut rgba = zeros::<u16>(pixels * 4)?;
        for (src, dst) in raw
            .chunks_exact(components * 2)
            .zip(rgba.chunks_exact_mut(4))
        {
            let sample = |n: usize| u16::from_be_bytes([src[n * 2], src[n * 2 + 1]]);
            expand16(
                color,
                sample(0),
                if components > 1 { sample(1) } else { 0 },
                if components > 2 { sample(2) } else { 0 },
                if components > 3 { sample(3) } else { 0 },
                dst,
            );
        }
        Pixels::Rgba16(rgba)
    } else {
        let mut rgba = zeros::<u8>(pixels * 4)?;
        for (src, dst) in raw.chunks_exact(components).zip(rgba.chunks_exact_mut(4)) {
            match color {
                ColorSpace::Luma => dst.copy_from_slice(&[src[0], src[0], src[0], 255]),
                ColorSpace::LumaA => dst.copy_from_slice(&[src[0], src[0], src[0], src[1]]),
                ColorSpace::RGB => dst.copy_from_slice(&[src[0], src[1], src[2], 255]),
                ColorSpace::RGBA => dst.copy_from_slice(src),
                _ => return Err(Error::UnsupportedColor),
            }
        }
        Pixels::Rgba8(rgba)
    };
    Ok(Image {
        source: input,
        format: Format::Png,
        width: w,
        height: h,
        display_width: w,
        display_height: h,
        orientation: None,
        pixels: output,
        has_exif,
        has_icc,
        metadata_transforms_applied: false,
        icc_color_transform_applied: false,
    })
}

fn expand16(color: ColorSpace, a: u16, b: u16, c: u16, d: u16, out: &mut [u16]) {
    match color {
        ColorSpace::Luma => out.copy_from_slice(&[a, a, a, 65535]),
        ColorSpace::LumaA => out.copy_from_slice(&[a, a, a, b]),
        ColorSpace::RGB => out.copy_from_slice(&[a, b, c, 65535]),
        ColorSpace::RGBA => out.copy_from_slice(&[a, b, c, d]),
        _ => unreachable!(), // Checked before invoking the maintained decoder.
    }
}

type Jpeg<'a> = JpegDecoder<ZCursor<&'a [u8]>>;

/// Construct the large upstream value on a separate stack frame, then move it
/// into the caller's private arena. That construction frame is gone before
/// Huffman/scan parsing starts. Vec's stable fallible allocation API reserves
/// exactly one element; the checked capacity means boxing cannot need shrink.
#[inline(never)]
fn owned_jpeg(input: &[u8], limits: Limits) -> Result<Box<[Jpeg<'_>]>, Error> {
    if core::mem::size_of::<Jpeg<'_>>() > limits.work_bytes {
        return Err(Error::WorkLimit);
    }
    let mut owned = Vec::new();
    owned.try_reserve_exact(1).map_err(|_| Error::OutOfMemory)?;
    if owned.capacity() != 1 {
        return Err(Error::WorkLimit);
    }
    owned.push(JpegDecoder::new_with_options(
        ZCursor::new(input),
        options(limits).jpeg_set_out_colorspace(ColorSpace::RGBA),
    ));
    Ok(owned.into_boxed_slice())
}

#[inline(never)]
fn jpeg<'a>(input: &'a [u8], limits: Limits, cancel: &Cancellation) -> Result<Image<'a>, Error> {
    let orientation = jpeg_exif_orientation(input, limits.metadata_bytes, cancel)?;
    let mut owner = owned_jpeg(input, limits)?;
    let decoder = owner.first_mut().ok_or(Error::InvalidData)?;
    let callback = cancel.clone();
    decoder.set_cancel(move || callback.is_cancelled());
    decoder.set_cancel_interval(32);
    decoder.decode_headers().map_err(|error| {
        if cancel.is_cancelled() {
            Error::Cancelled
        } else if matches!(error, zune_jpeg::errors::DecodeErrors::AllocationFailed) {
            Error::OutOfMemory
        } else {
            Error::InvalidData
        }
    })?;
    let info = decoder.info().ok_or(Error::InvalidData)?;
    let (w, h) = (usize::from(info.width), usize::from(info.height));
    let (display_width, display_height) = orientation
        .unwrap_or(Orientation::Normal)
        .display_dimensions(w, h);
    let pixels = limits.admit(w, h, input.len(), 64)?;
    if info.pixel_density != 8 {
        return Err(Error::UnsupportedPrecision);
    }
    if !matches!(
        decoder.input_colorspace(),
        Some(ColorSpace::Luma | ColorSpace::RGB | ColorSpace::YCbCr)
    ) {
        return Err(Error::UnsupportedColor);
    }
    let has_exif = info.exif_data.is_some();
    // ICC extraction can allocate: bound its source chunks by bounded input.
    let has_icc = decoder.icc_profile().is_some();
    let expected = pixels.checked_mul(4).ok_or(Error::WorkLimit)?;
    if decoder.output_buffer_size() != Some(expected) {
        return Err(Error::InvalidData);
    }
    let mut rgba = zeros::<u8>(expected)?;
    decoder.decode_into(&mut rgba).map_err(|error| {
        if cancel.is_cancelled() {
            Error::Cancelled
        } else if matches!(error, zune_jpeg::errors::DecodeErrors::AllocationFailed) {
            Error::OutOfMemory
        } else {
            Error::InvalidData
        }
    })?;
    cancel.check()?;
    Ok(Image {
        source: input,
        format: Format::Jpeg,
        width: w,
        height: h,
        display_width,
        display_height,
        orientation,
        pixels: Pixels::Rgba8(rgba),
        has_exif,
        has_icc,
        metadata_transforms_applied: false,
        icc_color_transform_applied: false,
    })
}
