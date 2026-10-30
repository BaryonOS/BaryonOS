#![forbid(unsafe_code)]
//! ABI1.7 private heap suffixes. This is no allocator, shared-memory grant,
//! physical address or proof of general 8GiB-profile application capacity.
use crate::Error;

pub const PAGE_BYTES: u64 = 4096;
pub const HEAP_BASE: u64 = 0x6000_0000;
pub const INITIAL_HEAP_BYTES: u64 = 65536;
/// Existing process quota includes text, data, stack and heap together.
pub const MAX_PROCESS_PAGES: u64 = 2048;
pub const HEAP_LIMIT: u64 = HEAP_BASE + MAX_PROCESS_PAGES * PAGE_BYTES;
/// ABI1.7 physical-frame exhaustion; -9 retains its existing Closed meaning.
pub const OUT_OF_MEMORY: i64 = -16;

/// Fresh private RW/NX pages returned by the native kernel. Not Clone/Copy;
/// dropping this descriptor does not unmap memory. There is no shrink contract.
/// Raw-pointer access still requires callers to establish Rust alias/lifetime
/// rules; this type deliberately provides no safe mutable slice or allocator.
#[derive(Debug)]
pub struct OwnedHeapPages {
    base: u64,
    pages: u64,
}
impl OwnedHeapPages {
    pub const fn base_address(&self) -> u64 {
        self.base
    }
    pub const fn page_count(&self) -> u64 {
        self.pages
    }
    pub const fn len_bytes(&self) -> u64 {
        self.pages * PAGE_BYTES
    }
    pub fn as_ptr(&self) -> *mut u8 {
        self.base as *mut u8
    }
}

#[cfg_attr(not(all(target_arch = "x86_64", target_os = "none")), allow(dead_code))]
pub(crate) fn decode_growth(base: u64, pages: u64) -> Result<OwnedHeapPages, Error> {
    let bytes = pages.checked_mul(PAGE_BYTES).ok_or(Error::InvalidReturn)?;
    let end = base.checked_add(bytes).ok_or(Error::InvalidReturn)?;
    if pages == 0
        || pages > MAX_PROCESS_PAGES
        || base < HEAP_BASE + INITIAL_HEAP_BYTES
        || !base.is_multiple_of(PAGE_BYTES)
        || end > HEAP_LIMIT
    {
        return Err(Error::InvalidReturn);
    }
    Ok(OwnedHeapPages { base, pages })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn fresh_suffix_geometry_and_raw_access_are_preserved() {
        let block = decode_growth(HEAP_BASE + INITIAL_HEAP_BYTES, 33).unwrap();
        assert_eq!(block.base_address(), 0x60010000);
        assert_eq!(block.page_count(), 33);
        assert_eq!(block.len_bytes(), 33 * 4096);
        assert_eq!(block.as_ptr() as u64, block.base_address());
        assert_eq!(
            decode_growth(HEAP_LIMIT - PAGE_BYTES, 1)
                .unwrap()
                .len_bytes(),
            PAGE_BYTES
        );
    }
    #[test]
    fn malformed_or_nonprivate_kernel_extents_fail_closed() {
        for (base, pages) in [
            (HEAP_BASE + INITIAL_HEAP_BYTES, 0),
            (HEAP_BASE, 1),
            (HEAP_BASE + INITIAL_HEAP_BYTES + 1, 1),
            (HEAP_LIMIT, 1),
            (HEAP_LIMIT - PAGE_BYTES, 2),
            (0x3f100000, 1),
            (0x7ffef000, 1),
            (HEAP_BASE + INITIAL_HEAP_BYTES, u64::MAX),
            (u64::MAX - PAGE_BYTES + 1, 1),
            (HEAP_BASE + INITIAL_HEAP_BYTES, MAX_PROCESS_PAGES + 1),
        ] {
            assert!(
                matches!(decode_growth(base, pages), Err(Error::InvalidReturn)),
                "{base:x}/{pages}"
            );
        }
        assert_eq!(OUT_OF_MEMORY, -16);
        assert_eq!(crate::decode_return(-9, 0), Err(Error::Kernel(-9)));
        assert_eq!(
            crate::decode_return(OUT_OF_MEMORY, 0),
            Err(Error::Kernel(OUT_OF_MEMORY))
        );
    }
}
