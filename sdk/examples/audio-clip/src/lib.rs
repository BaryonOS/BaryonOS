//! Sample-exact PCM16 WAVE clipping. No playback, device, file or network rights.
#![no_std]
#![forbid(unsafe_code)]
extern crate alloc;

use alloc::vec::Vec;
use core::ops::Range;

pub const MAX_INPUT_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_CHUNKS: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    InputLimit,
    InvalidRiff,
    Truncated,
    ChunkLimit,
    DuplicateAudioChunk,
    MissingAudioChunk,
    UnsupportedFormat,
    InconsistentFormat,
    IncompleteFrame,
    Range,
    MetadataChoiceRequired,
    OutputLimit,
    OutOfMemory,
}

#[derive(Debug)]
pub struct Wave<'a> {
    channels: u16,
    sample_rate: u32,
    samples: &'a [u8],
    metadata_chunks: usize,
}

impl<'a> Wave<'a> {
    /// Validate the complete bounded RIFF stream, not just its audio prefix.
    /// Accept PCM16 mono/stereo and exactly one fmt/data pair in either order.
    pub fn parse(input: &'a [u8]) -> Result<Self, Error> {
        if input.len() > MAX_INPUT_BYTES {
            return Err(Error::InputLimit);
        }
        if input.len() < 12 {
            return Err(Error::Truncated);
        }
        if &input[..4] != b"RIFF"
            || &input[8..12] != b"WAVE"
            || u32_at(input, 4) as u64 + 8 != input.len() as u64
        {
            return Err(Error::InvalidRiff);
        }
        let mut at = 12usize;
        let mut chunks = 0;
        let mut metadata_chunks = 0;
        let mut format = None;
        let mut samples = None;
        while at < input.len() {
            chunks += 1;
            if chunks > MAX_CHUNKS {
                return Err(Error::ChunkLimit);
            }
            let header_end = at
                .checked_add(8)
                .filter(|end| *end <= input.len())
                .ok_or(Error::Truncated)?;
            let length = usize::try_from(u32_at(input, at + 4)).map_err(|_| Error::Truncated)?;
            let end = header_end
                .checked_add(length)
                .filter(|end| *end <= input.len())
                .ok_or(Error::Truncated)?;
            let next = end
                .checked_add(length & 1)
                .filter(|end| *end <= input.len())
                .ok_or(Error::Truncated)?;
            let body = &input[header_end..end];
            match &input[at..at + 4] {
                b"fmt " => {
                    if format.is_some() {
                        return Err(Error::DuplicateAudioChunk);
                    }
                    format = Some(parse_format(body)?);
                }
                b"data" => {
                    if samples.is_some() {
                        return Err(Error::DuplicateAudioChunk);
                    }
                    samples = Some(body);
                }
                _ => metadata_chunks += 1,
            }
            at = next;
        }
        let (channels, sample_rate) = format.ok_or(Error::MissingAudioChunk)?;
        let samples = samples.ok_or(Error::MissingAudioChunk)?;
        if samples.len() % (usize::from(channels) * 2) != 0 {
            return Err(Error::IncompleteFrame);
        }
        Ok(Self {
            channels,
            sample_rate,
            samples,
            metadata_chunks,
        })
    }

    pub const fn channels(&self) -> u16 {
        self.channels
    }
    pub const fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub const fn metadata_chunks(&self) -> usize {
        self.metadata_chunks
    }
    pub fn frames(&self) -> usize {
        self.samples.len() / (usize::from(self.channels) * 2)
    }

    /// Frame range is half-open and must be nonempty. Every channel's exact
    /// signed PCM16 bytes survive; no resampling, mixing or gain conversion.
    /// Dropping metadata is explicit because cue/loop positions become stale.
    pub fn clip(
        &self,
        range: Range<usize>,
        drop_metadata: bool,
        output_limit: usize,
    ) -> Result<Vec<u8>, Error> {
        if range.start >= range.end || range.end > self.frames() {
            return Err(Error::Range);
        }
        if self.metadata_chunks != 0 && !drop_metadata {
            return Err(Error::MetadataChoiceRequired);
        }
        let block = usize::from(self.channels) * 2;
        let samples = &self.samples[range.start * block..range.end * block];
        let total = 44 + samples.len();
        if total > output_limit.min(MAX_INPUT_BYTES) {
            return Err(Error::OutputLimit);
        }
        let mut output = Vec::new();
        output
            .try_reserve_exact(total)
            .map_err(|_| Error::OutOfMemory)?;
        output.extend_from_slice(b"RIFF");
        output.extend_from_slice(&((total - 8) as u32).to_le_bytes());
        output.extend_from_slice(b"WAVEfmt ");
        output.extend_from_slice(&16u32.to_le_bytes());
        output.extend_from_slice(&1u16.to_le_bytes());
        output.extend_from_slice(&self.channels.to_le_bytes());
        output.extend_from_slice(&self.sample_rate.to_le_bytes());
        output.extend_from_slice(&(self.sample_rate * block as u32).to_le_bytes());
        output.extend_from_slice(&(block as u16).to_le_bytes());
        output.extend_from_slice(&16u16.to_le_bytes());
        output.extend_from_slice(b"data");
        output.extend_from_slice(&(samples.len() as u32).to_le_bytes());
        output.extend_from_slice(samples);
        Ok(output)
    }
}

fn parse_format(body: &[u8]) -> Result<(u16, u32), Error> {
    if !matches!(body.len(), 16 | 18) {
        return Err(Error::UnsupportedFormat);
    }
    if u16_at(body, 0) != 1
        || u16_at(body, 14) != 16
        || !matches!(u16_at(body, 2), 1 | 2)
        || (body.len() == 18 && u16_at(body, 16) != 0)
    {
        return Err(Error::UnsupportedFormat);
    }
    let channels = u16_at(body, 2);
    let rate = u32_at(body, 4);
    let block = channels * 2;
    if !(8000..=192000).contains(&rate) {
        return Err(Error::UnsupportedFormat);
    }
    if u16_at(body, 12) != block || u32_at(body, 8) != rate * u32::from(block) {
        return Err(Error::InconsistentFormat);
    }
    Ok((channels, rate))
}

fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([bytes[at], bytes[at + 1]])
}
fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
}
