#![no_std]
#![forbid(unsafe_code)]

//! Bounded safetensors observations from a verified immutable artifact.
//! This parser grants no inference, provenance or access authority.

use artifact_store_core::{ArtifactDescriptor, ArtifactStore, READ_CHUNK_MAX};
use nieuw_os_model_store_contract::{Cancellation, ReadBudget, VerifiedArtifact};

pub const MAX_HEADER_BYTES: usize = 1024 * 1024;
pub const MAX_TENSORS: usize = 4096;
pub const MAX_RANK: usize = 8;
pub const MAX_METADATA_ENTRIES: usize = 64;
/// Fixed schema: root object, tensor object, shape/offset array; no recursion.
pub const MAX_NESTING_DEPTH: usize = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Error {
    Store(nieuw_os_model_store_contract::Error),
    InvalidHeaderLength,
    HeaderTooLarge,
    BufferTooSmall,
    InvalidUtf8,
    InvalidJson,
    UnsupportedEscape,
    DuplicateKey,
    UnknownField,
    UnsupportedDtype,
    RankLimit,
    TensorLimit,
    MetadataLimit,
    DepthLimit,
    Overflow,
    InvalidOffsets,
    SizeMismatch,
    Overlap,
    UncoveredData,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Dtype {
    Bool,
    U8,
    I8,
    U16,
    I16,
    U32,
    I32,
    U64,
    I64,
    F16,
    BF16,
    F32,
    F64,
}

impl Dtype {
    pub const fn byte_width(self) -> u64 {
        match self {
            Self::Bool | Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 | Self::F16 | Self::BF16 => 2,
            Self::U32 | Self::I32 | Self::F32 => 4,
            Self::U64 | Self::I64 | Self::F64 => 8,
        }
    }

    fn parse(value: &str) -> Result<Self, Error> {
        Ok(match value {
            "BOOL" => Self::Bool,
            "U8" => Self::U8,
            "I8" => Self::I8,
            "U16" => Self::U16,
            "I16" => Self::I16,
            "U32" => Self::U32,
            "I32" => Self::I32,
            "U64" => Self::U64,
            "I64" => Self::I64,
            "F16" => Self::F16,
            "BF16" => Self::BF16,
            "F32" => Self::F32,
            "F64" => Self::F64,
            _ => return Err(Error::UnsupportedDtype),
        })
    }
}

/// Parsed observations, not authority to read a different artifact generation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Tensor<'a> {
    name: &'a str,
    dtype: Dtype,
    shape: [u64; MAX_RANK],
    rank: usize,
    start: u64,
    end: u64,
}

impl<'a> Tensor<'a> {
    /// Scratch-table initializer; only the returned Model's slice is validated.
    pub const EMPTY: Self = Self {
        name: "",
        dtype: Dtype::U8,
        shape: [0; MAX_RANK],
        rank: 0,
        start: 0,
        end: 0,
    };

    pub const fn name(&self) -> &'a str {
        self.name
    }

    pub const fn dtype(&self) -> Dtype {
        self.dtype
    }

    pub fn shape(&self) -> &[u64] {
        &self.shape[..self.rank]
    }

    /// Half-open offsets relative to the Model's data_start, not file addresses.
    pub const fn data_offsets(&self) -> (u64, u64) {
        (self.start, self.end)
    }
}

#[derive(Debug)]
pub struct Model<'header, 'table> {
    descriptor: ArtifactDescriptor,
    data_start: u64,
    tensors: &'table [Tensor<'header>],
}

impl<'header, 'table> Model<'header, 'table> {
    pub const fn descriptor(&self) -> ArtifactDescriptor {
        self.descriptor
    }

    pub const fn data_start(&self) -> u64 {
        self.data_start
    }

    /// Sorted by data offsets, independently of header object order.
    pub const fn tensors(&self) -> &'table [Tensor<'header>] {
        self.tensors
    }
}

