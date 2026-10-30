//! Image installation with a readback check before publication.
use update_core::{
    header_after_install, image_sector, image_sectors, manifest_sector, Header, ImageDigest,
    Manifest, Reject, INBOX_IMAGE, MANIFEST_LEN, SECTOR,
};

pub(crate) const STEP: u64 = 240;
pub(crate) const COPIED: u64 = 352;
pub(crate) const NEW_GENERATION: u64 = 360;
pub(crate) const RESUMED: u64 = 368;

pub(crate) trait Storage {
    fn read(&mut self, sector: u32, buffer: &mut [u8; SECTOR]) -> Result<(), i64>;
    fn write(&mut self, sector: u32, buffer: &[u8; SECTOR]) -> Result<(), i64>;
    fn flush(&mut self) -> Result<(), i64>;
    fn write_manifest(&mut self, first: u32, bytes: &[u8; MANIFEST_LEN]) -> Result<(), i64>;
    fn mark(&mut self, offset: u64, value: u64);
}

/// Two-phase install: digest pass over the inbox image without writing,
/// then image copy, FLUSH, manifest, FLUSH, header, FLUSH.
pub(crate) fn install<D: Storage>(
    disk: &mut D,
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
    disk.mark(STEP, 9);
    disk.read(INBOX_IMAGE, &mut first)?;
    disk.read(image_sector(target), &mut sector)?;
    let same_prefix = first == sector && first.iter().any(|byte| *byte != 0);
    disk.read(manifest_sector(target), &mut sector)?;
    let no_manifest = sector.iter().all(|byte| *byte == 0);
    disk.mark(RESUMED, u64::from(same_prefix && no_manifest));
    let mut digest = ImageDigest::new(manifest);
    for index in 0..sectors {
        disk.mark(STEP, 10);
        disk.read(INBOX_IMAGE + index, &mut sector)?;
        digest.update(&sector);
    }
    if let Err(reject) = digest.finish(manifest) {
        return Ok(Err(reject));
    }
    let mut copied = 0u64;
    for index in 0..sectors {
        disk.mark(STEP, 11);
        disk.read(INBOX_IMAGE + index, &mut sector)?;
        disk.write(image_sector(target) + index, &sector)?;
        copied += 1;
        disk.mark(COPIED, copied);
    }
    disk.mark(STEP, 12);
    disk.flush()?;
    // Verify what the device returns after the image flush before publishing
    // a manifest or changing the slot header.
    let mut copied_digest = ImageDigest::new(manifest);
    for index in 0..sectors {
        disk.read(image_sector(target) + index, &mut sector)?;
        copied_digest.update(&sector);
    }
    if let Err(reject) = copied_digest.finish(manifest) {
        return Ok(Err(reject));
    }
    disk.mark(STEP, 13);
    disk.write_manifest(manifest_sector(target), manifest_bytes)?;
    disk.mark(STEP, 14);
    disk.flush()?;
    disk.mark(STEP, 15);
    let next = header_after_install(header, target);
    disk.write(0, &next.encode())?;
    disk.mark(STEP, 16);
    disk.flush()?;
    disk.mark(NEW_GENERATION, next.generation);
    Ok(Ok(()))
}
