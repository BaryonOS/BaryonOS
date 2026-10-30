#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(feature = "graphics-g0")]
mod graphics;
mod memory;
mod serial;
#[cfg(feature = "signed-boot")]
mod signed;

use boot_protocol::{
    BootInfo, ABI_MAJOR, ABI_MINOR, ADDRESS_LIMIT, KERNEL_LEN, KERNEL_START, MAGIC, MAP_LIMIT,
    PAGE_SIZE, STACK_LEN,
};
use core::{
    arch::{asm, global_asm},
    ffi::c_void,
    mem::{align_of, size_of},
    ptr,
};
use r_efi::{
    efi,
    protocols::{file, loaded_image, simple_file_system},
};

global_asm!(include_str!("handoff.S"));
extern "efiapi" {
    fn handoff(entry: u64, info: u64, stack_top: u64) -> !;
}

type Failure = &'static [u8];
type Result<T> = core::result::Result<T, Failure>;

fn check(status: efi::Status, code: Failure) -> Result<()> {
    if status == efi::Status::SUCCESS {
        Ok(())
    } else {
        Err(code)
    }
}
fn aligned<T>(p: *const T) -> bool {
    !p.is_null() && (p as usize).is_multiple_of(align_of::<T>())
}
fn page_round(n: u64) -> Result<u64> {
    n.checked_add(PAGE_SIZE - 1)
        .map(|v| v & !(PAGE_SIZE - 1))
        .ok_or(b"RANGE_OVERFLOW")
}

/// Present only before finalization. No Drop implementation or global service pointer.
struct Firmware {
    services: *mut efi::BootServices,
    #[cfg_attr(not(feature = "signed-boot"), allow(dead_code))]
    runtime: *mut efi::RuntimeServices,
}

impl Firmware {
    /// # Safety
    /// `table` must be the live UEFI entry argument, valid for SystemTable reads.
    /// The firmware and its service/protocol function pointers are trusted in M1.
    unsafe fn new(image: efi::Handle, table: *mut efi::SystemTable) -> Result<Self> {
        if image.is_null() || !aligned(table) {
            return Err(b"FIRMWARE_POINTER");
        }
        // SAFETY: UEFI supplies a live aligned SystemTable; checked null/alignment.
        let table = unsafe { &*table };
        if table.hdr.signature != efi::SYSTEM_TABLE_SIGNATURE
            || table.hdr.revision < efi::SYSTEM_TABLE_REVISION_2_00
            || (table.hdr.header_size as usize) < size_of::<efi::SystemTable>()
            || !aligned(table.boot_services)
            || !aligned(table.runtime_services)
        {
            return Err(b"SYSTEM_TABLE");
        }
        // SAFETY: Live firmware owns this runtime table; only its header is read here.
        let runtime = unsafe { &*table.runtime_services };
        if runtime.hdr.signature != efi::RUNTIME_SERVICES_SIGNATURE
            || (runtime.hdr.header_size as usize) < size_of::<efi::RuntimeServices>()
        {
            return Err(b"RUNTIME_SERVICES_TABLE");
        }
        // SAFETY: Live firmware owns this complete services table before EBS.
        let services = unsafe { &*table.boot_services };
        if services.hdr.signature != efi::BOOT_SERVICES_SIGNATURE
            || services.hdr.revision < efi::SYSTEM_TABLE_REVISION_2_00
            || (services.hdr.header_size as usize) < size_of::<efi::BootServices>()
        {
            return Err(b"BOOT_SERVICES_TABLE");
        }
        let fw = Self {
            services: table.boot_services,
            runtime: table.runtime_services,
        };
        // SAFETY: Live service, disable watchdog; zero data size permits null data.
        check(
            unsafe { (services.set_watchdog_timer)(0, 0, 0, ptr::null_mut()) },
            b"WATCHDOG",
        )?;
        Ok(fw)
    }

    fn services(&self) -> &efi::BootServices {
        // SAFETY: Firmware exists only in live preparatory phase; freeze consumes it.
        unsafe { &*self.services }
    }

    /// Variable services, used before ExitBootServices only (version floor).
    #[cfg(feature = "signed-boot")]
    fn runtime(&self) -> &efi::RuntimeServices {
        // SAFETY: Validated at construction; the live phase ends at freeze.
        unsafe { &*self.runtime }
    }

