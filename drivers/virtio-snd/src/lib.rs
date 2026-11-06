#![no_std]
#![forbid(unsafe_code)]

//! Bounded virtio-snd control codec (OASIS virtio 1.3, section 5.14).
//!
//! Pure byte encoding and validation only: no transport, queue, DMA, MMIO,
//! device grant, broker, app permission or audio playback. Capture is never
//! selected. Callers pass exactly the device-written used-length bytes.

pub mod pcm;

pub const VIRTIO_SND_DEVICE_ID: u16 = 25;
pub const VIRTIO_SND_PCI_DEVICE_ID: u16 = 0x1040 + VIRTIO_SND_DEVICE_ID;

const VERSION_1: u64 = 1 << 32;
const ACCESS_PLATFORM: u64 = 1 << 33;
const F_CTLS: u64 = 1 << 0;

use pcm::S_OK;

const R_PCM_INFO: u32 = 0x0100;
const HDR_BYTES: usize = 4;
const CONFIG_BYTES: usize = 12;
const QUERY_BYTES: usize = 16;
const INFO_BYTES: usize = 32;

/// Own resource bound per config field; larger devices fail closed.
pub const MAX_ITEMS: u32 = 32;
pub const MAX_RESPONSE_BYTES: usize = HDR_BYTES + MAX_ITEMS as usize * INFO_BYTES;

const PCM_FEATURES_MASK: u32 = (1 << 5) - 1;
const FORMATS_MASK: u64 = (1 << 25) - 1;
const RATES_MASK: u64 = (1 << 14) - 1;
const FMT_S16: u64 = 1 << 5;
const RATE_48000: u64 = 1 << 7;
const D_OUTPUT: u8 = 0;
const D_INPUT: u8 = 1;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Length,
    Limit,
    Range,
    Overflow,
    Features,
    Status(u32),
    InvalidDirection,
    InvalidChannels,
    Reserved,
    Unsupported,
}

/// Features this codec accepts: modern virtio only, never CTLS.
///
/// A device offering ACCESS_PLATFORM is refused, as in `virtio-core`: this
/// crate makes no IOMMU or DMA-isolation claim.
pub fn negotiate_features(offered: u64) -> Result<Negotiated, Error> {
    if offered & VERSION_1 == 0 || offered & ACCESS_PLATFORM != 0 {
        return Err(Error::Features);
    }
    Ok(Negotiated(VERSION_1))
}

/// Only constructible through [`negotiate_features`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Negotiated(u64);

impl Negotiated {
    pub const fn bits(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Config {
    jacks: u32,
    streams: u32,
    chmaps: u32,
}

impl Config {
    /// Parses the 12-byte config; `controls` is absent because CTLS is never
    /// negotiated. The caller performs the generation-stable read.
    pub fn parse(features: Negotiated, bytes: &[u8]) -> Result<Self, Error> {
        if features.0 & F_CTLS != 0 {
            return Err(Error::Features);
        }
        if bytes.len() != CONFIG_BYTES {
            return Err(Error::Length);
        }
        let jacks = u32_at(bytes, 0)?;
        let streams = u32_at(bytes, 4)?;
        let chmaps = u32_at(bytes, 8)?;
        if jacks > MAX_ITEMS || streams > MAX_ITEMS || chmaps > MAX_ITEMS {
            return Err(Error::Limit);
        }
        Ok(Self {
            jacks,
            streams,
            chmaps,
        })
    }

    pub const fn jacks(self) -> u32 {
        self.jacks
    }

    pub const fn streams(self) -> u32 {
        self.streams
    }

    pub const fn chmaps(self) -> u32 {
        self.chmaps
    }

    pub fn query_stream(self, id: u32) -> Result<Query, Error> {
        self.query_streams(id, 1)
    }

    /// Range `[start, start + count)` must lie within advertised streams.
    pub fn query_streams(self, start: u32, count: u32) -> Result<Query, Error> {
        if count == 0 || count > MAX_ITEMS {
            return Err(Error::Range);
        }
        let end = start.checked_add(count).ok_or(Error::Overflow)?;
        if end > self.streams {
            return Err(Error::Range);
        }
        Ok(Query { start, count })
    }
}

/// One outstanding PCM_INFO query. Correlation, not capability authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Query {
    start: u32,
    count: u32,
}

impl Query {
    pub fn encode(self) -> [u8; QUERY_BYTES] {
        let mut bytes = [0; QUERY_BYTES];
        bytes[..4].copy_from_slice(&R_PCM_INFO.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.start.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.count.to_le_bytes());
        bytes[12..].copy_from_slice(&(INFO_BYTES as u32).to_le_bytes());
        bytes
    }

    /// Exact device-writable buffer size the transport must post.
    pub fn response_len(self) -> Result<usize, Error> {
        (self.count as usize)
            .checked_mul(INFO_BYTES)
            .and_then(|items| items.checked_add(HDR_BYTES))
            .ok_or(Error::Overflow)
    }

