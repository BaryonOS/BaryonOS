//! Host-testable ELF/user-ABI policy. All addresses remain integers.
use crate::{Error, PAGE};
pub const MAX_SEGMENTS: usize = 8;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Segment {
    pub va: u64,
    pub file_offset: u64,
    pub file_size: u64,
    pub memory_size: u64,
    pub writable: bool,
    pub executable: bool,
}
const EMPTY: Segment = Segment {
    va: 0,
    file_offset: 0,
    file_size: 0,
    memory_size: 0,
    writable: false,
    executable: false,
};
#[derive(Clone, Copy, Debug)]
pub struct Image {
    pub entry: u64,
    pub segments: [Segment; MAX_SEGMENTS],
    pub count: usize,
    pub pages: usize,
}
fn u16at(bytes: &[u8], at: usize) -> Result<u16, Error> {
    Ok(u16::from_le_bytes(
        bytes
            .get(at..at.checked_add(2).ok_or(Error::Range)?)
            .ok_or(Error::Range)?
            .try_into()
            .unwrap(),
    ))
}
fn u32at(bytes: &[u8], at: usize) -> Result<u32, Error> {
    Ok(u32::from_le_bytes(
        bytes
            .get(at..at.checked_add(4).ok_or(Error::Range)?)
            .ok_or(Error::Range)?
            .try_into()
            .unwrap(),
    ))
}
fn u64at(bytes: &[u8], at: usize) -> Result<u64, Error> {
    Ok(u64::from_le_bytes(
        bytes
            .get(at..at.checked_add(8).ok_or(Error::Range)?)
            .ok_or(Error::Range)?
            .try_into()
            .unwrap(),
    ))
}
pub fn parse(bytes: &[u8]) -> Result<Image, Error> {
    // p_paddr is intentionally ignored: user VAs are backed only by allocator-
    // owned frames. ELF input never chooses a physical frame or identity mapping.
    if bytes.get(..7) != Some(&b"\x7fELF\x02\x01\x01"[..])
        || u16at(bytes, 16)? != 2
        || u16at(bytes, 18)? != 62
        || u32at(bytes, 20)? != 1
        || u32at(bytes, 48)? != 0
        || u16at(bytes, 52)? != 64
        || u16at(bytes, 54)? != 56
    {
        return Err(Error::Permission);
    }
    let entry = u64at(bytes, 24)?;
    let phoff = usize::try_from(u64at(bytes, 32)?).map_err(|_| Error::Range)?;
    let count = usize::from(u16at(bytes, 56)?);
    if count == 0 || count > 32 || phoff.checked_add(count * 56).ok_or(Error::Range)? > bytes.len()
    {
        return Err(Error::Range);
    }
    let mut result = Image {
        entry,
        segments: [EMPTY; MAX_SEGMENTS],
        count: 0,
        pages: 0,
    };
    for i in 0..count {
        let at = phoff + i * 56;
        let kind = u32at(bytes, at)?;
        let flags = u32at(bytes, at + 4)?;
        if kind == 0 || kind == 4 || kind == 6 || (kind == 0x6474e551 && flags & 1 == 0) {
            continue;
        }
        if kind != 1 || !matches!(flags, 4 | 5 | 6) {
            return Err(Error::Permission);
        }
        let offset = u64at(bytes, at + 8)?;
        let va = u64at(bytes, at + 16)?;
        let filesz = u64at(bytes, at + 32)?;
        let memsz = u64at(bytes, at + 40)?;
        let align = u64at(bytes, at + 48)?;
        let end = va.checked_add(memsz).ok_or(Error::Range)?;
        if memsz == 0
            || filesz > memsz
            || offset.checked_add(filesz).ok_or(Error::Range)? > bytes.len() as u64
            || va < 0x40000000
            || end > 0x60000000
            || offset % PAGE != va % PAGE
            || align < PAGE
            || !align.is_power_of_two()
            || offset % align != va % align
            || result.count == MAX_SEGMENTS
        {
            return Err(Error::Range);
        }
        let first = va & !(PAGE - 1);
        let last = end.checked_add(PAGE - 1).ok_or(Error::Range)? & !(PAGE - 1);
        for old in &result.segments[..result.count] {
            let old_first = old.va & !(PAGE - 1);
            let old_last = (old.va + old.memory_size + PAGE - 1) & !(PAGE - 1);
            if first < old_last && old_first < last {
                return Err(Error::Overlap);
            }
        }
        result.pages = result
            .pages
            .checked_add(((last - first) / PAGE) as usize)
            .ok_or(Error::Quota)?;
        if result.pages > crate::USER_QUOTA - 32 {
            return Err(Error::Quota);
        }
        result.segments[result.count] = Segment {
            va,
            file_offset: offset,
            file_size: filesz,
            memory_size: memsz,
            writable: flags & 2 != 0,
            executable: flags & 1 != 0,
        };
        result.count += 1;
    }
    if !result.segments[..result.count]
        .iter()
        .any(|s| s.executable && entry >= s.va && entry < s.va + s.file_size)
    {
        return Err(Error::Permission);
    }
    Ok(result)
}
pub fn return_flags(flags: u64) -> u64 {
    (flags & 0x8d5) | 0x202
}
pub fn issue_pid(next: u64) -> Result<(u64, u64), Error> {
    if next == 0 {
        return Err(Error::Range);
    }
    Ok((next, next.checked_add(1).ok_or(Error::Quota)?))
}
pub fn valid_return_layout(rip: u64, rsp: u64, cs: u64, ss: u64) -> bool {
    cs == 0x23
        && ss == 0x1b
        && (0x40000000..0x60000000).contains(&rip)
        && (0x7ffef000..0x7ffff000).contains(&rsp)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageAccess {
    pub physical: u64,
    pub owner: u64,
    pub user: bool,
    pub writable: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CopyChunk {
    pub physical: u64,
    pub offset: usize,
    pub length: usize,
}
const NO_CHUNK: CopyChunk = CopyChunk {
    physical: 0,
    offset: 0,
    length: 0,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CopyPlan {
    pub chunks: [CopyChunk; 2],
    pub count: usize,
}
pub fn copy_plan(
    pointer: u64,
    length: u64,
    pid: u64,
    output: bool,
    lookup: impl FnMut(u64) -> Option<PageAccess>,
) -> Result<CopyPlan, Error> {
    bounded_copy_plan(pointer, length, 320, pid, output, lookup)
}
/// Exact sector input. This does not widen the 320-byte IPC/output contract.
pub fn sector_input_plan(
    pointer: u64,
    pid: u64,
    lookup: impl FnMut(u64) -> Option<PageAccess>,
) -> Result<CopyPlan, Error> {
    bounded_copy_plan(pointer, 512, 512, pid, false, lookup)
}
fn bounded_copy_plan(
    pointer: u64,
    length: u64,
    maximum: u64,
    pid: u64,
    output: bool,
    mut lookup: impl FnMut(u64) -> Option<PageAccess>,
) -> Result<CopyPlan, Error> {
    let end = pointer.checked_add(length).ok_or(Error::Range)?;
    if length == 0 || length > maximum || pointer < 0x40000000 || end > 0x80000000 {
        return Err(Error::Range);
    }
    let mut plan = CopyPlan {
        chunks: [NO_CHUNK; 2],
        count: 0,
    };
    let mut cursor = pointer;
    while cursor < end {
        let page = lookup(cursor).ok_or(Error::Permission)?;
        if !page.user || page.owner != pid || output && !page.writable || page.physical % PAGE != 0
        {
            return Err(Error::Permission);
        }
        let offset = (cursor & (PAGE - 1)) as usize;
        let length = core::cmp::min(PAGE - offset as u64, end - cursor) as usize;
        plan.chunks[plan.count] = CopyChunk {
            physical: page.physical,
            offset,
            length,
        };
        plan.count += 1;
        cursor += length as u64;
    }
    Ok(plan)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sector_input_validates_second_page_owner_and_preserves_ipc_limit() {
        let ro = |va: u64| {
            Some(PageAccess {
                physical: va & !(PAGE - 1),
                owner: 7,
                user: true,
                writable: false,
            })
        };
        let plan = sector_input_plan(0x40013f01, 7, ro).unwrap();
        assert_eq!(plan.count, 2);
        assert_eq!(plan.chunks[0].length, 255);
        assert_eq!(plan.chunks[1].length, 257);
        assert!(copy_plan(0x40013000, 512, 7, false, ro).is_err());
        assert!(sector_input_plan(u64::MAX - 255, 7, ro).is_err());
        assert!(sector_input_plan(0x40013f01, 7, |va| if va >= 0x40014000 {
            None
        } else {
            ro(va)
        })
        .is_err());
        assert!(sector_input_plan(0x40013f01, 8, ro).is_err());
        assert!(sector_input_plan(0x40013f01, 7, |va| ro(va).map(|mut p| {
            p.user = va < 0x40014000;
            p
        }))
        .is_err());
    }
    #[test]
    fn copy_plan_checks_entire_range_before_any_copy() {
        let good = |_: u64| {
            Some(PageAccess {
                physical: 0x1000,
                owner: 1,
                user: true,
                writable: true,
            })
        };
        for pointer in [0, 0x2000000, 0x0001000040000000, u64::MAX - 100, 0x7fffff00] {
            assert!(copy_plan(pointer, 320, 1, true, good).is_err());
        }
        assert_eq!(
            copy_plan(0x40011001, 320, 1, true, good).unwrap().chunks[0].offset,
            1
        );
        assert!(
            copy_plan(0x40011fff, 320, 1, true, |va| if va >= 0x40012000 {
                None
            } else {
                good(va)
            })
            .is_err()
        );
        let ro = |_: u64| {
            Some(PageAccess {
                physical: 0x1000,
                owner: 1,
                user: true,
                writable: false,
            })
        };
        assert!(copy_plan(0x40011000, 320, 1, true, ro).is_err());
        assert!(copy_plan(0x40011000, 320, 1, false, ro).is_ok());
        assert!(copy_plan(0x40011000, 320, 2, false, good).is_err());
        assert!(copy_plan(0x40011000, 320, 1, false, |_| Some(PageAccess {
            physical: 0x1000,
            owner: 1,
            user: false,
            writable: true
        }))
        .is_err());
    }
    fn elf() -> [u8; 8192] {
        let mut b = [0; 8192];
        b[..7].copy_from_slice(b"\x7fELF\x02\x01\x01");
        for (at, v) in [(16, 2u16), (18, 62), (52, 64), (54, 56), (56, 1)] {
            b[at..at + 2].copy_from_slice(&v.to_le_bytes());
        }
        b[20..24].copy_from_slice(&1u32.to_le_bytes());
        for (at, v) in [
            (24, 0x40000000u64),
            (32, 64),
            (72, 4096),
            (80, 0x40000000),
            (96, 8),
            (104, 4096),
            (112, 4096),
        ] {
            b[at..at + 8].copy_from_slice(&v.to_le_bytes());
        }
        b[64..68].copy_from_slice(&1u32.to_le_bytes());
        b[68..72].copy_from_slice(&5u32.to_le_bytes());
        b
    }
    #[test]
    fn valid_and_wx_entry_checks() {
        let mut b = elf();
        assert_eq!(parse(&b).unwrap().pages, 1);
        b[68..72].copy_from_slice(&7u32.to_le_bytes());
        assert!(parse(&b).is_err());
        let mut b = elf();
        b[24..32].copy_from_slice(&0x40001000u64.to_le_bytes());
        assert!(parse(&b).is_err());
    }
    #[test]
    fn legal_large_image_and_exact_user_mapping_budget() {
        let mut b = elf();
        b[104..112].copy_from_slice(&(33u64 * PAGE).to_le_bytes());
        assert_eq!(parse(&b).unwrap().pages, 33);
        b[104..112].copy_from_slice(&(2016u64 * PAGE).to_le_bytes());
        assert_eq!(parse(&b).unwrap().pages + 32, 2048);
        b[104..112].copy_from_slice(&(2017u64 * PAGE).to_le_bytes());
        assert_eq!(parse(&b).unwrap_err(), Error::Quota);
    }
    #[test]
    fn overflow_offset_and_page_overlap() {
        let mut b = elf();
        b[104..112].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(parse(&b).is_err());
        let mut b = elf();
        b[72..80].copy_from_slice(&4097u64.to_le_bytes());
        assert!(parse(&b).is_err());
        let mut b = elf();
        b[112..120].copy_from_slice(&8192u64.to_le_bytes());
        assert!(parse(&b).is_err());
        let mut b = elf();
        b[56..58].copy_from_slice(&2u16.to_le_bytes());
        let (a, c) = b.split_at_mut(120);
        c[..56].copy_from_slice(&a[64..120]);
        assert_eq!(parse(&b).unwrap_err(), Error::Overlap);
    }
    #[test]
    fn return_flags_strip_authority() {
        assert_eq!(return_flags(u64::MAX), 0xad7);
        assert_eq!(return_flags(0), 0x202);
        assert_eq!(
            return_flags(u64::MAX)
                & ((3 << 12) | (1 << 10) | (1 << 8) | (1 << 14) | (1 << 17) | (1 << 18)),
            0
        );
    }
    #[test]
    fn pids_survive_16bit_boundary_and_never_wrap() {
        let mut next = 65534u64;
        for expected in [65534, 65535, 65536, 65537] {
            let (pid, new_next) = issue_pid(next).unwrap();
            assert_eq!(pid, expected);
            next = new_next;
        }
        assert_eq!(issue_pid(u64::MAX), Err(Error::Quota));
        assert_eq!(issue_pid(0), Err(Error::Range));
    }
    #[test]
    fn user_return_layout_rejects_untrusted_stack_and_noncanonical_alias() {
        assert!(valid_return_layout(0x40000000, 0x7fffeff8, 0x23, 0x1b));
        for rsp in [0, 0x7ffeefff, 0x7ffff000, u64::MAX] {
            assert!(!valid_return_layout(0x40000000, rsp, 0x23, 0x1b));
        }
        for rip in [0, 0x2000000, 0x60000000, 0x0001000040000000] {
            assert!(!valid_return_layout(rip, 0x7fffeff8, 0x23, 0x1b));
        }
        assert!(!valid_return_layout(0x40000000, 0x7fffeff8, 8, 0x1b));
    }
}
