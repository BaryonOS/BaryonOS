//! Opt-in BootInfo 1.2 graphics metadata. The original 1.1 contract is unchanged.
use crate::{BootInfo, Error, MemoryMap, PAGE_SIZE};

pub const GRAPHICS_MINOR: u16 = 2;
pub const GRAPHICS_SMP_MINOR: u16 = crate::SMP_MINOR;
pub const GRAPHICS_FLAGS: u64 = 15;
pub const EXTENSION_OFFSET: u64 = 128;
pub const SMP_EXTENSION_OFFSET: u64 = 192;
pub const FRAMEBUFFER_MAGIC: [u8; 8] = *b"NOSGOP\0\0";
pub const MAX_FRAMEBUFFER: u64 = 32 * 1024 * 1024;
pub const FRAMEBUFFER_WINDOW: u64 = 0x3800_0000;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FramebufferInfo {
    pub magic: [u8; 8],
    pub base: u64,
    pub byte_len: u64,
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    /// UEFI GOP format0 RGB or format1 BGR, each with one reserved byte.
    pub pixel_format: u32,
    pub reserved: [u64; 3],
}
const _: () = {
    use core::mem::{align_of, offset_of, size_of};
    assert!(size_of::<FramebufferInfo>() == 64);
    assert!(align_of::<FramebufferInfo>() == 8);
    assert!(offset_of!(FramebufferInfo, base) == 8);
    assert!(offset_of!(FramebufferInfo, byte_len) == 16);
    assert!(offset_of!(FramebufferInfo, width) == 24);
    assert!(offset_of!(FramebufferInfo, pixel_format) == 36);
    assert!(offset_of!(FramebufferInfo, reserved) == 40);
};
impl BootInfo {
    pub fn validate_graphics_header(&self, received_addr: u64) -> Result<(), Error> {
        if self.flags != GRAPHICS_FLAGS {
            return Err(Error::Header);
        }
        let extension_offset = match self.abi_minor {
            GRAPHICS_MINOR
                if self.smp_info_addr == 0
                    && self.smp_info_len == 0
                    && self.smp_trampoline_page == 0
                    && self.header_size == 128
                    && self.bootinfo_len == PAGE_SIZE =>
            {
                EXTENSION_OFFSET
            }
            GRAPHICS_SMP_MINOR
                if self.header_size == crate::SMP_HEADER_SIZE
                    && self.bootinfo_len == crate::BOOTINFO_SMP_LEN
                    && self.smp_info_addr
                        == received_addr
                            .checked_add(crate::SMP_INFO_OFFSET)
                            .ok_or(Error::Range)?
                    && self.smp_info_len == core::mem::size_of::<crate::smp::Info>() as u64 =>
            {
                SMP_EXTENSION_OFFSET
            }
            _ => return Err(Error::Header),
        };
        if self.reserved0
            != received_addr
                .checked_add(extension_offset)
                .ok_or(Error::Range)?
        {
            return Err(Error::Header);
        }
        let mut normal = *self;
        normal.flags = 7;
        normal.reserved0 = 0;
        if self.abi_minor == GRAPHICS_MINOR {
            normal.abi_minor = crate::ABI_MINOR;
        }
        normal.validate_header(received_addr)
    }
}
pub fn validate_graphics_memory_map(info: &BootInfo, bytes: &[u8]) -> Result<(), Error> {
    info.validate_graphics_header(info.bootinfo_start)?;
    let mut normal = *info;
    normal.flags = 7;
    normal.reserved0 = 0;
    if info.abi_minor == GRAPHICS_MINOR {
        normal.abi_minor = crate::ABI_MINOR;
    }
    crate::validate_memory_map(&normal, bytes)
}
impl FramebufferInfo {
    pub fn validate(&self) -> Result<(), Error> {
        if self.magic != FRAMEBUFFER_MAGIC || self.reserved != [0; 3] || self.pixel_format > 1 {
            return Err(Error::Header);
        }
        if self.base < 0x8000_0000
            || !self.base.is_multiple_of(PAGE_SIZE)
            || self.byte_len == 0
            || self.byte_len > MAX_FRAMEBUFFER
            || self
                .base
                .checked_add(self.byte_len)
                .is_none_or(|end| end > (1 << 52))
            || self.width == 0
            || self.width > 1920
            || self.height == 0
            || self.height > 1080
            || self.stride < self.width
            || self.stride > 4096
        {
            return Err(Error::Range);
        }
        let visible = u64::from(self.stride)
            .checked_mul(u64::from(self.height))
            .and_then(|n| n.checked_mul(4))
            .ok_or(Error::Range)?;
        if visible > self.byte_len {
            return Err(Error::Range);
        }
        Ok(())
    }
    pub fn mapped_len(&self) -> Result<u64, Error> {
        self.validate()?;
        Ok(
            (u64::from(self.stride) * u64::from(self.height) * 4 + PAGE_SIZE - 1)
                & !(PAGE_SIZE - 1),
        )
    }
    /// A framebuffer may overlap only UEFI MemoryMappedIO (type11), or a hole
    /// outside known RAM. Never treat ordinary/reserved/runtime/ACPI RAM as pixels.
    pub fn validate_memory_map(&self, map: &MemoryMap<'_>) -> Result<(), Error> {
        self.validate()?;
        let end = self.base + self.mapped_len()?;
        for index in 0..map.len() {
            let d = map.descriptor(index).ok_or(Error::Descriptor)?;
            let other_end = d
                .physical_start
                .checked_add(
                    d.number_of_pages
                        .checked_mul(PAGE_SIZE)
                        .ok_or(Error::Descriptor)?,
                )
                .ok_or(Error::Descriptor)?;
            if self.base < other_end && d.physical_start < end && d.memory_type != 11 {
                return Err(Error::Overlap);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn header() -> BootInfo {
        BootInfo {
            magic: crate::MAGIC,
            abi_major: crate::ABI_MAJOR,
            abi_minor: 2,
            header_size: 128,
            flags: 15,
            memory_map_addr: 0x0500_0000,
            memory_map_len: 200,
            descriptor_size: 40,
            descriptor_version: 1,
            kernel_reserved_start: crate::KERNEL_START,
            kernel_reserved_len: crate::KERNEL_LEN,
            stack_start: 0x0400_0000,
            stack_len: crate::STACK_LEN,
            bootinfo_start: 0x0410_0000,
            bootinfo_len: PAGE_SIZE,
            loader_start: 0x0420_0000,
            loader_len: 0x20000,
            memory_map_capacity: 0x40000,
            reserved0: 0x0410_0080,
            smp_info_addr: 0,
            smp_info_len: 0,
            smp_trampoline_page: 0,
        }
    }
    #[test]
    fn graphics_header_is_exact_opt_in_and_preserves_legacy_rejection() {
        let original = header();
        assert_eq!(
            original.validate_graphics_header(original.bootinfo_start),
            Ok(())
        );
        assert_eq!(
            original.validate_header(original.bootinfo_start),
            Err(Error::Header)
        );
        for mutate in [
            |i: &mut BootInfo| i.abi_minor = 1,
            |i: &mut BootInfo| i.abi_minor = 3,
            |i: &mut BootInfo| i.flags = 7,
            |i: &mut BootInfo| i.flags = 31,
            |i: &mut BootInfo| i.reserved0 = 0,
            |i: &mut BootInfo| i.reserved0 += 8,
            |i: &mut BootInfo| i.header_size = 192,
            |i: &mut BootInfo| i.abi_major = 2,
        ] {
            let mut i = original;
            mutate(&mut i);
            assert!(i.validate_graphics_header(i.bootinfo_start).is_err());
        }

        let mut overlapping = original;
        overlapping.stack_start = crate::KERNEL_START;
        assert_eq!(
            overlapping.validate_graphics_header(overlapping.bootinfo_start),
            Err(Error::Overlap)
        );
        assert!(original
            .validate_graphics_header(original.bootinfo_start + PAGE_SIZE)
            .is_err());
    }
    #[test]
    fn graphics_and_smp_extensions_have_disjoint_versioned_offsets() {
        let mut info = header();
        info.abi_minor = GRAPHICS_SMP_MINOR;
        info.header_size = crate::SMP_HEADER_SIZE;
        info.bootinfo_len = crate::BOOTINFO_SMP_LEN;
        info.reserved0 = info.bootinfo_start + SMP_EXTENSION_OFFSET;
        info.smp_info_addr = info.bootinfo_start + crate::SMP_INFO_OFFSET;
        info.smp_info_len = core::mem::size_of::<crate::smp::Info>() as u64;
        info.smp_trampoline_page = 0x8000;
        assert_eq!(info.validate_graphics_header(info.bootinfo_start), Ok(()));
        info.reserved0 = info.bootinfo_start + EXTENSION_OFFSET;
        assert_eq!(
            info.validate_graphics_header(info.bootinfo_start),
            Err(Error::Header)
        );
    }
    #[test]
    fn graphics_map_keeps_all_five_owned_retained_allocations() {
        let info = header();
        let mut bytes = [0u8; 200];
        for (index, r) in info.retained_ranges().unwrap().into_iter().enumerate() {
            if r.start == r.end {
                continue;
            }
            let at = index * 40;
            bytes[at..at + 4].copy_from_slice(&2u32.to_le_bytes());
            bytes[at + 8..at + 16].copy_from_slice(&r.start.to_le_bytes());
            bytes[at + 24..at + 32].copy_from_slice(&((r.end - r.start) / PAGE_SIZE).to_le_bytes());
        }
        assert_eq!(validate_graphics_memory_map(&info, &bytes), Ok(()));
        bytes[..4].copy_from_slice(&7u32.to_le_bytes());
        assert_eq!(
            validate_graphics_memory_map(&info, &bytes),
            Err(Error::Ownership)
        );
    }
    fn valid() -> FramebufferInfo {
        FramebufferInfo {
            magic: FRAMEBUFFER_MAGIC,
            base: 0x8000_0000,
            byte_len: 4 * 1024 * 1024,
            width: 800,
            height: 600,
            stride: 800,
            pixel_format: 1,
            reserved: [0; 3],
        }
    }
    #[test]
    fn framebuffer_bounds_formats_and_checked_span() {
        assert_eq!(valid().mapped_len(), Ok(1921024));
        let mut exact = valid();
        exact.byte_len = 800 * 600 * 4;
        assert_eq!(exact.mapped_len(), Ok(1921024));
        for mutate in [
            |i: &mut FramebufferInfo| i.width = 0,
            |i: &mut FramebufferInfo| i.width = 1921,
            |i: &mut FramebufferInfo| i.height = 1081,
            |i: &mut FramebufferInfo| i.stride = 799,
            |i: &mut FramebufferInfo| i.stride = u32::MAX,
            |i: &mut FramebufferInfo| i.byte_len = 4096,
            |i: &mut FramebufferInfo| i.byte_len = u64::MAX,
            |i: &mut FramebufferInfo| i.base = u64::MAX - 4095,
            |i: &mut FramebufferInfo| i.base = 0x1000,
            |i: &mut FramebufferInfo| i.base += 1,
            |i: &mut FramebufferInfo| i.pixel_format = 2,
            |i: &mut FramebufferInfo| i.reserved[2] = 1,
        ] {
            let mut i = valid();
            mutate(&mut i);
            assert!(i.validate().is_err());
        }
    }
    #[test]
    fn framebuffer_cannot_overlap_ram_of_any_ordinary_type() {
        for kind in [0u32, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 99] {
            let mut bytes = [0; 40];
            bytes[..4].copy_from_slice(&kind.to_le_bytes());
            bytes[8..16].copy_from_slice(&0x8000_0000u64.to_le_bytes());
            bytes[24..32].copy_from_slice(&1024u64.to_le_bytes());
            let map = MemoryMap::new(&bytes, 40, 1).unwrap();
            assert_eq!(valid().validate_memory_map(&map).is_ok(), kind == 11);
        }
    }
    #[test]
    fn framebuffer_hole_adjacency_and_rounded_last_page_are_checked() {
        let mut f = valid();
        f.byte_len = 800 * 600 * 4;
        for (start, pages, okay) in [
            (0x1000u64, 1u64, true),
            (f.base - PAGE_SIZE, 1, true),
            (f.base + f.mapped_len().unwrap(), 1, true),
            (f.base + f.mapped_len().unwrap() - PAGE_SIZE, 1, false),
        ] {
            let mut bytes = [0u8; 40];
            bytes[..4].copy_from_slice(&7u32.to_le_bytes());
            bytes[8..16].copy_from_slice(&start.to_le_bytes());
            bytes[24..32].copy_from_slice(&pages.to_le_bytes());
            let map = MemoryMap::new(&bytes, 40, 1).unwrap();
            assert_eq!(f.validate_memory_map(&map).is_ok(), okay);
        }
    }
}