    /// `bytes` must be exactly the validated used-length slice.
    pub fn parse_response(self, bytes: &[u8]) -> Result<StreamInfos, Error> {
        if bytes.len() < HDR_BYTES {
            return Err(Error::Length);
        }
        let status = u32_at(bytes, 0)?;
        if status != S_OK {
            return Err(Error::Status(status));
        }
        if bytes.len() != self.response_len()? {
            return Err(Error::Length);
        }
        let mut infos = StreamInfos {
            items: [StreamInfo::EMPTY; MAX_ITEMS as usize],
            len: 0,
        };
        for (index, record) in bytes[HDR_BYTES..].chunks_exact(INFO_BYTES).enumerate() {
            let id = self
                .start
                .checked_add(u32::try_from(index).map_err(|_| Error::Overflow)?)
                .ok_or(Error::Overflow)?;
            let slot = infos.items.get_mut(index).ok_or(Error::Limit)?;
            *slot = parse_info(id, record)?;
            infos.len += 1;
        }
        Ok(infos)
    }
}

fn parse_info(id: u32, item: &[u8]) -> Result<StreamInfo, Error> {
    if item.len() != INFO_BYTES {
        return Err(Error::Length);
    }
    let features = u32_at(item, 4)?;
    let formats = u64_at(item, 8)?;
    let rates = u64_at(item, 16)?;
    let direction = item[24];
    let min = item[25];
    let max = item[26];
    if direction != D_OUTPUT && direction != D_INPUT {
        return Err(Error::InvalidDirection);
    }
    if min == 0 || min > max {
        return Err(Error::InvalidChannels);
    }
    if item[27..].iter().any(|byte| *byte != 0) {
        return Err(Error::Reserved);
    }
    if features & !PCM_FEATURES_MASK != 0
        || formats & !FORMATS_MASK != 0
        || rates & !RATES_MASK != 0
    {
        return Err(Error::Reserved);
    }
    if formats == 0 || rates == 0 {
        return Err(Error::Unsupported);
    }
    Ok(StreamInfo {
        id,
        features,
        formats,
        rates,
        direction,
        channels_min: min,
        channels_max: max,
    })
}

/// Fixed-capacity result; no heap allocation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StreamInfos {
    items: [StreamInfo; MAX_ITEMS as usize],
    len: usize,
}

impl StreamInfos {
    pub fn as_slice(&self) -> &[StreamInfo] {
        &self.items[..self.len]
    }

    /// First advertised output stream that supports stereo S16 at 48 kHz.
    pub fn first_stereo_s16_48000(&self) -> Result<PlaybackSelection, Error> {
        self.as_slice()
            .iter()
            .find_map(|info| info.stereo_s16_48000().ok())
            .ok_or(Error::Unsupported)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StreamInfo {
    id: u32,
    features: u32,
    formats: u64,
    rates: u64,
    direction: u8,
    channels_min: u8,
    channels_max: u8,
}

impl StreamInfo {
    const EMPTY: Self = Self {
        id: 0,
        features: 0,
        formats: 0,
        rates: 0,
        direction: D_OUTPUT,
        channels_min: 0,
        channels_max: 0,
    };

    pub const fn id(self) -> u32 {
        self.id
    }

    pub const fn is_output(self) -> bool {
        self.direction == D_OUTPUT
    }

    /// Advertised `VIRTIO_SND_PCM_F_*` bits (validated, reserved bits clear).
    pub const fn features(self) -> u32 {
        self.features
    }

    /// Advertised `VIRTIO_SND_PCM_FMT_*` bit map.
    pub const fn formats(self) -> u64 {
        self.formats
    }

    /// Advertised `VIRTIO_SND_PCM_RATE_*` bit map.
    pub const fn rates(self) -> u64 {
        self.rates
    }

    pub const fn channels_min(self) -> u8 {
        self.channels_min
    }

    pub const fn channels_max(self) -> u8 {
        self.channels_max
    }

    /// Selects the initial output profile; not a permanent product limit.
    pub fn stereo_s16_48000(self) -> Result<PlaybackSelection, Error> {
        if self.direction != D_OUTPUT
            || self.formats & FMT_S16 == 0
            || self.rates & RATE_48000 == 0
            || self.channels_min > 2
            || self.channels_max < 2
        {
            return Err(Error::Unsupported);
        }
        Ok(PlaybackSelection { stream_id: self.id })
    }
}

/// Advertised format selection only; grants no app permission or DMA.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlaybackSelection {
    stream_id: u32,
}

impl PlaybackSelection {
    pub const fn stream_id(self) -> u32 {
        self.stream_id
    }
}

fn u32_at(bytes: &[u8], offset: usize) -> Result<u32, Error> {
    let end = offset.checked_add(4).ok_or(Error::Overflow)?;
    let raw = bytes.get(offset..end).ok_or(Error::Length)?;
    Ok(u32::from_le_bytes(
        raw.try_into().map_err(|_| Error::Length)?,
    ))
}

fn u64_at(bytes: &[u8], offset: usize) -> Result<u64, Error> {
    let end = offset.checked_add(8).ok_or(Error::Overflow)?;
    let raw = bytes.get(offset..end).ok_or(Error::Length)?;
    Ok(u64::from_le_bytes(
        raw.try_into().map_err(|_| Error::Length)?,
    ))
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod transport_model;