    fn pages(&self, pages: usize, kind: efi::MemoryType, address: Option<u64>) -> Result<u64> {
        let length = (pages as u64)
            .checked_mul(PAGE_SIZE)
            .ok_or(b"ALLOC_RANGE" as Failure)?;
        let mut base = address.unwrap_or(ADDRESS_LIMIT - 1);
        let allocation = if address.is_some() {
            efi::ALLOCATE_ADDRESS
        } else {
            efi::ALLOCATE_MAX_ADDRESS
        };
        // SAFETY: Live service, valid mutable output; no Rust references point into new allocation.
        let status =
            unsafe { (self.services().allocate_pages)(allocation, kind, pages, &mut base) };
        if status != efi::Status::SUCCESS {
            self.diagnose_allocation(status, address.unwrap_or(ADDRESS_LIMIT - 1), length, kind);
            return Err(if address.is_some() {
                b"KERNEL_RESERVATION"
            } else {
                b"ALLOC_PAGES"
            });
        }
        if base == 0
            || !base.is_multiple_of(PAGE_SIZE)
            || base
                .checked_add(length)
                .filter(|end| *end <= ADDRESS_LIMIT)
                .is_none()
            || address.is_some_and(|wanted| wanted != base)
        {
            return Err(b"ALLOC_RANGE");
        }
        Ok(base)
    }

    /// Terminal diagnostics only: fixed numeric guest metadata, no allocation or contents.
    fn diagnose_allocation(&self, status: efi::Status, address: u64, length: u64, kind: u32) {
        serial::init();
        serial::write(b"DIAG:ALLOC status=");
        serial::hex(status.as_usize() as u64);
        serial::write(b" address=");
        serial::hex(address);
        serial::write(b" length=");
        serial::hex(length);
        serial::write(b" type=");
        serial::hex(u64::from(kind));
        serial::write(b"\n");
        static mut DIAG_MAP: [u64; 8192] = [0; 8192];
        let buffer = ptr::addr_of_mut!(DIAG_MAP).cast::<efi::MemoryDescriptor>();
        let mut size = 65536;
        let (mut key, mut stride, mut version) = (0, 0, 0);
        // SAFETY: Single-threaded terminal path before EBS. Static u64 backing
        // provides alignment and 64 KiB exclusive capacity; no Rust reference aliases it.
        let result = unsafe {
            (self.services().get_memory_map)(&mut size, buffer, &mut key, &mut stride, &mut version)
        };
        if result != efi::Status::SUCCESS || size > 65536 || stride > u32::MAX as usize {
            serial::write(b"DIAG:MAP_UNAVAILABLE\n");
            return;
        }
        // SAFETY: Firmware returned initialized bytes within the fixed capacity.
        // No later firmware write or concurrent diagnostic can mutate this buffer.
        let bytes = unsafe { core::slice::from_raw_parts(buffer.cast::<u8>(), size) };
        let Ok(map) = boot_protocol::MemoryMap::new(bytes, stride as u32, version) else {
            serial::write(b"DIAG:MAP_INVALID\n");
            return;
        };
        let mut shown = 0;
        for index in 0..map.len() {
            let descriptor = map.descriptor(index).unwrap();
            if descriptor.physical_start >= 64 * 1024 * 1024 {
                continue;
            }
            if shown == 64 {
                serial::write(b"DIAG:MAP_TRUNCATED\n");
                break;
            }
            serial::write(b"DIAG:MAP type=");
            serial::hex(u64::from(descriptor.memory_type));
            serial::write(b" start=");
            serial::hex(descriptor.physical_start);
            serial::write(b" pages=");
            serial::hex(descriptor.number_of_pages);
            serial::write(b" attributes=");
            serial::hex(descriptor.attributes);
            serial::write(b"\n");
            shown += 1;
        }
    }

    fn free_pages(&self, address: u64, pages: usize) -> Result<()> {
        // SAFETY: Callers supply exact previously allocated pages, after all views expire.
        check(
            unsafe { (self.services().free_pages)(address, pages) },
            b"FREE_PAGES",
        )
    }

    fn protocol<T>(&self, handle: efi::Handle, mut guid: efi::Guid) -> Result<*mut T> {
        let mut result: *mut c_void = ptr::null_mut();
        // SAFETY: Live firmware handle; GUID and output pointer exist through call.
        // HandleProtocol does not establish a BY_DRIVER relationship to later close.
        check(
            unsafe { (self.services().handle_protocol)(handle, &mut guid, &mut result) },
            b"PROTOCOL",
        )?;
        if !aligned(result.cast::<T>()) {
            return Err(b"PROTOCOL_POINTER");
        }
        Ok(result.cast())
    }

