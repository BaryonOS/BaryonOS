#![no_std]
#![no_main]
#![deny(unsafe_op_in_unsafe_fn)]
//! Native B03 update service (ADR-049). It owns exactly one block capability
//! over the slot store and nothing else. It verifies the inbox bundle with
//! both pinned anchors and the image digest BEFORE any write, decides with
//! the shared `update-core` policy against the kernel-reported current
//! version, and installs into the inactive slot in the order image, FLUSH,
//! manifest, FLUSH, header, FLUSH. Everything it observes becomes evidence
//! words the kernel prints; the loader re-verifies at the next boot.

#[cfg(not(all(target_arch = "x86_64", target_os = "none")))]
compile_error!("update-service is only for the Nieuw OS x86_64-unknown-none target");

mod support;

use nieuw_os_sdk_runtime::{block::READ_MAX, native, BlockDevice, Error, Handle};
use update_core::{
    decide, header_after_install, image_sector, image_sectors, manifest_sector, Decision, Header,
    ImageDigest, Manifest, Reject, INBOX_IMAGE, INBOX_MANIFEST, MANIFEST_LEN, MANIFEST_SECTORS,
    SECTOR,
};

#[cfg(feature = "test-signing-anchors")]
use update_core::test_policy::verify_manifest;

#[cfg(all(feature = "native", not(feature = "test-signing-anchors")))]
fn verify_manifest(_: &[u8; MANIFEST_LEN]) -> Result<Manifest, Reject> {
    compile_error!(
        "the update service requires an explicit signing policy; TEST anchors are test-only"
    );
}

const POLL_LIMIT: usize = 200_000;

// Evidence words in the startup page (kernel update.rs reads these).
const STATE: u64 = 192;
const ERROR: u64 = 232;
const STEP: u64 = 240;
const RESULT: u64 = 256;
const HEADER_ACTIVE: u64 = 264;
const HEADER_GENERATION: u64 = 272;
const ACTIVE_VERSION: u64 = 280;
const ACTIVE_REJECT: u64 = 288;
const INBOX_VERSION: u64 = 296;
const INBOX_REJECT: u64 = 304;
const INSTALLED_VERSION: u64 = 312;
const TARGET_SLOT: u64 = 320;
const READS: u64 = 328;
const WRITES: u64 = 336;
const FLUSHES: u64 = 344;
const COPIED: u64 = 352;
const NEW_GENERATION: u64 = 360;
/// 1 when a partial install of this bundle was already present in the target
/// slot (image prefix without manifest): the install is being resumed. The
/// kernel's labelled cut fixture reads this word to cut only a first attempt.
const RESUMED: u64 = 368;
const RUNNING_SLOT: u64 = 384;
const OTHER_VERSION: u64 = 392;
const OTHER_REJECT: u64 = 400;

const RESULT_INSTALLED: u64 = 1;
const RESULT_UP_TO_DATE: u64 = 2;
const RESULT_REJECTED: u64 = 3;
const RESULT_NO_BUNDLE: u64 = 4;

fn tick() {
    support::put(8, support::word(8).wrapping_add(1));
}

fn code(error: Error) -> i64 {
    match error {
        Error::Kernel(code) => code,
        _ => -99,
    }
}

fn reject_code(reject: Reject) -> u64 {
    match reject {
        Reject::Format => 1,
        Reject::UnknownKey => 2,
        Reject::Signature => 3,
        Reject::Digest => 4,
        Reject::Downgrade => 5,
        Reject::Read => 6,
        Reject::KeyRevoked => 7,
        Reject::KeyExpired => 8,
        Reject::KeyNotYetValid => 9,
        Reject::KeyPolicy => 10,
    }
}

/// Startup/IO failure: report step and code, keep running so the kernel
/// observes the refusal instead of a process exit.
fn fail(step: u64, code: i64) -> ! {
    support::put(STEP, step);
    support::put(ERROR, code as u64);
    support::put(STATE, 9);
    loop {
        tick();
        let _ = native::yield_now();
    }
}

struct Disk {
    device: BlockDevice,
    reads: u64,
    writes: u64,
    flushes: u64,
}