/// Reads only the prefix/header through the owner's VerifiedArtifact handle.
/// Positive short reads, cancellation, and requested-byte budget charging retain
/// the model-store contract. Scratch buffers are unspecified on error; only an
/// Ok Model exposes a complete validated table. The owner retains the artifact,
/// generation lifetime, revocation checks, and any later data reads.
pub fn parse<'header, 'table, S: ArtifactStore, C: Cancellation>(
    artifact: &mut VerifiedArtifact<S::Handle>,
    store: &mut S,
    budget: &mut ReadBudget,
    cancellation: &C,
    header: &'header mut [u8],
    tensors: &'table mut [Tensor<'header>],
) -> Result<Model<'header, 'table>, Error> {
    let descriptor = artifact.descriptor();
    if descriptor.length < 8 {
        return Err(Error::InvalidHeaderLength);
    }
    let mut prefix = [0; 8];
    read_exact(artifact, store, 0, budget, cancellation, &mut prefix)?;
    let length = u64::from_le_bytes(prefix);
    let data_start = 8u64.checked_add(length).ok_or(Error::Overflow)?;
    if length == 0 || data_start > descriptor.length {
        return Err(Error::InvalidHeaderLength);
    }
    if length > MAX_HEADER_BYTES as u64 {
        return Err(Error::HeaderTooLarge);
    }
    let length = length as usize;
    let header = header.get_mut(..length).ok_or(Error::BufferTooSmall)?;
    read_exact(artifact, store, 8, budget, cancellation, header)?;
    let text = core::str::from_utf8(header).map_err(|_| Error::InvalidUtf8)?;
    if !text.starts_with('{') {
        return Err(Error::InvalidJson);
    }
    let mut json = Json {
        bytes: text.as_bytes(),
        cursor: 0,
    };
    json.expect(b'{')?;
    let mut first = true;
    let mut count = 0;
    let mut metadata_seen = false;
    while json.entry(&mut first)? {
        check_cancel(cancellation)?;
        let name = json.string()?;
        json.expect(b':')?;
        if name == "__metadata__" {
            if metadata_seen {
                return Err(Error::DuplicateKey);
            }
            metadata_seen = true;
            json.metadata(cancellation)?;
        } else {
            if tensors[..count].iter().any(|tensor| tensor.name == name) {
                return Err(Error::DuplicateKey);
            }
            if count >= MAX_TENSORS || count >= tensors.len() {
                return Err(Error::TensorLimit);
            }
            tensors[count] = json.tensor(name)?;
            count += 1;
        }
    }
    if json.bytes[json.cursor..].iter().any(|byte| *byte != b' ') {
        return Err(Error::InvalidJson);
    }
    let tensors = &mut tensors[..count];
    tensors.sort_unstable_by_key(|tensor| (tensor.start, tensor.end));
    let data_length = descriptor.length - data_start;
    let mut end = 0;
    for tensor in tensors.iter() {
        check_cancel(cancellation)?;
        if tensor.end > data_length {
            return Err(Error::InvalidOffsets);
        }
        if tensor.start < end {
            return Err(Error::Overlap);
        }
        if tensor.start != end {
            return Err(Error::UncoveredData);
        }
        end = tensor.end;
    }
    if end != data_length {
        return Err(Error::UncoveredData);
    }
    check_cancel(cancellation)?;
    Ok(Model {
        descriptor,
        data_start,
        tensors,
    })
}

fn check_cancel<C: Cancellation>(cancellation: &C) -> Result<(), Error> {
    if cancellation.is_cancelled() {
        Err(Error::Store(
            nieuw_os_model_store_contract::Error::Cancelled,
        ))
    } else {
        Ok(())
    }
}

fn read_exact<S: ArtifactStore, C: Cancellation>(
    artifact: &mut VerifiedArtifact<S::Handle>,
    store: &mut S,
    mut offset: u64,
    budget: &mut ReadBudget,
    cancellation: &C,
    mut output: &mut [u8],
) -> Result<(), Error> {
    while !output.is_empty() {
        let length = output.len().min(READ_CHUNK_MAX);
        let amount = artifact
            .read_at(store, offset, budget, cancellation, &mut output[..length])
            .map_err(Error::Store)?;
        offset = offset.checked_add(amount as u64).ok_or(Error::Overflow)?;
        output = &mut output[amount..];
    }
    Ok(())
}

struct Json<'a> {
    bytes: &'a [u8],
    cursor: usize,
}

impl<'a> Json<'a> {
    fn whitespace(&mut self) {
        while self
            .bytes
            .get(self.cursor)
            .is_some_and(|byte| matches!(byte, b' ' | b'\t' | b'\r' | b'\n'))
        {
            self.cursor += 1;
        }
    }

    fn take(&mut self, byte: u8) -> bool {
        self.whitespace();
        if self.bytes.get(self.cursor) == Some(&byte) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, byte: u8) -> Result<(), Error> {
        if self.take(byte) {
            Ok(())
        } else if matches!(self.bytes.get(self.cursor), Some(b'[' | b'{')) {
            Err(Error::DepthLimit)
        } else {
            Err(Error::InvalidJson)
        }
    }