    fn loaded_image(&self, image: efi::Handle) -> Result<(efi::Handle, u64, u64)> {
        let p = self.protocol::<loaded_image::Protocol>(image, loaded_image::PROTOCOL_GUID)?;
        // SAFETY: GUID binds the firmware result to LoadedImage; read only during live phase.
        let li = unsafe { &*p };
        if li.revision < loaded_image::REVISION || li.device_handle.is_null() || li.image_size == 0
        {
            return Err(b"LOADED_IMAGE");
        }
        let base = (li.image_base as u64) & !(PAGE_SIZE - 1);
        let end = page_round(
            (li.image_base as u64)
                .checked_add(li.image_size)
                .ok_or(b"IMAGE_RANGE" as Failure)?,
        )?;
        if base == 0 || end > ADDRESS_LIMIT || end <= base {
            return Err(b"IMAGE_RANGE");
        }
        Ok((li.device_handle, base, end - base))
    }

    #[cfg_attr(feature = "signed-boot", allow(dead_code))]
    fn open_kernel(
        &self,
        device: efi::Handle,
    ) -> Result<(*mut file::Protocol, *mut file::Protocol)> {
        let fs = self
            .protocol::<simple_file_system::Protocol>(device, simple_file_system::PROTOCOL_GUID)?;
        let mut root = ptr::null_mut();
        // SAFETY: Protocol came from same boot device and live firmware; output is valid.
        check(
            unsafe { ((*fs).open_volume)(fs, &mut root) },
            b"OPEN_VOLUME",
        )?;
        if !aligned(root) {
            return Err(b"FILE_POINTER");
        }
        let mut name = [92u16, 107, 101, 114, 110, 101, 108, 46, 101, 108, 102, 0];
        let mut kernel = ptr::null_mut();
        // SAFETY: Root handle is live; name is a terminated UTF-16 literal and mode is read-only.
        check(
            unsafe { ((*root).open)(root, &mut kernel, name.as_mut_ptr(), file::MODE_READ, 0) },
            b"KERNEL_OPEN",
        )?;
        if !aligned(kernel) {
            return Err(b"FILE_POINTER");
        }
        Ok((root, kernel))
    }

    #[cfg_attr(feature = "signed-boot", allow(dead_code))]
    fn read_kernel(&self, kernel: *mut file::Protocol, address: u64) -> Result<usize> {
        let capacity = elf_loader_core::MAX_FILE_SIZE;
        let mut used = 0;
        loop {
            let request = core::cmp::min(capacity - used, 65536);
            if request == 0 {
                let mut extra = 0u8;
                let mut amount = 1;
                // SAFETY: Live file; one writable stack byte proves EOF at exact capacity.
                check(
                    unsafe {
                        ((*kernel).read)(kernel, &mut amount, ptr::addr_of_mut!(extra).cast())
                    },
                    b"KERNEL_READ",
                )?;
                if amount != 0 {
                    return Err(b"KERNEL_TOO_LARGE");
                }
                return Ok(used);
            }
            let mut amount = request;
            // SAFETY: address is a dedicated capacity-sized allocation; offset and
            // request stay in range. Firmware may short-read; no view exists concurrently.
            check(
                unsafe {
                    ((*kernel).read)(kernel, &mut amount, (address as *mut u8).add(used).cast())
                },
                b"KERNEL_READ",
            )?;
            if amount > request {
                return Err(b"KERNEL_READ_SIZE");
            }
            if amount == 0 {
                return Ok(used);
            }
            used = used
                .checked_add(amount)
                .ok_or(b"KERNEL_READ_SIZE" as Failure)?;
        }
    }

    #[cfg_attr(feature = "signed-boot", allow(dead_code))]
    fn close(&self, file: *mut file::Protocol) -> Result<()> {
        // SAFETY: Caller passes its live open handle exactly once, before freeze.
        check(unsafe { ((*file).close)(file) }, b"FILE_CLOSE")
    }

    fn map_size(&self) -> Result<(usize, usize)> {
        let mut size = 0;
        let (mut key, mut stride, mut version) = (0, 0, 0);
        // SAFETY: Zero capacity query permits a null map buffer; other outputs live.
        let status = unsafe {
            (self.services().get_memory_map)(
                &mut size,
                ptr::null_mut(),
                &mut key,
                &mut stride,
                &mut version,
            )
        };
        if status != efi::Status::BUFFER_TOO_SMALL
            || !(40..=4096).contains(&stride)
            || !stride.is_multiple_of(8)
            || version != 1
        {
            return Err(b"MAP_QUERY");
        }
        Ok((size, stride))
    }

