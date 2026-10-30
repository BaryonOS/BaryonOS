//! B03 signed boot (ADR-044). Before ExitBootServices the loader looks for a
//! `NOSSLOT1` store on a Block I/O device, verifies each slot manifest with
//! BOTH pinned anchors (Ed25519 and ML-DSA-65), applies the firmware-held
//! version floor, reads and hashes the chosen image into the caller's file
//! buffer, raises the floor and only then returns the image length to the
//! existing ELF path. The store header is a hint for slot order, never an
//! authority; a failing slot falls back to the other; two failing slots stop
//! the boot with `BOOT:FAIL:NO_VALID_SLOT`; no store at all stops it with
//! `BOOT:FAIL:NO_STORE`. A `signed-boot` loader never falls back to the
//! unsigned ESP kernel of the other profiles.
use super::{aligned, check, serial, Firmware, Result, PAGE_SIZE};
use core::{arch::asm, ffi::c_void, ptr};
use r_efi::{efi, protocols::block_io};
use update_core::{
    choose, image_sector, image_sectors, manifest_sector, verify_image, verify_manifest, Choice,
    Header, Manifest, Reject, SlotResult, TrustAnchors, HEADER_SECTOR, MANIFEST_LEN,
    MANIFEST_SECTORS, SECTOR, STORE_MAGIC, STORE_SECTORS,
};

/// The test profile uses only the public test keys committed to the repository.
#[cfg(feature = "test-anchors")]
const ANCHORS: TrustAnchors = TrustAnchors {
    ed25519: *include_bytes!("../../../tools/update-keys/TEST-ed25519.pub"),
    mldsa65: *include_bytes!("../../../tools/update-keys/TEST-mldsa65.pub"),
};
/// Release anchors are supplied as absolute paths at compile time by the
/// offline release process. Neither key may be a repository test anchor.
#[cfg(not(feature = "test-anchors"))]
const ANCHORS: TrustAnchors = TrustAnchors {
    ed25519: *include_bytes!(env!("NIEUWOS_RELEASE_ED25519_ANCHOR")),
    mldsa65: *include_bytes!(env!("NIEUWOS_RELEASE_MLDSA65_ANCHOR")),
};
#[cfg(not(feature = "test-anchors"))]
const fn same_bytes(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}
#[cfg(not(feature = "test-anchors"))]
const _: () = {
    assert!(
        !same_bytes(
            &ANCHORS.ed25519,
            include_bytes!("../../../tools/update-keys/TEST-ed25519.pub")
        ),
        "TEST Ed25519 anchor requires test-anchors"
    );
    assert!(
        !same_bytes(
            &ANCHORS.mldsa65,
            include_bytes!("../../../tools/update-keys/TEST-mldsa65.pub")
        ),
        "TEST ML-DSA anchor requires test-anchors"
    );
};
const MAX_HANDLES: usize = 32;
/// Vendor GUID of the non-volatile variable `NieuwOsMinVersion`.
const FLOOR_GUID: efi::Guid = efi::Guid::from_fields(
    0x5040_2e54,
    0xa387,
    0x47bc,
    0x8f,
    0x75,
    &[0xdc, 0xd7, 0x9c, 0xd0, 0xa6, 0x03],
);
/// "NieuwOsMinVersion" as a terminated UTF-16 string.
const FLOOR_NAME: [u16; 18] = [
    78, 105, 101, 117, 119, 79, 115, 77, 105, 110, 86, 101, 114, 115, 105, 111, 110, 0,
];

struct Store {
    protocol: *mut block_io::Protocol,
    media_id: u32,
}

impl Store {
    /// Read `count` whole sectors into our own allocation of at least that size.
    fn read(&self, lba: u32, count: u32, buffer: u64) -> core::result::Result<(), Reject> {
        // SAFETY: Live Block I/O protocol from the firmware handle; the caller's
        // buffer is a dedicated page allocation of at least count * 512 bytes.
        let status = unsafe {
            ((*self.protocol).read_blocks)(
                self.protocol,
                self.media_id,
                u64::from(lba),
                count as usize * SECTOR,
                buffer as *mut c_void,
            )
        };
        if status == efi::Status::SUCCESS {
            Ok(())
        } else {
            Err(Reject::Read)
        }
    }
}

