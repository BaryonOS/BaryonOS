//! Only 4-KiB leaves. Tables are exclusive PageTable(0) allocations, never freed
//! while active. Mapping changes are bootstrap-only until M3 supplies invalidation.
use super::{zero, FRAME_COUNT};
use core::ptr;
use kernel_core::{Alias, Frames, Owner, PAGE};
const PRESENT: u64 = 1;
const WRITE: u64 = 2;
const USER: u64 = 4;
const NX: u64 = 1 << 63;
const ADDRESS: u64 = 0x000f_ffff_ffff_f000;
const MAX_TABLES: usize = 256;
pub const SCRATCH: u64 = 0x3fffe000;
const FIRMWARE_SCRATCH: u64 = 0x3fffd000;
static mut SCRATCH_ENTRY: u64 = 0;
static mut EXECUTABLE: [bool; FRAME_COUNT] = [false; FRAME_COUNT];
static mut OBSERVED: [Alias; FRAME_COUNT] = [Alias {
    writable: 0,
    executable: 0,
    total: 0,
}; FRAME_COUNT];
/// Read-only plan for strictly increasing RW/NX mappings. Keys identify absent
/// intermediate table paths, not frames or authority. One IF=0 transaction owns
/// each plan until commit/rollback; no user pointer is retained here.
#[cfg(feature = "process-memory")]
pub(super) struct MapPlan {
    keys: [u64; MAX_TABLES],
    count: usize,
    last_va: u64,
}
#[cfg(feature = "process-memory")]
impl MapPlan {
    pub const fn new() -> Self {
        Self {
            keys: [0; MAX_TABLES],
            count: 0,
            last_va: 0,
        }
    }
    pub fn clear(&mut self) {
        self.keys[..self.count].fill(0);
        self.count = 0;
        self.last_va = 0;
    }
    pub fn tables(&self) -> usize {
        self.count
    }
    pub fn is_clear(&self) -> bool {
        self.count == 0 && self.last_va == 0 && self.keys.iter().all(|key| *key == 0)
    }
    fn add(&mut self, va: u64, shift: u8) -> Result<(), kernel_core::Error> {
        let key = (va & !((1u64 << shift) - 1)) | u64::from(shift);
        if self.keys[..self.count].contains(&key) {
            return Ok(());
        }
        if self.count == MAX_TABLES {
            return Err(kernel_core::Error::Quota);
        }
        self.keys[self.count] = key;
        self.count += 1;
        Ok(())
    }
}
pub struct Paging {
    pub root: u64,
    tables: [u64; MAX_TABLES],
    count: usize,
    asid: u16,
    #[cfg(feature = "framebuffer")]
    framebuffer: Option<(u64, u64)>,
    #[cfg(feature = "native-smp")]
    lapic: Option<u64>,
}
/// Fixed supervisor UC/NX window for the xAPIC register page (native-smp only).
#[cfg(feature = "native-smp")]
pub const LAPIC_WINDOW: u64 = 0x3f1c0000;
/// Supervisor RW/NX RAM window whose frame the native-smp shootdown proof
/// replaces while other CPUs may still cache the old translation.
#[cfg(feature = "native-smp")]
pub const SHOOTDOWN_WINDOW: u64 = 0x3f1d0000;
impl Paging {
    /// Fallible read-only seam before existing scalar map commit. The caller
    /// supplies fresh owned data frames, ordered unique VAs and one exclusive
    /// plan. Every missing path is deduplicated across the complete batch.
    #[cfg(feature = "process-memory")]
    pub(super) fn preflight_private_rw(
        &self,
        frames: &Frames<FRAME_COUNT>,
        plan: &mut MapPlan,
        va: u64,
        pa: u64,
        user: bool,
    ) -> Result<(), kernel_core::Error> {
        use kernel_core::Error;
        if va == 0
            || va >= 0x80000000
            || !va.is_multiple_of(PAGE)
            || pa == 0
            || !pa.is_multiple_of(PAGE)
            || pa >= PAGE * FRAME_COUNT as u64
            || va <= plan.last_va
            || user != (self.asid != 0)
            || user && va < 0x40000000
            || !user && va >= 0x40000000
        {
            return Err(Error::Range);
        }
        if !matches!(frames.owner(pa)?, Owner::User(_, kernel_core::Kind::Data))
            || frames.alias(pa)?
                != (Alias {
                    writable: 0,
                    executable: 0,
                    total: 0,
                })
        {
            return Err(Error::Owner);
        }
        let mut table = self.root;
        let mut missing = false;
        for shift in [39u8, 30, 21] {
            if missing {
                plan.add(va, shift)?;
                continue;
            }
            if frames.owner(table)? != Owner::PageTable(self.asid)
                || !self.tables[..self.count].contains(&table)
            {
                return Err(Error::Owner);
            }
            let entry = read(table, ((va >> shift) & 511) as usize);
            if entry & PRESENT == 0 {
                if entry != 0 {
                    return Err(Error::Overlap);
                }
                missing = true;
                plan.add(va, shift)?;
            } else if entry & (128 | NX) == 0 && entry & WRITE != 0 && (!user || entry & USER != 0)
            {
                table = entry & ADDRESS;
            } else {
                return Err(Error::Permission);
            }
        }
        if !missing {
            if frames.owner(table)? != Owner::PageTable(self.asid)
                || !self.tables[..self.count].contains(&table)
            {
                return Err(Error::Owner);
            }
            if read(table, ((va >> 12) & 511) as usize) != 0 {
                return Err(Error::Overlap);
            }
        }
        if self
            .count
            .checked_add(plan.count)
            .is_none_or(|count| count > MAX_TABLES)
        {
            return Err(Error::Quota);
        }
        plan.last_va = va;
        Ok(())
    }
    #[cfg(any(feature = "milestone-m5", feature = "native-keyring"))]
    pub fn map_mmio(
        &mut self,
        frames: &mut Frames<FRAME_COUNT>,
        va: u64,
        pa: u64,
    ) -> Result<(), kernel_core::Error> {
        let end = if cfg!(feature = "native-system-net") {
            0x3f160000
        } else {
            0x3f100000
        };
        if !(0x3f000000..end).contains(&va) || va % PAGE != 0 || pa % PAGE != 0 || pa < 0x80000000 {
            return Err(kernel_core::Error::Range);
        }
        self.map_device_page(frames, va, pa)
    }
    #[cfg(feature = "native-tpm-crb")]
    pub fn map_tpm_crb(
        &mut self,
        frames: &mut Frames<FRAME_COUNT>,
    ) -> Result<(), kernel_core::Error> {
        self.map_device_page(frames, 0x3f180000, nieuw_os_tpm_crb::transport::BASE)
    }
    #[cfg(feature = "framebuffer")]
    pub fn map_framebuffer(
        &mut self,
        frames: &mut Frames<FRAME_COUNT>,
        framebuffer: &boot_protocol::graphics::FramebufferInfo,
        map: &boot_protocol::MemoryMap<'_>,
    ) -> Result<(), kernel_core::Error> {
        framebuffer
            .validate_memory_map(map)
            .map_err(|_| kernel_core::Error::Range)?;
        let length = framebuffer
            .mapped_len()
            .map_err(|_| kernel_core::Error::Range)?;
        if self.framebuffer.is_some() {
            return Err(kernel_core::Error::Overlap);
        }
        let start = boot_protocol::graphics::FRAMEBUFFER_WINDOW;
        // Entire range is checked before any PTE; no user-selected physical
        // address reaches this bootstrap-only, exact framebuffer mapping API.
        for offset in (0..length).step_by(PAGE as usize) {
            if self.leaf(start + offset) != 0 {
                return Err(kernel_core::Error::Overlap);
            }
        }
        for offset in (0..length).step_by(PAGE as usize) {
            self.map_device_page(frames, start + offset, framebuffer.base + offset)?;
        }
        self.framebuffer = Some((framebuffer.base, length));
        Ok(())
    }
    /// Bootstrap-only exact mapping of the IA32_APIC_BASE register page. Must
    /// run before `map_tables` because a new PT is zeroed through firmware
    /// identity mappings. The base is never taken from ACPI or any request.
    #[cfg(feature = "native-smp")]
    pub fn map_lapic(
        &mut self,
        frames: &mut Frames<FRAME_COUNT>,
        base: u64,
    ) -> Result<(), kernel_core::Error> {
        if self.lapic.is_some() || self.leaf(LAPIC_WINDOW) != 0 {
            return Err(kernel_core::Error::Overlap);
        }
        if base == 0 || !base.is_multiple_of(PAGE) || base < PAGE * FRAME_COUNT as u64 {
            return Err(kernel_core::Error::Range);
        }
        self.map_device_page(frames, LAPIC_WINDOW, base)?;
        self.lapic = Some(base);
        Ok(())
    }
    /// Leaf slot of an existing supervisor 4-KiB mapping; never allocates.
    #[cfg(feature = "native-smp")]
    fn existing_leaf(&self, va: u64) -> Result<(u64, usize, u64), kernel_core::Error> {
        let mut table = self.root;
        for shift in [39, 30, 21] {
            let entry = read(table, ((va >> shift) & 511) as usize);
            if entry & PRESENT == 0 || entry & (USER | 128) != 0 {
                return Err(kernel_core::Error::Range);
            }
            table = entry & ADDRESS;
        }
        let index = ((va >> 12) & 511) as usize;
        let leaf = read(table, index);
        if leaf & (PRESENT | WRITE | USER | NX) != PRESENT | WRITE | NX {
            return Err(kernel_core::Error::Permission);
        }
        Ok((table, index, leaf & ADDRESS))
    }
    /// Point SHOOTDOWN_WINDOW at `new_pa` and invalidate it on THIS CPU only.
    /// The old frame's alias metadata is deliberately NOT released: other CPUs
    /// may still cache it. Call `release_stale_alias` only after every CPU that
    /// may hold the translation has acknowledged its invalidation.
    #[cfg(feature = "native-smp")]
    pub fn remap_window(
        &mut self,
        frames: &mut Frames<FRAME_COUNT>,
        new_pa: u64,
    ) -> Result<u64, kernel_core::Error> {
        let (table, index, old_pa) = self.existing_leaf(SHOOTDOWN_WINDOW)?;
        if new_pa == old_pa {
            return Err(kernel_core::Error::Overlap);
        }
        frames.map(new_pa, true, false, false)?;
        write(table, index, new_pa | PRESENT | WRITE | NX);
        invalidate(SHOOTDOWN_WINDOW);
        Ok(old_pa)
    }
    /// Release one RW/NX alias of `old_pa` after the GLOBAL invalidation of
    /// that alias completed (its PTE is already cleared or replaced).
    #[cfg(feature = "native-smp")]
    pub fn release_stale_alias(
        &mut self,
        frames: &mut Frames<FRAME_COUNT>,
        old_pa: u64,
    ) -> Result<(), kernel_core::Error> {
        frames.unmap(old_pa, true, false)
    }
    /// Clear a bootstrap identity RW/NX alias (va == pa) and invalidate it on
    /// THIS CPU. As with `remap_window`, the alias metadata is kept: with one
    /// shared CR3 another CPU may hold (even speculatively) this translation,
    /// so `release_stale_alias` must wait for the shootdown acknowledgements.
    #[cfg(feature = "native-smp")]
    pub fn clear_identity(&mut self, pa: u64) -> Result<(), kernel_core::Error> {
        let (table, index, mapped) = self.existing_leaf(pa)?;
        if mapped != pa {
            return Err(kernel_core::Error::Range);
        }
        write(table, index, 0);
        invalidate(pa);
        Ok(())
    }
    #[cfg(any(
        feature = "milestone-m5",
        feature = "framebuffer",
        feature = "native-keyring",
        feature = "native-tpm-crb",
        feature = "native-smp"
    ))]
    fn map_device_page(
        &mut self,
        frames: &mut Frames<FRAME_COUNT>,
        va: u64,
        pa: u64,
    ) -> Result<(), kernel_core::Error> {
        // SAFETY: CPUID queries integer CPU capabilities only. Do not install a
        // PTE whose physical bits exceed this CPU's MAXPHYADDR, even if PCI
        // metadata itself permits a wider BAR; missing/unknown widths fail closed.
        // SAFETY: CPUID is available in long mode and the leaf is checked before
        // querying the physical-address-width field.
        let width = unsafe {
            if core::arch::x86_64::__cpuid(0x80000000).eax < 0x80000008 {
                return Err(kernel_core::Error::Range);
            }
            core::arch::x86_64::__cpuid(0x80000008).eax as u8
        };
        if !kernel_core::physical_page_fits(pa, width) {
            return Err(kernel_core::Error::Range);
        }
        let (lo, hi): (u32, u32);
        // SAFETY: CPL0 read-only PAT query; require index3=UC rather than changing
        // global cache attributes inherited from firmware.
        unsafe {
            core::arch::asm!("rdmsr",in("ecx") 0x277u32,out("eax") lo,out("edx") hi,options(nostack));
        }
        if ((u64::from(hi) << 32 | u64::from(lo)) >> 24) & 255 != 0 {
            return Err(kernel_core::Error::Permission);
        }
        let mut table = self.root;
        for shift in [39, 30, 21] {
            let index = ((va >> shift) & 511) as usize;
            let mut e = read(table, index);
            if e & PRESENT == 0 {
                e = self.table(frames) | PRESENT | WRITE;
                write(table, index, e);
            }
            if e & (USER | 128 | NX) != 0 {
                return Err(kernel_core::Error::Permission);
            }
            table = e & ADDRESS;
        }
        let index = ((va >> 12) & 511) as usize;
        if read(table, index) != 0 {
            return Err(kernel_core::Error::Overlap);
        }
        // Separate MMIO class: never RAM/user frames. PCD|PWT selects checked UC.
        write(table, index, pa | PRESENT | WRITE | NX | 8 | 16);
        invalidate(va);
        Ok(())
    }
    pub fn new(frames: &mut Frames<FRAME_COUNT>) -> Self {
        let root = frames
            .allocate(Owner::PageTable(0), zero)
            .expect("root table");
        let mut result = Self {
            root,
            tables: [0; MAX_TABLES],
            count: 1,
            asid: 0,
            #[cfg(feature = "framebuffer")]
            framebuffer: None,
            #[cfg(feature = "native-smp")]
            lapic: None,
        };
        result.tables[0] = root;
        result
    }
    fn table(&mut self, frames: &mut Frames<FRAME_COUNT>) -> u64 {
        assert!(self.count < MAX_TABLES);
        let address = frames
            .allocate(Owner::PageTable(self.asid), zero_page)
            .expect("table capacity");
        self.tables[self.count] = address;
        self.count += 1;
        address
    }
    pub fn map(
        &mut self,
        frames: &mut Frames<FRAME_COUNT>,
        va: u64,
        pa: u64,
        writable: bool,
        executable: bool,
    ) {
        self.map_kind(frames, va, pa, writable, executable, false);
    }

    /// Reserve the temporary firmware mapping's page-table path before installing
    /// the new root, while newly allocated tables are still identity-accessible.
    pub(super) fn prepare_firmware_scratch(&mut self, frames: &mut Frames<FRAME_COUNT>) {
        let mut table = self.root;
        for shift in [39, 30, 21] {
            let index = ((FIRMWARE_SCRATCH >> shift) & 511) as usize;
            let mut entry = read(table, index);
            if entry & PRESENT == 0 {
                assert_eq!(entry, 0);
                entry = self.table(frames) | PRESENT | WRITE;
                write(table, index, entry);
            }
            assert_eq!(entry & (USER | 128 | NX), 0);
            assert_ne!(entry & WRITE, 0);
            table = entry & ADDRESS;
        }
        assert_eq!(read(table, ((FIRMWARE_SCRATCH >> 12) & 511) as usize), 0);
    }

    /// Read firmware-owned ACPI bytes through one temporary supervisor RO/NX
    /// mapping. The caller must first validate the whole span against the UEFI
    /// ACPI reclaim/NVS descriptors; no allocator ownership or alias is created.
    ///
    /// # Safety
    /// `address..address + output.len()` must lie entirely inside UEFI ACPI
    /// reclaim/NVS memory; reading other physical ranges (e.g. MMIO) can have
    /// side effects or raise machine checks.
    pub(super) unsafe fn read_firmware(
        &mut self,
        address: u64,
        output: &mut [u8],
    ) -> Result<(), kernel_core::Error> {
        let end = address
            .checked_add(output.len() as u64)
            .filter(|end| *end <= (1 << 52))
            .ok_or(kernel_core::Error::Range)?;
        if output.is_empty() {
            return Ok(());
        }
        // SAFETY: CPUID is available in long mode and the leaf is checked before
        // querying the physical-address-width field.
        let width = unsafe {
            if core::arch::x86_64::__cpuid(0x80000000).eax < 0x80000008 {
                return Err(kernel_core::Error::Range);
            }
            core::arch::x86_64::__cpuid(0x80000008).eax as u8
        };
        let mut current = address;
        let mut copied = 0usize;
        while current < end {
            let page = current & !(PAGE - 1);
            if !kernel_core::physical_page_fits(page, width) {
                return Err(kernel_core::Error::Range);
            }
            let offset = (current - page) as usize;
            let length = core::cmp::min(PAGE as usize - offset, output.len() - copied);
            let mut table = self.root;
            for shift in [39, 30, 21] {
                let index = ((FIRMWARE_SCRATCH >> shift) & 511) as usize;
                let entry = read(table, index);
                if entry & PRESENT == 0 || entry & (USER | 128 | NX) != 0 {
                    return Err(kernel_core::Error::Permission);
                }
                if entry & WRITE == 0 {
                    return Err(kernel_core::Error::Permission);
                }
                table = entry & ADDRESS;
            }
            let index = ((FIRMWARE_SCRATCH >> 12) & 511) as usize;
            if read(table, index) != 0 {
                return Err(kernel_core::Error::Overlap);
            }
            write(table, index, page | PRESENT | NX);
            invalidate(FIRMWARE_SCRATCH);
            // SAFETY: The caller validated this ACPI memory span against the UEFI
            // map. The temporary leaf is supervisor, read-only, NX, and removed
            // before returning; output is a valid exclusive kernel slice.
            unsafe {
                ptr::copy_nonoverlapping(
                    (FIRMWARE_SCRATCH + offset as u64) as *const u8,
                    output.as_mut_ptr().add(copied),
                    length,
                );
            }
            write(table, index, 0);
            invalidate(FIRMWARE_SCRATCH);
            copied += length;
            current += length as u64;
        }
        Ok(())
    }
    #[cfg(feature = "milestone-m3")]
    pub fn map_user(
        &mut self,
        frames: &mut Frames<FRAME_COUNT>,
        va: u64,
        pa: u64,
        writable: bool,
        executable: bool,
    ) {
        assert!((0x40000000..0x80000000).contains(&va));
        self.map_kind(frames, va, pa, writable, executable, true);
    }
    fn map_kind(
        &mut self,
        frames: &mut Frames<FRAME_COUNT>,
        va: u64,
        pa: u64,
        writable: bool,
        executable: bool,
        user: bool,
    ) {
        assert!(va != 0 && va < 0x80000000 && va % PAGE == 0 && pa % PAGE == 0);
        let mut table = self.root;
        for shift in [39, 30, 21] {
            let index = ((va >> shift) & 511) as usize;
            let mut entry = read(table, index);
            if entry & PRESENT == 0 {
                let next = self.table(frames);
                entry = next | PRESENT | WRITE | if user { USER } else { 0 };
                write(table, index, entry);
            }
            assert_eq!(entry & (128 | NX), 0);
            if user {
                assert_ne!(entry & USER, 0);
            }
            table = entry & ADDRESS;
        }
        let index = ((va >> 12) & 511) as usize;
        assert_eq!(read(table, index) & PRESENT, 0);
        frames
            .map(pa, writable, executable, user)
            .expect("physical alias W^X");
        if executable {
            // SAFETY: Exclusive single-CPU mapper, IF=0; this deny bitmap covers
            // every executable alias before a hardware executable leaf exists.
            unsafe {
                ptr::addr_of_mut!(EXECUTABLE)
                    .cast::<bool>()
                    .add((pa / PAGE) as usize)
                    .write(true);
            }
        }
        write(
            table,
            index,
            pa | PRESENT
                | if writable { WRITE } else { 0 }
                | if user { USER } else { 0 }
                | if executable { 0 } else { NX },
        );
    }
    #[cfg(feature = "milestone-m3")]
    pub fn new_user(frames: &mut Frames<FRAME_COUNT>, kernel: &Self, asid: u16) -> Self {
        let root = frames
            .allocate(Owner::PageTable(asid), zero_dynamic)
            .unwrap();
        let pdpt = frames
            .allocate(Owner::PageTable(asid), zero_dynamic)
            .unwrap();
        let kernel_pdpt = read(kernel.root, 0) & ADDRESS;
        write(root, 0, pdpt | PRESENT | WRITE | USER);
        write(pdpt, 0, read(kernel_pdpt, 0));
        let mut result = Self {
            root,
            tables: [0; MAX_TABLES],
            count: 2,
            asid,
            #[cfg(feature = "framebuffer")]
            framebuffer: kernel.framebuffer,
            #[cfg(feature = "native-smp")]
            lapic: kernel.lapic,
        };
        result.tables[0] = root;
        result.tables[1] = pdpt;
        result
    }
    #[cfg(feature = "milestone-m3")]
    pub fn unmap(&mut self, frames: &mut Frames<FRAME_COUNT>, va: u64) {
        let (table, index) = self.location(va);
        let entry = read(table, index);
        assert_ne!(entry & PRESENT, 0);
        write(table, index, 0);
        invalidate(va);
        let pa = entry & ADDRESS;
        frames
            .unmap(pa, entry & WRITE != 0, entry & NX == 0)
            .unwrap();
        if frames.alias(pa).unwrap().executable == 0 {
            // SAFETY: All executable leaves were removed and TLB invalidated
            // before the writable scratch prohibition may be released.
            unsafe {
                ptr::addr_of_mut!(EXECUTABLE)
                    .cast::<bool>()
                    .add((pa / PAGE) as usize)
                    .write(false);
            }
        }
    }
    #[cfg(feature = "milestone-m3")]
    pub fn identity_readonly(&mut self, frames: &mut Frames<FRAME_COUNT>, pa: u64) {
        self.unmap(frames, pa);
        self.map(frames, pa, pa, false, false);
        invalidate(pa);
    }
    #[cfg(feature = "milestone-m3")]
    fn location(&self, va: u64) -> (u64, usize) {
        let mut table = self.root;
        for shift in [39, 30, 21] {
            let e = read(table, ((va >> shift) & 511) as usize);
            assert_ne!(e & PRESENT, 0);
            assert_eq!(e & 128, 0);
            table = e & ADDRESS;
        }
        (table, ((va >> 12) & 511) as usize)
    }
    #[cfg(feature = "milestone-m3")]
    pub fn begin_scratch(&mut self, frames: &mut Frames<FRAME_COUNT>) {
        let (table, index) = self.location(SCRATCH);
        self.unmap(frames, SCRATCH);
        // SAFETY: Initial scratch PT is identitymapped, permanent and exclusive.
        unsafe {
            ptr::addr_of_mut!(SCRATCH_ENTRY).write(table + index as u64 * 8);
        }
    }
    #[cfg(feature = "milestone-m3")]
    pub fn destroy(self, frames: &mut Frames<FRAME_COUNT>) {
        let active: u64;
        // SAFETY: CPL0 register read validates that no active root is released.
        unsafe {
            core::arch::asm!("mov {}, cr3",out(reg) active,options(nomem,nostack));
        }
        assert_ne!(self.root, active);
        for &table in &self.tables[..self.count] {
            frames
                .free(table, Owner::PageTable(self.asid), zero_dynamic)
                .unwrap();
        }
    }
    #[cfg(feature = "milestone-m3")]
    pub fn owned_table_count(&self) -> usize {
        self.count
    }
    pub fn map_tables(&mut self, frames: &mut Frames<FRAME_COUNT>) {
        // Newly needed PTs can themselves require identity mappings. The fixed
        // table bound makes this a bounded bootstrap fixed point.
        let mut i = 0;
        while i < self.count {
            let address = self.tables[i];
            self.map(frames, address, address, true, false);
            i += 1;
        }
    }
    pub fn leaf(&self, va: u64) -> u64 {
        let mut table = self.root;
        for shift in [39, 30, 21] {
            let entry = read(table, ((va >> shift) & 511) as usize);
            if entry & PRESENT == 0 {
                return 0;
            }
            assert_eq!(entry & 128, 0);
            table = entry & ADDRESS;
        }
        read(table, ((va >> 12) & 511) as usize)
    }
    #[cfg(feature = "milestone-m3")]
    pub fn permissions(&self, va: u64) -> Option<(u64, bool, bool, bool)> {
        let mut table = self.root;
        let mut writable = true;
        let mut executable = true;
        let mut user = true;
        for shift in [39, 30, 21, 12] {
            let entry = read(table, ((va >> shift) & 511) as usize);
            if entry & PRESENT == 0 {
                return None;
            }
            assert_eq!(entry & 128, 0);
            writable &= entry & WRITE != 0;
            executable &= entry & NX == 0;
            user &= entry & USER != 0;
            table = entry & ADDRESS;
        }
        Some((table + (va & (PAGE - 1)), writable, executable, user))
    }
    pub fn audit(&self, frames: &Frames<FRAME_COUNT>) {
        assert_eq!(self.leaf(0), 0);
        // SAFETY: Exclusive single-CPU bootstrap, IF=0. Static scratch is distinct
        // from frame metadata and page tables, initialized before every audit.
        let observed = unsafe { &mut *ptr::addr_of_mut!(OBSERVED) };
        observed.fill(Alias {
            writable: 0,
            executable: 0,
            total: 0,
        });
        self.walk(frames, observed, self.root, 4, false, true, false, 0);
        for (i, actual) in observed.iter().enumerate() {
            let expected = frames.alias(i as u64 * PAGE).unwrap();
            assert_eq!(
                *actual, expected,
                "physical alias index differs from actual tables"
            );
            assert!(actual.executable == 0 || actual.writable == 0);
        }
        for &table in &self.tables[..self.count] {
            assert_eq!(frames.owner(table), Ok(Owner::PageTable(0)));
            assert_eq!(
                self.leaf(table) & (PRESENT | WRITE | USER | NX),
                PRESENT | WRITE | NX
            );
        }
    }
    fn walk(
        &self,
        frames: &Frames<FRAME_COUNT>,
        observed: &mut [Alias; FRAME_COUNT],
        table: u64,
        level: u8,
        parent_user: bool,
        parent_write: bool,
        parent_nx: bool,
        virtual_base: u64,
    ) {
        assert!(self.tables[..self.count].contains(&table));
        assert_eq!(frames.owner(table), Ok(Owner::PageTable(0)));
        for i in 0..512 {
            let entry = read(table, i);
            if entry & PRESENT == 0 {
                assert_eq!(entry, 0);
                continue;
            }
            assert_eq!(entry & 128, 0, "huge pages forbidden");
            let user = (level == 4 || parent_user) && entry & USER != 0;
            let writable = parent_write && entry & WRITE != 0;
            let nx = parent_nx || entry & NX != 0;
            let pa = entry & ADDRESS;
            let va = virtual_base | ((i as u64) << (12 + (u64::from(level) - 1) * 9));
            if level > 1 {
                self.walk(frames, observed, pa, level - 1, user, writable, nx, va);
            } else {
                assert!(!user, "M2 has no user mappings");
                #[cfg(feature = "native-smp")]
                if va == LAPIC_WINDOW {
                    // MMIO is outside the RAM ownership table; check its exact class.
                    assert_eq!(Some(pa), self.lapic, "LAPIC physical binding");
                    assert!(writable && nx);
                    assert_eq!(entry & !(ADDRESS | 32 | 64), PRESENT | WRITE | NX | 8 | 16);
                    continue;
                }
                #[cfg(feature = "framebuffer")]
                if let Some((base, length)) = self.framebuffer {
                    let start = boot_protocol::graphics::FRAMEBUFFER_WINDOW;
                    if (start..start + length).contains(&va) {
                        assert_eq!(pa, base + va - start, "framebuffer physical binding");
                        assert!(writable && nx);
                        assert_eq!(
                            entry & (PRESENT | WRITE | USER | NX | 8 | 16),
                            PRESENT | WRITE | NX | 8 | 16
                        );
                        assert_eq!(
                            entry & !(ADDRESS | PRESENT | WRITE | NX | 8 | 16 | 32 | 64),
                            0
                        );
                        continue;
                    }
                }
                assert!(!matches!(frames.owner(pa).unwrap(), Owner::Free));
                let a = &mut observed[(pa / PAGE) as usize];
                a.total = a.total.checked_add(1).unwrap();
                a.writable += u16::from(writable);
                a.executable += u16::from(!nx);
            }
        }
    }
}
fn read(table: u64, index: usize) -> u64 {
    assert_eq!(table % PAGE, 0);
    assert!(index < 512);
    // SAFETY: Callers only follow validated owned PageTable(0) pages. All pages
    // are identity mapped by firmware during building and by our CR3 afterwards.
    // u64 accesses are naturally aligned, no concurrent hardware table writers
    // except CPU A/D bits; volatile observes those and stores preserve ownership.
    unsafe {
        if ptr::addr_of!(SCRATCH_ENTRY).read() != 0 {
            physical(table, |p| p.cast::<u64>().add(index).read_volatile())
        } else {
            (table as *const u64).add(index).read_volatile()
        }
    }
}
fn write(table: u64, index: usize, value: u64) {
    assert_eq!(table % PAGE, 0);
    assert!(index < 512);
    // SAFETY: Exclusive owned aligned table; bootstrap-only with firmware CR3
    // active. No references to entries exist, and this table is not yet active.
    unsafe {
        if ptr::addr_of!(SCRATCH_ENTRY).read() != 0 {
            physical(table, |p| p.cast::<u64>().add(index).write_volatile(value));
        } else {
            (table as *mut u64).add(index).write_volatile(value);
        }
    }
}
fn invalidate(va: u64) {
    // SAFETY: CPL0, PCID/PGE disabled; invalidates only THIS CPU's TLB. All
    // profiles but native-smp have one CPU; under native-smp a translation
    // other CPUs may hold additionally needs the smp.rs shootdown before any
    // alias metadata is released or the frame reused.
    unsafe {
        core::arch::asm!("invlpg [{}]",in(reg) va,options(nostack));
    }
}
fn zero_page(pa: u64) {
    // SAFETY: Read immutable-after-setup single-CPU scratch-entry address.
    if unsafe { ptr::addr_of!(SCRATCH_ENTRY).read() } == 0 {
        zero(pa);
    } else {
        zero_dynamic(pa);
    }
}
pub fn zero_dynamic(pa: u64) {
    physical(pa, |p| {
        // SAFETY: Exclusive zeroing window; callback touches exactly one page.
        unsafe {
            ptr::write_bytes(p, 0, PAGE as usize);
        }
    });
}
/// One temporary supervisor RW/NX page. IF=0 and no nested callbacks. The caller
/// owns the frame (or the allocator's unpublished zeroing transition). The PTE is
/// removed and locally invalidated before ownership can be published or freed.
pub fn physical<R>(pa: u64, operation: impl FnOnce(*mut u8) -> R) -> R {
    assert_eq!(pa % PAGE, 0);
    assert!(pa != 0 && pa < PAGE * FRAME_COUNT as u64);
    // SAFETY: Scratch PT permanently retained/identitymapped; all callers run on
    // one CPU, IF=0, no nested scratch operation, and hold exclusive allocator or
    // table access. Raw pointer exists only for this synchronous callback.
    unsafe {
        assert!(
            !ptr::addr_of!(EXECUTABLE)
                .cast::<bool>()
                .add((pa / PAGE) as usize)
                .read(),
            "scratch cannot alias executable frame writable"
        );
        let entry = ptr::addr_of!(SCRATCH_ENTRY).read() as *mut u64;
        assert!(!entry.is_null());
        assert_eq!(entry.read_volatile(), 0);
        entry.write_volatile(pa | PRESENT | WRITE | NX);
        invalidate(SCRATCH);
        let result = operation(SCRATCH as *mut u8);
        entry.write_volatile(0);
        invalidate(SCRATCH);
        result
    }
}
#[cfg(feature = "milestone-m4")]
pub fn physical_read<R>(pa: u64, operation: impl FnOnce(*const u8) -> R) -> R {
    assert_eq!(pa % PAGE, 0);
    assert!(pa != 0 && pa < PAGE * FRAME_COUNT as u64);
    // SAFETY: Same serialized scratch lifetime as physical(), but a supervisor
    // RO/NX alias cannot violate W^X even when reading executable input pages.
    unsafe {
        let entry = ptr::addr_of!(SCRATCH_ENTRY).read() as *mut u64;
        assert!(!entry.is_null());
        assert_eq!(entry.read_volatile(), 0);
        entry.write_volatile(pa | PRESENT | NX);
        invalidate(SCRATCH);
        let result = operation(SCRATCH as *const u8);
        entry.write_volatile(0);
        invalidate(SCRATCH);
        result
    }
}