    fn freeze(self) -> Frozen {
        Frozen {
            get_map: self.services().get_memory_map,
            exit: self.services().exit_boot_services,
        }
    }
}

#[derive(Clone, Copy)]
struct MapState {
    length: usize,
    key: usize,
    stride: usize,
    version: u32,
}

/// After freeze, only these two firmware functions can be reached. No allocator,
/// file/protocol handle, console, runtime pointer or firmware destructor is retained.
struct Frozen {
    get_map: efi::BootGetMemoryMap,
    exit: efi::BootExitBootServices,
}

impl Frozen {
    fn read_map(
        &self,
        address: u64,
        capacity: usize,
    ) -> core::result::Result<MapState, (efi::Status, usize)> {
        let mut state = MapState {
            length: capacity,
            key: 0,
            stride: 0,
            version: 0,
        };
        // SAFETY: Dedicated page allocation survives until terminal exit; length
        // is reset to its full capacity on every call, including EBS retries.
        let status = unsafe {
            (self.get_map)(
                &mut state.length,
                address as *mut efi::MemoryDescriptor,
                &mut state.key,
                &mut state.stride,
                &mut state.version,
            )
        };
        if status != efi::Status::SUCCESS {
            return Err((status, state.length));
        }
        if state.length > capacity {
            return Err((efi::Status::BAD_BUFFER_SIZE, state.length));
        }
        Ok(state)
    }

    fn transfer(
        self,
        image: efi::Handle,
        info_address: u64,
        entry: u64,
        initial: Option<MapState>,
    ) -> ! {
        // SAFETY: info_address is an initialized dedicated BootInfo allocation.
        // Copying avoids holding a mutable reference while inspecting the map.
        let mut info = unsafe { (info_address as *const BootInfo).read() };
        let capacity = info.memory_map_capacity as usize;
        let mut retried = false;
        for attempt in 0..3 {
            let state = if attempt == 0 {
                initial.or_else(|| self.read_map(info.memory_map_addr, capacity).ok())
            } else {
                self.read_map(info.memory_map_addr, capacity).ok()
            };
            let state = state.unwrap_or_else(|| serial::fail(b"FINAL_MEMORY_MAP"));
            info.memory_map_len = state.length as u64;
            info.descriptor_size =
                u32::try_from(state.stride).unwrap_or_else(|_| serial::fail(b"MAP_STRIDE"));
            info.descriptor_version = state.version;
            let mut candidate = info;
            candidate.flags = if cfg!(feature = "graphics-g0") { 15 } else { 7 };
            #[cfg(feature = "graphics-g0")]
            let valid_header = candidate.validate_graphics_header(info_address);
            #[cfg(not(feature = "graphics-g0"))]
            let valid_header = candidate.validate_header(info_address);
            if valid_header.is_err() {
                serial::fail(b"BOOTINFO_HEADER");
            }
            // SAFETY: Returned length <= allocated capacity, checked before slice;
            // this immutable view ends before the next firmware map write.
            let map = unsafe {
                core::slice::from_raw_parts(info.memory_map_addr as *const u8, state.length)
            };
            #[cfg(feature = "graphics-g0")]
            let valid_map = boot_protocol::graphics::validate_graphics_memory_map(&candidate, map);
            #[cfg(not(feature = "graphics-g0"))]
            let valid_map = boot_protocol::validate_memory_map(&candidate, map);
            if valid_map.is_err() {
                serial::fail(b"MEMORY_MAP");
            }
            #[cfg(feature = "graphics-g0")]
            {
                // SAFETY: Header binds the initialized 64-byte extension to our
                // retained BootInfo page; it cannot overlap the separate map.
                let framebuffer = unsafe {
                    (candidate.reserved0 as *const boot_protocol::graphics::FramebufferInfo).read()
                };
                let view = boot_protocol::MemoryMap::new(
                    map,
                    candidate.descriptor_size,
                    candidate.descriptor_version,
                )
                .unwrap_or_else(|_| serial::fail(b"GOP_MAP"));
                if framebuffer.validate_memory_map(&view).is_err() {
                    serial::fail(b"GOP_RAM_OVERLAP");
                }
            }
            // SAFETY: Own aligned page and valid plain fixed-width value. The map is
            // a disjoint allocation; CPU-only writes do not invalidate its map key.
            unsafe {
                (info_address as *mut BootInfo).write(info);
            }
            // SAFETY: Original entry image handle and immediately captured key.
            // After first attempt the only allowed firmware calls are this and GetMemoryMap.
            let status = unsafe { (self.exit)(image, state.key) };
            if status == efi::Status::SUCCESS {
                static EMPTY_IDT: [u8; 10] = [0; 10];
                // SAFETY: EBS succeeded at CPL0; disable firmware interrupt paths
                // immediately and install a fail-closed empty IDT before further Rust calls.
                unsafe {
                    asm!("cli", "cld", "lidt [{}]", in(reg) EMPTY_IDT.as_ptr(), options(nostack));
                }
                info.flags = if cfg!(feature = "graphics-g0") { 15 } else { 7 };
                // SAFETY: Reserved page remains mapped and owned after EBS.
                unsafe {
                    (info_address as *mut BootInfo).write(info);
                }
                serial::init();
                #[cfg(feature = "test-ebs-retry")]
                if !retried {
                    serial::fail(b"EBS_RETRY_NOT_OBSERVED");
                }
                serial::write(b"BOOT:EBS_OK\n");
                if retried {
                    serial::write(b"BOOT:EBS_RETRY\n");
                }
                // SAFETY: Validated ELF entry points to loaded RX-intent bytes;
                // all retained ranges are disjoint and covered by LoaderCode/Data.
                // Shim translates EFIAPI registers into SysV and installs own stack.
                unsafe {
                    handoff(entry, info_address, info.stack_start + info.stack_len);
                }
            }
            if status != efi::Status::INVALID_PARAMETER {
                serial::fail(b"EXIT_BOOT_SERVICES");
            }
            retried = true;
        }
        serial::fail(b"EBS_RETRY_LIMIT")
    }
}