fn locate(fw: &Firmware, scratch: u64) -> Result<Option<Store>> {
    let mut guid = block_io::PROTOCOL_GUID;
    let mut count = 0usize;
    let mut handles: *mut efi::Handle = ptr::null_mut();
    // SAFETY: Live boot services; outputs are valid locals, search key is null for BY_PROTOCOL.
    let status = unsafe {
        (fw.services().locate_handle_buffer)(
            efi::BY_PROTOCOL,
            &mut guid,
            ptr::null_mut(),
            &mut count,
            &mut handles,
        )
    };
    if status == efi::Status::NOT_FOUND {
        return Ok(None);
    }
    check(status, b"LOCATE_BLOCK_IO")?;
    if handles.is_null() {
        return Err(b"LOCATE_BLOCK_IO");
    }
    let mut found = None;
    for index in 0..count.min(MAX_HANDLES) {
        // SAFETY: The firmware filled `count` handles into the pool buffer.
        let handle = unsafe { *handles.add(index) };
        let Ok(protocol) = fw.protocol::<block_io::Protocol>(handle, block_io::PROTOCOL_GUID)
        else {
            continue;
        };
        // SAFETY: GUID-bound protocol instance; media pointer validated before the read.
        let media = unsafe { (*protocol).media };
        if !aligned(media) {
            continue;
        }
        // SAFETY: Non-null aligned media description owned by the live driver.
        let media = unsafe { *media };
        if !media.media_present
            || media.block_size != SECTOR as u32
            || media.last_block < u64::from(STORE_SECTORS) - 1
        {
            continue;
        }
        let store = Store {
            protocol,
            media_id: media.media_id,
        };
        if store.read(HEADER_SECTOR, 1, scratch).is_err() {
            continue;
        }
        // SAFETY: The firmware wrote one sector into our scratch page.
        let magic = unsafe { core::slice::from_raw_parts(scratch as *const u8, 8) };
        if magic == STORE_MAGIC {
            found = Some(store);
            break;
        }
    }
    // SAFETY: Pool buffer came from LocateHandleBuffer; freed exactly once.
    let _ = unsafe { (fw.services().free_pool)(handles.cast()) };
    Ok(found)
}

fn floor(fw: &Firmware) -> Result<u64> {
    let mut name = FLOOR_NAME;
    let mut guid = FLOOR_GUID;
    let mut attributes = 0u32;
    let mut size = 8usize;
    let mut value = [0u8; 8];
    // SAFETY: Live runtime services before EBS; all outputs are valid locals.
    let status = unsafe {
        (fw.runtime().get_variable)(
            name.as_mut_ptr(),
            &mut guid,
            &mut attributes,
            &mut size,
            value.as_mut_ptr().cast(),
        )
    };
    if status == efi::Status::NOT_FOUND {
        return Ok(0);
    }
    check(status, b"FLOOR_READ")?;
    if size != 8 {
        return Err(b"FLOOR_SIZE");
    }
    Ok(u64::from_le_bytes(value))
}

fn raise_floor(fw: &Firmware, version: u64) -> Result<()> {
    let mut name = FLOOR_NAME;
    let mut guid = FLOOR_GUID;
    let mut value = version.to_le_bytes();
    // SAFETY: Live runtime services before EBS; NV + BS attributes, 8 data bytes.
    let status = unsafe {
        (fw.runtime().set_variable)(
            name.as_mut_ptr(),
            &mut guid,
            efi::VARIABLE_NON_VOLATILE | efi::VARIABLE_BOOTSERVICE_ACCESS,
            8,
            value.as_mut_ptr().cast(),
        )
    };
    check(status, b"FLOOR_WRITE")
}

fn manifest(store: &Store, slot: u8, scratch: u64) -> SlotResult {
    store.read(manifest_sector(slot), MANIFEST_SECTORS, scratch)?;
    // SAFETY: Seven sectors (3584 bytes) were written into our 4 KiB scratch page.
    let bytes = unsafe { &*(scratch as *const [u8; MANIFEST_LEN]) };
    verify_manifest(bytes, &ANCHORS)
}

fn image(
    store: &Store,
    slot: u8,
    manifest: &Manifest,
    file_address: u64,
) -> core::result::Result<usize, Reject> {
    let sectors = image_sectors(manifest.image_length);
    store.read(image_sector(slot), sectors, file_address)?;
    let length = manifest.image_length as usize;
    // SAFETY: The file buffer is a dedicated MAX_FILE_SIZE allocation (>= 2 MiB);
    // the firmware wrote sectors * 512 >= length bytes; no other view exists.
    let bytes = unsafe { core::slice::from_raw_parts(file_address as *const u8, length) };
    verify_image(manifest, bytes)?;
    // A correctly signed image the loader cannot load is still not a valid
    // slot: reject it here so the other slot is tried and the floor is only
    // raised for an image that will actually run.
    elf_loader_core::parse(bytes).map_err(|_| Reject::Format)?;
    Ok(length)
}

fn slot_name(slot: u8) -> &'static [u8] {
    if slot == 0 {
        b"A"
    } else {
        b"B"
    }
}

fn reject_name(reject: Reject) -> &'static [u8] {
    match reject {
        Reject::Format => b"FORMAT",
        Reject::UnknownKey => b"UNKNOWN_KEY",
        Reject::Signature => b"SIGNATURE",
        Reject::Digest => b"DIGEST",
        Reject::Downgrade => b"DOWNGRADE",
        Reject::Read => b"READ",
    }
}