    /// Call after an opening object brace; rejects trailing commas.
    fn entry(&mut self, first: &mut bool) -> Result<bool, Error> {
        if self.take(b'}') {
            return Ok(false);
        }
        if !*first {
            self.expect(b',')?;
        }
        *first = false;
        Ok(true)
    }

    fn string(&mut self) -> Result<&'a str, Error> {
        self.expect(b'"')?;
        let start = self.cursor;
        loop {
            match self.bytes.get(self.cursor) {
                Some(b'"') => {
                    let text = core::str::from_utf8(&self.bytes[start..self.cursor])
                        .map_err(|_| Error::InvalidUtf8)?;
                    self.cursor += 1;
                    return Ok(text);
                }
                Some(b'\\') => return Err(Error::UnsupportedEscape),
                Some(0..=31) | None => return Err(Error::InvalidJson),
                Some(_) => self.cursor += 1,
            }
        }
    }

    fn integer(&mut self) -> Result<u64, Error> {
        self.whitespace();
        let start = self.cursor;
        let mut value = 0u64;
        while let Some(byte @ b'0'..=b'9') = self.bytes.get(self.cursor) {
            value = value
                .checked_mul(10)
                .and_then(|v| v.checked_add(u64::from(*byte - b'0')))
                .ok_or(Error::Overflow)?;
            self.cursor += 1;
        }
        if self.cursor == start {
            return if matches!(self.bytes.get(self.cursor), Some(b'[' | b'{')) {
                Err(Error::DepthLimit)
            } else {
                Err(Error::InvalidJson)
            };
        }
        if self.cursor - start > 1 && self.bytes[start] == b'0' {
            return Err(Error::InvalidJson);
        }
        Ok(value)
    }

    fn metadata<C: Cancellation>(&mut self, cancellation: &C) -> Result<(), Error> {
        self.expect(b'{')?;
        let mut keys = [""; MAX_METADATA_ENTRIES];
        let mut count = 0;
        let mut first = true;
        while self.entry(&mut first)? {
            check_cancel(cancellation)?;
            let key = self.string()?;
            if keys[..count].contains(&key) {
                return Err(Error::DuplicateKey);
            }
            if count == MAX_METADATA_ENTRIES {
                return Err(Error::MetadataLimit);
            }
            keys[count] = key;
            count += 1;
            self.expect(b':')?;
            self.string()?;
        }
        Ok(())
    }

    fn tensor(&mut self, name: &'a str) -> Result<Tensor<'a>, Error> {
        self.expect(b'{')?;
        let mut dtype = None;
        let mut shape = [0; MAX_RANK];
        let mut rank = 0;
        let mut offsets = None;
        let mut seen = 0;
        let mut first = true;
        while self.entry(&mut first)? {
            let key = self.string()?;
            let bit = match key {
                "dtype" => 1,
                "shape" => 2,
                "data_offsets" => 4,
                _ => return Err(Error::UnknownField),
            };
            if seen & bit != 0 {
                return Err(Error::DuplicateKey);
            }
            seen |= bit;
            self.expect(b':')?;
            match key {
                "dtype" => dtype = Some(Dtype::parse(self.string()?)?),
                "shape" => {
                    self.expect(b'[')?;
                    if !self.take(b']') {
                        loop {
                            if rank == MAX_RANK {
                                return Err(Error::RankLimit);
                            }
                            shape[rank] = self.integer()?;
                            rank += 1;
                            if self.take(b']') {
                                break;
                            }
                            self.expect(b',')?;
                        }
                    }
                }
                "data_offsets" => {
                    self.expect(b'[')?;
                    let start = self.integer()?;
                    self.expect(b',')?;
                    let end = self.integer()?;
                    self.expect(b']')?;
                    offsets = Some((start, end));
                }
                _ => return Err(Error::UnknownField),
            }
        }
        if seen != 7 {
            return Err(Error::InvalidJson);
        }
        let dtype = dtype.ok_or(Error::InvalidJson)?;
        let (start, end) = offsets.ok_or(Error::InvalidJson)?;
        let elements = if shape[..rank].contains(&0) {
            0
        } else {
            shape[..rank].iter().try_fold(1u64, |total, dim| {
                total.checked_mul(*dim).ok_or(Error::Overflow)
            })?
        };
        let size = elements
            .checked_mul(dtype.byte_width())
            .ok_or(Error::Overflow)?;
        if end < start {
            return Err(Error::InvalidOffsets);
        }
        if end - start != size {
            return Err(Error::SizeMismatch);
        }
        Ok(Tensor {
            name,
            dtype,
            shape,
            rank,
            start,
            end,
        })
    }
}
