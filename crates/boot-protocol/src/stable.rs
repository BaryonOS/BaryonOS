//! M1 wire contract. Validation never follows physical pointers or reclaims memory.

pub const PAGE_SIZE: u64 = 4096;
pub const ADDRESS_LIMIT: u64 = 1 << 30;
pub const KERNEL_START: u64 = 0x0200_0000;
pub const KERNEL_LEN: u64 = 0x0100_0000;
pub const ABI_MAJOR: u16 = 1;
pub const ABI_MINOR: u16 = 1;
pub const STACK_LEN: u64 = 65536;
pub const MAP_LIMIT: u64 = 1024 * 1024;
pub const MAGIC: [u8; 8] = *b"NOSBOOT\0";

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct BootInfo {
    pub magic: [u8; 8],
    pub abi_major: u16,
    pub abi_minor: u16,
    pub header_size: u32,
    pub flags: u64,
    pub memory_map_addr: u64,
    pub memory_map_len: u64,
    pub descriptor_size: u32,
    pub descriptor_version: u32,
    pub kernel_reserved_start: u64,
    pub kernel_reserved_len: u64,
    pub stack_start: u64,
    pub stack_len: u64,
    pub bootinfo_start: u64,
    pub bootinfo_len: u64,
    pub loader_start: u64,
    pub loader_len: u64,
    pub memory_map_capacity: u64,
    pub reserved0: u64,
}

const _: () = {
    use core::mem::{align_of, offset_of, size_of};
    assert!(size_of::<BootInfo>() == 128);
    assert!(align_of::<BootInfo>() == 8);
    assert!(offset_of!(BootInfo, magic) == 0);
    assert!(offset_of!(BootInfo, abi_major) == 8);
    assert!(offset_of!(BootInfo, abi_minor) == 10);
    assert!(offset_of!(BootInfo, header_size) == 12);
    assert!(offset_of!(BootInfo, flags) == 16);
    assert!(offset_of!(BootInfo, memory_map_addr) == 24);
    assert!(offset_of!(BootInfo, memory_map_len) == 32);
    assert!(offset_of!(BootInfo, descriptor_size) == 40);
    assert!(offset_of!(BootInfo, descriptor_version) == 44);
    assert!(offset_of!(BootInfo, kernel_reserved_start) == 48);
    assert!(offset_of!(BootInfo, kernel_reserved_len) == 56);
    assert!(offset_of!(BootInfo, stack_start) == 64);
    assert!(offset_of!(BootInfo, stack_len) == 72);
    assert!(offset_of!(BootInfo, bootinfo_start) == 80);
    assert!(offset_of!(BootInfo, bootinfo_len) == 88);
    assert!(offset_of!(BootInfo, loader_start) == 96);
    assert!(offset_of!(BootInfo, loader_len) == 104);
    assert!(offset_of!(BootInfo, memory_map_capacity) == 112);
    assert!(offset_of!(BootInfo, reserved0) == 120);
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Header,
    Range,
    Overlap,
    MapShape,
    Descriptor,
    Ownership,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub start: u64,
    pub end: u64,
}

impl Range {
    fn retained(start: u64, length: u64) -> Result<Self, Error> {
        let end = start.checked_add(length).ok_or(Error::Range)?;
        if start == 0
            || !start.is_multiple_of(PAGE_SIZE)
            || length == 0
            || !length.is_multiple_of(PAGE_SIZE)
            || end > ADDRESS_LIMIT
        {
            return Err(Error::Range);
        }
        Ok(Self { start, end })
    }