fn report(slot: u8, reject: Reject, printed: &mut [bool; 2]) {
    if printed[usize::from(slot)] {
        return;
    }
    printed[usize::from(slot)] = true;
    serial::write(b"BOOT:SLOT:");
    serial::write(slot_name(slot));
    serial::write(b":REJECTED:");
    serial::write(reject_name(reject));
    serial::write(b"\n");
}

fn fail(code: &[u8]) -> ! {
    serial::write(b"BOOT:FAIL:");
    serial::write(code);
    serial::write(b"\n");
    serial::exit(0x11)
}

/// ML-DSA-65 verification keeps about 200 KiB of live stack frames; the
/// firmware stack is not guaranteed to be that deep (OVMF: 128 KiB), so the
/// selection runs on a dedicated 1 MiB allocation.
const STACK_PAGES: usize = 256;

struct Context {
    fw: *const Firmware,
    scratch: u64,
    file_address: u64,
    result: Result<usize>,
}

extern "efiapi" fn trampoline(context: *mut Context) {
    // SAFETY: `load` passes its live, exclusively borrowed context for the
    // duration of this single call and reads the result afterwards.
    let context = unsafe { &mut *context };
    // SAFETY: The firmware handle outlives `load`; no other reference is live.
    let fw = unsafe { &*context.fw };
    context.result = select(fw, context.scratch, context.file_address);
}

/// `Ok(len)`: a verified, loadable image of `len` bytes is in the file
/// buffer and the floor covers its version. Every other outcome stops the
/// boot: structural firmware errors propagate, missing or invalid stores
/// end in `BOOT:FAIL:*`.
pub fn load(fw: &Firmware, file_address: u64) -> Result<usize> {
    let scratch = fw.pages(1, efi::LOADER_DATA, None)?;
    let stack = fw.pages(STACK_PAGES, efi::LOADER_DATA, None)?;
    let mut context = Context {
        fw,
        scratch,
        file_address,
        result: Err(b"SIGNED_SELECT"),
    };
    let top = stack + STACK_PAGES as u64 * PAGE_SIZE;
    // SAFETY: Switch to the fresh page-aligned allocation for exactly one
    // efiapi (Win64) call: 32 bytes of shadow space are reserved, RSP stays
    // 16-byte aligned at the call, r12 (callee-saved, declared clobbered)
    // carries the old RSP back, and every caller-saved register is clobbered.
    unsafe {
        asm!(
            "mov r12, rsp",
            "mov rsp, {top}",
            "sub rsp, 32",
            "call {f}",
            "mov rsp, r12",
            top = in(reg) top,
            f = sym trampoline,
            in("rcx") ptr::addr_of_mut!(context),
            out("r12") _,
            clobber_abi("efiapi"),
        );
    }
    fw.free_pages(stack, STACK_PAGES)?;
    fw.free_pages(scratch, 1)?;
    context.result
}

fn select(fw: &Firmware, scratch: u64, file_address: u64) -> Result<usize> {
    serial::init();
    #[cfg(feature = "test-anchors")]
    serial::write(b"BOOT:ANCHORS:TEST\n");
    let Some(store) = locate(fw, scratch)? else {
        // An unsigned ESP kernel is never an alternative for this loader.
        fail(b"NO_STORE");
    };
    // SAFETY: `locate` left the matching header sector in the scratch page.
    let header_bytes = unsafe { &*(scratch as *const [u8; SECTOR]) };
    let header = match Header::decode(header_bytes) {
        Ok(header) => header,
        Err(_) => {
            serial::write(b"BOOT:HEADER:REJECTED:FORMAT\n");
            Header {
                active: 0,
                generation: 0,
            }
        }
    };
    let floor = floor(fw)?;
    serial::write(b"BOOT:FLOOR:");
    serial::decimal(floor);
    serial::write(b"\n");
    let mut slots: [SlotResult; 2] = [manifest(&store, 0, scratch), manifest(&store, 1, scratch)];
    let mut printed = [false; 2];
    loop {
        let (choice, reasons) = choose(&header, slots, floor);
        let Some(Choice { slot, manifest }) = choice else {
            for (index, reason) in reasons.into_iter().enumerate() {
                if let Some(reject) = reason {
                    report(index as u8, reject, &mut printed);
                }
            }
            fail(b"NO_VALID_SLOT");
        };
        match image(&store, slot, &manifest, file_address) {
            Ok(length) => {
                if let Some(reject) = reasons[usize::from(update_core::other(slot))] {
                    report(update_core::other(slot), reject, &mut printed);
                }
                serial::write(b"BOOT:SLOT:");
                serial::write(slot_name(slot));
                serial::write(b":VERSION:");
                serial::decimal(manifest.version);
                serial::write(b"\n");
                if manifest.version > floor {
                    raise_floor(fw, manifest.version)?;
                }
                serial::write(b"BOOT:SIGNED:OK\n");
                return Ok(length);
            }
            Err(reject) => {
                report(slot, reject, &mut printed);
                slots[usize::from(slot)] = Err(reject);
            }
        }
    }
}