impl Disk {
    fn read(&mut self, sector: u32, buffer: &mut [u8; SECTOR]) -> Result<(), i64> {
        for part in 0..SECTOR / READ_MAX {
            tick();
            let ticket = native::block_read(
                self.device,
                u64::from(sector),
                (part * READ_MAX) as u64,
                READ_MAX as u64,
            )
            .map_err(code)?;
            native::block_wait(
                self.device,
                ticket,
                &mut buffer[part * READ_MAX..(part + 1) * READ_MAX],
                POLL_LIMIT,
            )
            .map_err(code)?;
            self.reads += 1;
        }
        Ok(())
    }
    fn write(&mut self, sector: u32, buffer: &[u8; SECTOR]) -> Result<(), i64> {
        tick();
        let ticket = native::block_write(self.device, u64::from(sector), buffer).map_err(code)?;
        native::block_wait(self.device, ticket, &mut [], POLL_LIMIT).map_err(code)?;
        self.writes += 1;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), i64> {
        tick();
        let ticket = native::block_flush(self.device).map_err(code)?;
        native::block_wait(self.device, ticket, &mut [], POLL_LIMIT).map_err(code)?;
        self.flushes += 1;
        Ok(())
    }
    fn read_manifest(&mut self, first: u32, out: &mut [u8; MANIFEST_LEN]) -> Result<(), i64> {
        let mut sector = [0u8; SECTOR];
        for index in 0..MANIFEST_SECTORS {
            self.read(first + index, &mut sector)?;
            let start = index as usize * SECTOR;
            let end = (start + SECTOR).min(MANIFEST_LEN);
            out[start..end].copy_from_slice(&sector[..end - start]);
        }
        Ok(())
    }
    fn write_manifest(&mut self, first: u32, bytes: &[u8; MANIFEST_LEN]) -> Result<(), i64> {
        let mut sector = [0u8; SECTOR];
        for index in 0..MANIFEST_SECTORS {
            let start = index as usize * SECTOR;
            let end = (start + SECTOR).min(MANIFEST_LEN);
            sector.fill(0);
            sector[..end - start].copy_from_slice(&bytes[start..end]);
            self.write(first + index, &sector)?;
        }
        Ok(())
    }
    fn counters(&self) {
        support::put(READS, self.reads);
        support::put(WRITES, self.writes);
        support::put(FLUSHES, self.flushes);
    }
}

fn done(disk: &Disk, result: u64) -> ! {
    disk.counters();
    support::put(RESULT, result);
    support::put(STATE, 1);
    loop {
        tick();
        let _ = native::yield_now();
    }
}

/// Two-phase install: digest pass over the inbox image without writing,
/// then image copy, FLUSH, manifest, FLUSH, header, FLUSH.
fn install(
    disk: &mut Disk,
    header: &Header,
    manifest_bytes: &[u8; MANIFEST_LEN],
    manifest: &Manifest,
    target: u8,
) -> Result<Result<(), Reject>, i64> {
    let sectors = image_sectors(manifest.image_length);
    let mut sector = [0u8; SECTOR];
    // Resume detection: the target already carries this image's first
    // sector but no manifest yet, so an earlier attempt was interrupted
    // between the image writes and the manifest write.
    let mut first = [0u8; SECTOR];
    support::put(STEP, 9);
    disk.read(INBOX_IMAGE, &mut first)?;
    disk.read(image_sector(target), &mut sector)?;
    let same_prefix = first == sector && first.iter().any(|byte| *byte != 0);
    disk.read(manifest_sector(target), &mut sector)?;
    let no_manifest = sector.iter().all(|byte| *byte == 0);
    support::put(RESUMED, u64::from(same_prefix && no_manifest));
    let mut digest = ImageDigest::new(manifest);
    for index in 0..sectors {
        support::put(STEP, 10);
        disk.read(INBOX_IMAGE + index, &mut sector)?;
        digest.update(&sector);
    }
    if let Err(reject) = digest.finish(manifest) {
        return Ok(Err(reject));
    }
    let mut copied = 0u64;
    for index in 0..sectors {
        support::put(STEP, 11);
        disk.read(INBOX_IMAGE + index, &mut sector)?;
        disk.write(image_sector(target) + index, &sector)?;
        copied += 1;
        support::put(COPIED, copied);
    }
    support::put(STEP, 12);
    disk.flush()?;
    support::put(STEP, 13);
    disk.write_manifest(manifest_sector(target), manifest_bytes)?;
    support::put(STEP, 14);
    disk.flush()?;
    support::put(STEP, 15);
    let next = header_after_install(header, target);
    disk.write(0, &next.encode())?;
    support::put(STEP, 16);
    disk.flush()?;
    support::put(NEW_GENERATION, next.generation);
    Ok(Ok(()))
}