fn prepare(fw: Firmware, image: efi::Handle) -> Result<()> {
    let (device, loader_start, loader_len) = fw.loaded_image(image)?;
    #[cfg(feature = "graphics-g0")]
    let framebuffer = graphics::prepare(&fw)?;
    fw.pages(
        (KERNEL_LEN / PAGE_SIZE) as usize,
        efi::LOADER_CODE,
        Some(KERNEL_START),
    )?;
    let file_pages = elf_loader_core::MAX_FILE_SIZE / PAGE_SIZE as usize;
    let file_address = fw.pages(file_pages, efi::LOADER_DATA, None)?;
    // B03: with `signed-boot` only a slot store with a verified, loadable,
    // floor-respecting image supplies the kernel; the ESP file is never read.
    #[cfg(feature = "signed-boot")]
    let file_length = {
        let _ = device;
        signed::load(&fw, file_address)?
    };
    #[cfg(not(feature = "signed-boot"))]
    let file_length = {
        let (root, kernel) = fw.open_kernel(device)?;
        let length = fw.read_kernel(kernel, file_address)?;
        fw.close(kernel)?;
        fw.close(root)?;
        length
    };
    let entry;
    {
        // SAFETY: Dedicated allocation holds exactly file_length initialized bytes;
        // firmware read bounded it to capacity, and both file handles are closed.
        let bytes = unsafe { core::slice::from_raw_parts(file_address as *const u8, file_length) };
        let plan = elf_loader_core::parse(bytes).map_err(|_| b"ELF_INVALID" as Failure)?;
        entry = plan.entry;
        // SAFETY: Exact 16 MiB fixed span was exclusively reserved before loading.
        // No typed values or aliases exist within the destination yet.
        unsafe {
            ptr::write_bytes(KERNEL_START as *mut u8, 0, KERNEL_LEN as usize);
        }
        for segment in plan.segments() {
            // SAFETY: Full parser validated all source bounds, distinct destination
            // page intervals and inclusion in reserved span before the first write.
            // UEFI gives disjoint allocations for the source buffer and destination.
            unsafe {
                ptr::copy_nonoverlapping(
                    bytes.as_ptr().add(segment.offset),
                    segment.physical_address as *mut u8,
                    segment.file_size,
                );
            }
        }
    }
    fw.free_pages(file_address, file_pages)?;
    let stack_start = fw.pages((STACK_LEN / PAGE_SIZE) as usize, efi::LOADER_DATA, None)?;
    let info_address = fw.pages(1, efi::LOADER_DATA, None)?;
    // SAFETY: Own dedicated page with no references yet; initialize padding and remainder too.
    unsafe {
        ptr::write_bytes(info_address as *mut u8, 0, PAGE_SIZE as usize);
    }
    #[cfg(feature = "graphics-g0")]
    // SAFETY: Our disjoint, zero-initialized page has 3968 bytes after its
    // 128-byte header. The copied scalar extension uses only the first 64.
    unsafe {
        ((info_address + 128) as *mut boot_protocol::graphics::FramebufferInfo).write(framebuffer);
    }

    let (needed, stride) = fw.map_size()?;
    let mut capacity = map_capacity(needed, stride)?;
    let mut map_address = fw.pages(capacity / PAGE_SIZE as usize, efi::LOADER_DATA, None)?;
    // Preparatory map reads may grow the buffer; no EBS has yet been attempted.
    let mut prepared = false;
    for _ in 0..3 {
        let reader = Frozen {
            get_map: fw.services().get_memory_map,
            exit: fw.services().exit_boot_services,
        };
        match reader.read_map(map_address, capacity) {
            Ok(_) => {
                prepared = true;
                break;
            }
            Err((status, length)) if status == efi::Status::BUFFER_TOO_SMALL => {
                let new_capacity = map_capacity(length, stride)?;
                fw.free_pages(map_address, capacity / PAGE_SIZE as usize)?;
                capacity = new_capacity;
                map_address = fw.pages(capacity / PAGE_SIZE as usize, efi::LOADER_DATA, None)?;
            }
            _ => return Err(b"PREPARE_MAP"),
        }
    }
    if !prepared {
        return Err(b"PREPARE_MAP_LIMIT");
    }
    let info = BootInfo {
        magic: MAGIC,
        abi_major: ABI_MAJOR,
        abi_minor: if cfg!(feature = "graphics-g0") {
            2
        } else {
            ABI_MINOR
        },
        header_size: 128,
        flags: if cfg!(feature = "graphics-g0") { 14 } else { 6 },
        memory_map_addr: map_address,
        memory_map_len: 0,
        descriptor_size: 0,
        descriptor_version: 0,
        kernel_reserved_start: KERNEL_START,
        kernel_reserved_len: KERNEL_LEN,
        stack_start,
        stack_len: STACK_LEN,
        bootinfo_start: info_address,
        bootinfo_len: PAGE_SIZE,
        loader_start,
        loader_len,
        memory_map_capacity: capacity as u64,
        reserved0: if cfg!(feature = "graphics-g0") {
            info_address + 128
        } else {
            0
        },
    };
    // SAFETY: Own aligned page, initialized and disjoint from every other live object.
    unsafe {
        (info_address as *mut BootInfo).write(info);
    }

    #[cfg(feature = "test-ebs-retry")]
    let initial = {
        let reader = Frozen {
            get_map: fw.services().get_memory_map,
            exit: fw.services().exit_boot_services,
        };
        let old = reader
            .read_map(map_address, capacity)
            .map_err(|_| b"INJECT_MAP" as Failure)?;
        // Deliberate stale-key injection exists only in the explicitly enabled test build.
        // The extra page stays reserved; no allocation occurs after freeze/first EBS.
        fw.pages(1, efi::LOADER_DATA, None)?;
        Some(old)
    };
    #[cfg(not(feature = "test-ebs-retry"))]
    let initial = None;
    fw.freeze().transfer(image, info_address, entry, initial)
}

fn map_capacity(needed: usize, stride: usize) -> Result<usize> {
    let slack = stride.checked_mul(8).ok_or(b"MAP_CAPACITY" as Failure)?;
    let required = needed
        .checked_add(slack)
        .ok_or(b"MAP_CAPACITY" as Failure)?;
    let capacity = page_round(core::cmp::max(256 * 1024, required) as u64)?;
    if capacity > MAP_LIMIT {
        return Err(b"MAP_CAPACITY");
    }
    Ok(capacity as usize)
}

#[no_mangle]
pub extern "efiapi" fn efi_main(image: efi::Handle, table: *mut efi::SystemTable) -> efi::Status {
    // SAFETY: UEFI invokes this exact entry ABI with live firmware-owned arguments.
    let result = unsafe { Firmware::new(image, table) }.and_then(|fw| prepare(fw, image));
    match result {
        Err(code) => serial::fail(code),
        Ok(()) => serial::fail(b"LOADER_RETURN"),
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    serial::fail(b"LOADER_PANIC")
}
