#![no_std]
#![forbid(unsafe_code)]

//! Bounded ELF64 load planning only. This crate cannot write physical memory.

use boot_protocol::{KERNEL_LEN, KERNEL_START, PAGE_SIZE};

pub const MAX_FILE_SIZE: usize = 32 * 1024 * 1024;
pub const MAX_SEGMENTS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Size,
    Header,
    Arithmetic,
    ProgramType,
    Segment,
    Permissions,
    Overlap,
    Entry,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Segment {
    pub offset: usize,
    pub physical_address: u64,
    pub file_size: usize,
    pub memory_size: u64,
    pub flags: u32,
}

#[derive(Debug)]
pub struct LoadPlan {
    pub entry: u64,
    segments: [Segment; MAX_SEGMENTS],
    count: usize,
}

impl LoadPlan {
    pub fn segments(&self) -> &[Segment] {
        &self.segments[..self.count]
    }
}

fn array<const N: usize>(b: &[u8], offset: usize) -> Result<[u8; N], Error> {
    b.get(offset..offset.checked_add(N).ok_or(Error::Arithmetic)?)
        .ok_or(Error::Size)?
        .try_into()
        .map_err(|_| Error::Size)
}
fn u16_at(b: &[u8], o: usize) -> Result<u16, Error> {
    Ok(u16::from_le_bytes(array(b, o)?))
}
fn u32_at(b: &[u8], o: usize) -> Result<u32, Error> {
    Ok(u32::from_le_bytes(array(b, o)?))
}
fn u64_at(b: &[u8], o: usize) -> Result<u64, Error> {
    Ok(u64::from_le_bytes(array(b, o)?))
}
fn usize_of(n: u64) -> Result<usize, Error> {
    usize::try_from(n).map_err(|_| Error::Arithmetic)
}
fn page_end(n: u64) -> Result<u64, Error> {
    Ok(n.checked_add(PAGE_SIZE - 1).ok_or(Error::Arithmetic)? & !(PAGE_SIZE - 1))
}