#[no_mangle]
#[link_section = ".text.entry"]
pub extern "C" fn _start() -> ! {
    if native::check_abi().is_err() {
        fail(1, -38);
    }
    #[cfg(feature = "test-policy-envelope-probe")]
    {
        // Independently authenticate immutable public TEST vectors in ring 3.
        // Evidence only: these words confer no policy or block authority.
        let report = update_core::policy_envelope::probe::run();
        support::put(408, report.cases);
        support::put(416, report.passed_mask);
        if !report.passed() {
            fail(17, -20);
        }
    }
    let Ok(handle) = Handle::from_raw(support::word(48)) else {
        fail(1, -1);
    };
    let current = support::word(120);
    let mut disk = Disk {
        device: BlockDevice::from_handle(handle),
        reads: 0,
        writes: 0,
        flushes: 0,
    };
    let mut sector = [0u8; SECTOR];
    support::put(STEP, 2);
    disk.read(0, &mut sector)
        .unwrap_or_else(|code| fail(2, code));
    // The header is a hint for the active slot; it carries no signature.
    let header = Header::decode(&sector).unwrap_or_else(|_| fail(3, -18));
    support::put(HEADER_ACTIVE, u64::from(header.active));
    support::put(HEADER_GENERATION, header.generation);
    // Both slot manifests, verified. The running slot is the one whose
    // verified version equals the kernel's own version; the header is only
    // the tie-breaker. Writing into the running slot is never allowed.
    let mut manifest_bytes = [0u8; MANIFEST_LEN];
    let mut versions: [Option<u64>; 2] = [None, None];
    for (slot, (version_word, reject_word)) in [
        (header.active, (ACTIVE_VERSION, ACTIVE_REJECT)),
        (
            update_core::other(header.active),
            (OTHER_VERSION, OTHER_REJECT),
        ),
    ] {
        support::put(STEP, 4);
        disk.read_manifest(manifest_sector(slot), &mut manifest_bytes)
            .unwrap_or_else(|code| fail(4, code));
        match verify_manifest(&manifest_bytes) {
            Ok(manifest) => {
                versions[usize::from(slot)] = Some(manifest.version);
                support::put(version_word, manifest.version);
            }
            Err(reject) => support::put(reject_word, reject_code(reject)),
        }
    }
    let running = if versions[usize::from(header.active)] == Some(current) {
        header.active
    } else if versions[usize::from(update_core::other(header.active))] == Some(current) {
        update_core::other(header.active)
    } else {
        // No verified slot carries the running version: refuse to guess.
        fail(8, -19)
    };
    support::put(RUNNING_SLOT, u64::from(running));
    let effective = Header {
        active: running,
        generation: header.generation,
    };
    support::put(STEP, 5);
    disk.read_manifest(INBOX_MANIFEST, &mut manifest_bytes)
        .unwrap_or_else(|code| fail(5, code));
    if manifest_bytes.iter().all(|byte| *byte == 0) {
        done(&disk, RESULT_NO_BUNDLE);
    }
    support::put(STEP, 6);
    let inbox = verify_manifest(&manifest_bytes);
    if let Ok(manifest) = inbox {
        support::put(INBOX_VERSION, manifest.version);
    }
    // The running kernel's version is both the current version and the
    // floor this service can know; the loader holds the firmware floor.
    match decide(&effective, current, current, inbox) {
        Decision::Rejected(reject) => {
            support::put(INBOX_REJECT, reject_code(reject));
            done(&disk, RESULT_REJECTED);
        }
        Decision::UpToDate => done(&disk, RESULT_UP_TO_DATE),
        Decision::Install { target, version } => {
            let manifest = inbox.unwrap_or_else(|_| fail(7, -1));
            support::put(TARGET_SLOT, u64::from(target));
            match install(&mut disk, &effective, &manifest_bytes, &manifest, target) {
                Err(code) => fail(support::word(STEP), code),
                Ok(Err(reject)) => {
                    support::put(INBOX_REJECT, reject_code(reject));
                    done(&disk, RESULT_REJECTED);
                }
                Ok(Ok(())) => {
                    support::put(INSTALLED_VERSION, version);
                    done(&disk, RESULT_INSTALLED);
                }
            }
        }
    }
}

#[panic_handler]
fn panic(_: &core::panic::PanicInfo<'_>) -> ! {
    support::put(ERROR, (-98i64) as u64);
    native::exit(3)
}