    fn overlaps(self, other: Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

impl BootInfo {
    pub fn retained_ranges(&self) -> Result<[Range; 5], Error> {
        Ok([
            Range::retained(self.kernel_reserved_start, self.kernel_reserved_len)?,
            Range::retained(self.stack_start, self.stack_len)?,
            Range::retained(self.bootinfo_start, self.bootinfo_len)?,
            Range::retained(self.loader_start, self.loader_len)?,
            Range::retained(self.memory_map_addr, self.memory_map_capacity)?,
        ])
    }

    /// Pure validation of the copied header; `received_addr` is never dereferenced.
    pub fn validate_header(&self, received_addr: u64) -> Result<(), Error> {
        if self.magic != MAGIC
            || self.abi_major != ABI_MAJOR
            || self.abi_minor != ABI_MINOR
            || self.header_size != 128
            || self.flags != 7
            || self.reserved0 != 0
        {
            return Err(Error::Header);
        }
        if self.kernel_reserved_start != KERNEL_START
            || self.kernel_reserved_len != KERNEL_LEN
            || self.stack_len != STACK_LEN
            || self.bootinfo_len != PAGE_SIZE
            || self.bootinfo_start != received_addr
        {
            return Err(Error::Range);
        }
        let stride = u64::from(self.descriptor_size);
        if !(40..=4096).contains(&stride)
            || !stride.is_multiple_of(8)
            || self.descriptor_version != 1
            || self.memory_map_len == 0
            || self.memory_map_len > self.memory_map_capacity
            || self.memory_map_capacity > MAP_LIMIT
            || !self.memory_map_len.is_multiple_of(stride)
        {
            return Err(Error::MapShape);
        }
        let ranges = self.retained_ranges()?;
        for (i, current) in ranges.iter().enumerate() {
            for other in &ranges[..i] {
                if current.overlaps(*other) {
                    return Err(Error::Overlap);
                }
            }
        }
        Ok(())
    }

    /// Explicit serialization also supplies an independent host byte fixture.
    pub fn to_bytes(&self) -> [u8; 128] {
        let mut bytes = [0u8; 128];
        bytes[..8].copy_from_slice(&self.magic);
        bytes[8..10].copy_from_slice(&self.abi_major.to_le_bytes());
        bytes[10..12].copy_from_slice(&self.abi_minor.to_le_bytes());
        bytes[12..16].copy_from_slice(&self.header_size.to_le_bytes());
        bytes[40..44].copy_from_slice(&self.descriptor_size.to_le_bytes());
        bytes[44..48].copy_from_slice(&self.descriptor_version.to_le_bytes());
        for (offset, value) in [
            (16, self.flags),
            (24, self.memory_map_addr),
            (32, self.memory_map_len),
            (48, self.kernel_reserved_start),
            (56, self.kernel_reserved_len),
            (64, self.stack_start),
            (72, self.stack_len),
            (80, self.bootinfo_start),
            (88, self.bootinfo_len),
            (96, self.loader_start),
            (104, self.loader_len),
            (112, self.memory_map_capacity),
            (120, self.reserved0),
        ] {
            bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Descriptor {
    pub memory_type: u32,
    pub physical_start: u64,
    pub virtual_start: u64,
    pub number_of_pages: u64,
    pub attributes: u64,
}

impl Descriptor {
    fn range(self) -> Result<Range, Error> {
        let length = self
            .number_of_pages
            .checked_mul(PAGE_SIZE)
            .ok_or(Error::Descriptor)?;
        let end = self
            .physical_start
            .checked_add(length)
            .ok_or(Error::Descriptor)?;
        if length == 0 || !self.physical_start.is_multiple_of(PAGE_SIZE) {
            return Err(Error::Descriptor);
        }
        Ok(Range {
            start: self.physical_start,
            end,
        })
    }
}

/// Safe byte view: neither descriptor padding nor firmware stride is assumed.
pub struct MemoryMap<'a> {
    bytes: &'a [u8],
    stride: usize,
}

impl<'a> MemoryMap<'a> {
    pub fn new(bytes: &'a [u8], stride: u32, version: u32) -> Result<Self, Error> {
        let stride = stride as usize;
        if !(40..=4096).contains(&stride)
            || !stride.is_multiple_of(8)
            || version != 1
            || bytes.is_empty()
            || bytes.len() > MAP_LIMIT as usize
            || !bytes.len().is_multiple_of(stride)
        {
            return Err(Error::MapShape);
        }
        let map = Self { bytes, stride };
        for i in 0..map.len() {
            let range = map.descriptor(i).ok_or(Error::MapShape)?.range()?;
            for j in 0..i {
                if range.overlaps(map.descriptor(j).ok_or(Error::MapShape)?.range()?) {
                    return Err(Error::Overlap);
                }
            }
        }
        Ok(map)
    }

    pub fn len(&self) -> usize {
        self.bytes.len() / self.stride
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn descriptor(&self, index: usize) -> Option<Descriptor> {
        let start = index.checked_mul(self.stride)?;
        let b = self.bytes.get(start..start.checked_add(40)?)?;
        let u64_at = |offset| u64::from_le_bytes(b[offset..offset + 8].try_into().unwrap());
        Some(Descriptor {
            memory_type: u32::from_le_bytes(b[..4].try_into().unwrap()),
            physical_start: u64_at(8),
            virtual_start: u64_at(16),
            number_of_pages: u64_at(24),
            attributes: u64_at(32),
        })
    }

    /// Covers an entire retained allocation, permitting adjacent unsorted descriptors.
    /// No unknown memory type is considered owned or reusable.
    pub fn covers_retained(&self, range: Range) -> bool {
        let mut cursor = range.start;
        while cursor < range.end {
            let mut next = cursor;
            for i in 0..self.len() {
                let d = self.descriptor(i).unwrap();
                let r = d.range().unwrap(); // Established by new; the byte slice is immutable.
                if matches!(d.memory_type, 1 | 2) && r.start <= cursor && cursor < r.end {
                    next = core::cmp::min(r.end, range.end);
                    break;
                }
            }
            if next == cursor {
                return false;
            }
            cursor = next;
        }
        true
    }
}

pub fn validate_memory_map(info: &BootInfo, bytes: &[u8]) -> Result<(), Error> {
    info.validate_header(info.bootinfo_start)?;
    if bytes.len() as u64 != info.memory_map_len {
        return Err(Error::MapShape);
    }
    let map = MemoryMap::new(bytes, info.descriptor_size, info.descriptor_version)?;
    for range in info.retained_ranges()? {
        if !map.covers_retained(range) {
            return Err(Error::Ownership);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::{vec, vec::Vec};

    fn header() -> BootInfo {
        BootInfo {
            magic: MAGIC,
            abi_major: ABI_MAJOR,
            abi_minor: ABI_MINOR,
            header_size: 128,
            flags: 7,
            memory_map_addr: 0x0500_0000,
            memory_map_len: 5 * 48,
            descriptor_size: 48,
            descriptor_version: 1,
            kernel_reserved_start: KERNEL_START,
            kernel_reserved_len: KERNEL_LEN,
            stack_start: 0x0400_0000,
            stack_len: STACK_LEN,
            bootinfo_start: 0x0410_0000,
            bootinfo_len: PAGE_SIZE,
            loader_start: 0x0420_0000,
            loader_len: 0x20000,
            memory_map_capacity: 0x40000,
            reserved0: 0,
        }
    }

    fn record(kind: u32, start: u64, pages: u64) -> [u8; 48] {
        let mut b = [0xa5; 48]; // Deliberately nonzero padding and extension bytes.
        b[..4].copy_from_slice(&kind.to_le_bytes());
        b[8..16].copy_from_slice(&start.to_le_bytes());
        b[16..24].copy_from_slice(&0u64.to_le_bytes());
        b[24..32].copy_from_slice(&pages.to_le_bytes());
        b[32..40].copy_from_slice(&0u64.to_le_bytes());
        b
    }

    fn valid_map(info: &BootInfo) -> Vec<u8> {
        let mut b = Vec::new();
        for r in info.retained_ranges().unwrap().iter().rev() {
            b.extend(record(2, r.start, (r.end - r.start) / PAGE_SIZE));
        }
        b
    }

    #[test]
    fn explicit_wire_fixture_and_offsets() {
        let h = header();
        let fixture: [u8; 128] = [
            78, 79, 83, 66, 79, 79, 84, 0, 1, 0, 1, 0, 128, 0, 0, 0, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 5, 0, 0, 0, 0, 240, 0, 0, 0, 0, 0, 0, 0, 48, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 2, 0, 0,
            0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 16,
            4, 0, 0, 0, 0, 0, 16, 0, 0, 0, 0, 0, 0, 0, 0, 32, 4, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0,
            0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ];
        assert_eq!(h.to_bytes(), fixture);
        assert_eq!(h.validate_header(h.bootinfo_start), Ok(()));
    }

    #[test]
    fn invalid_header_and_received_pointer_are_rejected() {
        let original = header();
        for mutate in [
            |h: &mut BootInfo| h.magic[0] ^= 1,
            |h: &mut BootInfo| h.abi_major = 2,
            |h: &mut BootInfo| h.abi_minor = 0,
            |h: &mut BootInfo| h.abi_minor = 2,
            |h: &mut BootInfo| h.header_size = 136,
            |h: &mut BootInfo| h.flags = 6,
            |h: &mut BootInfo| h.flags = 15,
            |h: &mut BootInfo| h.reserved0 = 1,
        ] {
            let mut h = original;
            mutate(&mut h);
            assert_eq!(h.validate_header(h.bootinfo_start), Err(Error::Header));
        }
        assert_eq!(original.validate_header(0x1000), Err(Error::Range));
    }

    #[test]
    fn ranges_must_be_aligned_bounded_nonoverlapping() {
        for mutate in [
            |h: &mut BootInfo| h.stack_start = u64::MAX - 4095,
            |h: &mut BootInfo| h.stack_start = ADDRESS_LIMIT,
            |h: &mut BootInfo| h.loader_start = 1,
            |h: &mut BootInfo| h.loader_len = 0,
            |h: &mut BootInfo| h.stack_len = 4096,
            |h: &mut BootInfo| h.memory_map_addr = h.stack_start,
        ] {
            let mut h = header();
            mutate(&mut h);
            assert!(h.validate_header(h.bootinfo_start).is_err());
        }
    }

    #[test]
    fn map_shape_and_stride_are_checked() {
        for stride in [0, 32, 41, 4104] {
            let mut h = header();
            h.descriptor_size = stride;
            assert_eq!(h.validate_header(h.bootinfo_start), Err(Error::MapShape));
        }
        for mutate in [
            |h: &mut BootInfo| h.descriptor_version = 2,
            |h: &mut BootInfo| h.memory_map_len = 0,
            |h: &mut BootInfo| h.memory_map_len = 41,
            |h: &mut BootInfo| h.memory_map_len = h.memory_map_capacity + 48,
            |h: &mut BootInfo| h.memory_map_capacity = MAP_LIMIT + PAGE_SIZE,
        ] {
            let mut h = header();
            mutate(&mut h);
            assert_eq!(h.validate_header(h.bootinfo_start), Err(Error::MapShape));
        }
    }

    #[test]
    fn unsorted_extended_stride_and_adjacent_descriptors_cover_allocations() {
        let mut h = header();
        let mut b = valid_map(&h);
        let last = b.len() - 48;
        b[last..].copy_from_slice(&record(1, KERNEL_START, 1));
        b.extend(record(
            2,
            KERNEL_START + PAGE_SIZE,
            KERNEL_LEN / PAGE_SIZE - 1,
        ));
        h.memory_map_len = b.len() as u64;
        assert_eq!(validate_memory_map(&h, &b), Ok(()));
        // Removing one page creates a hole even though the overall span remains plausible.
        b[last + 24..last + 32].copy_from_slice(&0u64.to_le_bytes());
        assert_eq!(validate_memory_map(&h, &b), Err(Error::Descriptor));
    }

    #[test]
    fn overlap_overflow_and_zero_pages_rejected_unknown_memory_reserved() {
        for rec in [
            record(2, 0x1000, 0),
            record(2, 0x1001, 1),
            record(2, 0x1000, u64::MAX),
        ] {
            assert!(MemoryMap::new(&rec, 48, 1).is_err());
        }
        let mut b = vec![];
        b.extend(record(2, 0x1000, 3));
        b.extend(record(2, 0x3000, 1));
        assert!(matches!(MemoryMap::new(&b, 48, 1), Err(Error::Overlap)));
        let h = header();
        let mut b = valid_map(&h);
        b[..4].copy_from_slice(&99u32.to_le_bytes());
        assert_eq!(validate_memory_map(&h, &b), Err(Error::Ownership));
    }
}