pub fn parse(file: &[u8]) -> Result<LoadPlan, Error> {
    if file.len() < 64 || file.len() > MAX_FILE_SIZE {
        return Err(Error::Size);
    }
    if &file[..4] != b"\x7fELF"
        || file[4] != 2
        || file[5] != 1
        || file[6] != 1
        || u16_at(file, 16)? != 2
        || u16_at(file, 18)? != 62
        || u32_at(file, 20)? != 1
        || u16_at(file, 52)? != 64
        || u16_at(file, 54)? != 56
    {
        return Err(Error::Header);
    }
    let count = usize::from(u16_at(file, 56)?);
    if count == 0 || count > MAX_SEGMENTS {
        return Err(Error::Header);
    }
    let phoff = usize_of(u64_at(file, 32)?)?;
    let phend = phoff
        .checked_add(count.checked_mul(56).ok_or(Error::Arithmetic)?)
        .ok_or(Error::Arithmetic)?;
    if phoff < 64 || phend > file.len() {
        return Err(Error::Size);
    }
    let mut plan = LoadPlan {
        entry: u64_at(file, 24)?,
        segments: [Segment::default(); MAX_SEGMENTS],
        count: 0,
    };
    let mut entry_count = 0;
    for index in 0..count {
        let b = &file[phoff + index * 56..phoff + (index + 1) * 56];
        let kind = u32_at(b, 0)?;
        let flags = u32_at(b, 4)?;
        match kind {
            0 | 4 | 6 => continue, // NULL, NOTE, PHDR are metadata, never loaded.
            0x6474_e551 if flags & 1 == 0 => continue, // Non-executable GNU_STACK.
            1 => {}
            _ => return Err(Error::ProgramType),
        }
        if !matches!(flags, 4..=6) {
            return Err(Error::Permissions);
        }
        let offset = u64_at(b, 8)?;
        let vaddr = u64_at(b, 16)?;
        let paddr = u64_at(b, 24)?;
        let filesz = u64_at(b, 32)?;
        let memsz = u64_at(b, 40)?;
        let align = u64_at(b, 48)?;
        let file_end = offset.checked_add(filesz).ok_or(Error::Arithmetic)?;
        let mem_end = paddr.checked_add(memsz).ok_or(Error::Arithmetic)?;
        if filesz > memsz
            || file_end > file.len() as u64
            || vaddr != paddr
            || align < PAGE_SIZE
            || !align.is_power_of_two()
            || offset % align != vaddr % align
        {
            return Err(Error::Segment);
        }
        if memsz == 0 {
            continue;
        }
        if paddr < KERNEL_START || mem_end > KERNEL_START + KERNEL_LEN {
            return Err(Error::Segment);
        }
        let page_start = paddr & !(PAGE_SIZE - 1);
        let page_limit = page_end(mem_end)?;
        for previous in plan.segments() {
            let old_start = previous.physical_address & !(PAGE_SIZE - 1);
            let old_end = page_end(previous.physical_address + previous.memory_size)?;
            if page_start < old_end && old_start < page_limit {
                return Err(Error::Overlap);
            }
        }
        if flags == 5 && plan.entry >= paddr && plan.entry < paddr + filesz {
            entry_count += 1;
        }
        plan.segments[plan.count] = Segment {
            offset: usize_of(offset)?,
            physical_address: paddr,
            file_size: usize_of(filesz)?,
            memory_size: memsz,
            flags,
        };
        plan.count += 1;
    }
    if plan.count == 0 || entry_count != 1 {
        return Err(Error::Entry);
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::{vec, vec::Vec};
    fn put16(b: &mut [u8], o: usize, n: u16) {
        b[o..o + 2].copy_from_slice(&n.to_le_bytes());
    }
    fn put32(b: &mut [u8], o: usize, n: u32) {
        b[o..o + 4].copy_from_slice(&n.to_le_bytes());
    }
    fn put64(b: &mut [u8], o: usize, n: u64) {
        b[o..o + 8].copy_from_slice(&n.to_le_bytes());
    }
    fn fixture() -> Vec<u8> {
        let mut b = vec![0; 0x3010];
        b[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        put16(&mut b, 16, 2);
        put16(&mut b, 18, 62);
        put32(&mut b, 20, 1);
        put64(&mut b, 24, KERNEL_START);
        put64(&mut b, 32, 64);
        put16(&mut b, 52, 64);
        put16(&mut b, 54, 56);
        put16(&mut b, 56, 3);
        for i in 0..3 {
            let p = 64 + i * 56;
            put32(&mut b, p, 1);
            put32(&mut b, p + 4, [5, 4, 6][i]);
            put64(&mut b, p + 8, (i as u64 + 1) * 4096);
            put64(&mut b, p + 16, KERNEL_START + i as u64 * 4096);
            put64(&mut b, p + 24, KERNEL_START + i as u64 * 4096);
            put64(&mut b, p + 32, 16);
            put64(&mut b, p + 40, 64);
            put64(&mut b, p + 48, 4096);
        }
        b
    }
    #[test]
    fn valid_plan_keeps_bss_and_does_not_modify_input() {
        let b = fixture();
        let before = b.clone();
        let p = parse(&b).unwrap();
        assert_eq!(p.entry, KERNEL_START);
        assert_eq!(p.segments().len(), 3);
        assert_eq!(p.segments()[2].file_size, 16);
        assert_eq!(p.segments()[2].memory_size, 64);
        assert_eq!(b, before);
    }
    #[test]
    fn every_truncation_before_required_file_end_fails() {
        let b = fixture();
        for n in 0..b.len() {
            assert!(parse(&b[..n]).is_err(), "length {n}");
        }
    }
    #[test]
    fn wrong_headers_and_program_types_fail() {
        for (offset, value) in [
            (16, 3),
            (18, 3),
            (52, 63),
            (54, 55),
            (56, 0),
            (56, 65),
            (56, 0xffff),
        ] {
            let mut b = fixture();
            put16(&mut b, offset, value);
            assert!(parse(&b).is_err());
        }
        for kind in [2, 3, 7, 0x12345678] {
            let mut b = fixture();
            put32(&mut b, 64, kind);
            assert_eq!(parse(&b).unwrap_err(), Error::ProgramType);
        }
        for offset in [0, 4, 5, 6, 20] {
            let mut b = fixture();
            b[offset] ^= 1;
            assert!(parse(&b).is_err());
        }
    }
    #[test]
    fn arithmetic_and_file_bounds_fail_closed() {
        for offset in [32, 64 + 8, 64 + 24, 64 + 40] {
            let mut b = fixture();
            put64(&mut b, offset, u64::MAX);
            assert!(parse(&b).is_err());
        }
        let mut b = fixture();
        put64(&mut b, 64 + 32, 65);
        assert!(parse(&b).is_err());
    }
    #[test]
    fn entry_must_be_in_rx_file_bytes_not_bss_or_metadata() {
        for entry in [
            0,
            KERNEL_START + 16,
            KERNEL_START + 4096,
            KERNEL_START + KERNEL_LEN,
        ] {
            let mut b = fixture();
            put64(&mut b, 24, entry);
            assert_eq!(parse(&b).unwrap_err(), Error::Entry);
        }
        let mut b = fixture();
        put64(&mut b, 64 + 32, 0);
        put64(&mut b, 64 + 40, 0);
        assert_eq!(parse(&b).unwrap_err(), Error::Entry);
    }
    #[test]
    fn permissions_alignment_addresses_and_page_overlap_fail() {
        for flags in [0, 1, 2, 3, 7, 13] {
            let mut b = fixture();
            put32(&mut b, 64 + 4, flags);
            assert_eq!(parse(&b).unwrap_err(), Error::Permissions);
        }
        for align in [0, 1, 2048, 4097] {
            let mut b = fixture();
            put64(&mut b, 64 + 48, align);
            assert!(parse(&b).is_err());
        }
        let mut b = fixture();
        put64(&mut b, 64 + 16, KERNEL_START + 1);
        assert!(parse(&b).is_err());
        let mut b = fixture();
        put64(&mut b, 64 + 40, 4097);
        assert_eq!(parse(&b).unwrap_err(), Error::Overlap);
        // Disjoint byte ranges still share a page and must not acquire mixed permissions.
        let mut b = fixture();
        let p = 64 + 56;
        put64(&mut b, p + 8, 0x2040);
        put64(&mut b, p + 16, KERNEL_START + 0x40);
        put64(&mut b, p + 24, KERNEL_START + 0x40);
        assert_eq!(parse(&b).unwrap_err(), Error::Overlap);
    }
    #[test]
    fn nonexecuting_metadata_allowed_executable_stack_denied() {
        let mut b = fixture();
        put32(&mut b, 64 + 56, 0x6474_e551);
        assert!(parse(&b).is_ok());
        put32(&mut b, 64 + 56 + 4, 5);
        assert_eq!(parse(&b).unwrap_err(), Error::ProgramType);
    }
}
