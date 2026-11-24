//! The production archive verifier (spec 2.3, "Archive verifier"; route step
//! R5 precondition): `VolumeVerifier` runs `volume_core::archive::verify`.
//! These tests use real v1 archives written by `volume_core::archive::export`
//! from a host volume service, and complete the stand-in rows N7 and N10
//! (section 6) for the verifier that a native build links. Host only; no
//! native or QEMU claim.
mod common;

use common::*;
use destination_core::{
    archive_verifier::VolumeVerifier,
    service::{Source, Verifier},
    wire, Status,
};
use volume_core::{
    archive::{self, Sink},
    client::{Client, Transport},
    service::Service,
    table::Table,
    wire as vwire, Disk, SECTOR as VSECTOR, VOLUME_SECTORS,
};

const EPOCH: u64 = 4;

/// In-memory volume disk without a cache; only used to build archives.
struct Mem(Vec<[u8; VSECTOR]>);
impl Disk for Mem {
    fn read(&mut self, sector: u64, buffer: &mut [u8; VSECTOR]) -> Result<(), volume_core::Status> {
        *buffer = self.0[sector as usize];
        Ok(())
    }
    fn write(&mut self, sector: u64, buffer: &[u8; VSECTOR]) -> Result<(), volume_core::Status> {
        self.0[sector as usize] = *buffer;
        Ok(())
    }
    fn flush(&mut self) -> Result<(), volume_core::Status> {
        Ok(())
    }
}

struct Local<'a> {
    service: &'a mut Service,
    disk: &'a mut Mem,
}
impl Transport for Local<'_> {
    fn call(
        &mut self,
        request: &[u8; vwire::SIZE],
    ) -> Result<[u8; vwire::SIZE], volume_core::Status> {
        Ok(self.service.handle(0, request, self.disk))
    }
}

struct Bytes(Vec<u8>);
impl Sink for Bytes {
    fn put(&mut self, bytes: &[u8]) -> Result<(), volume_core::Status> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
}

fn fill(seed: u8) -> impl FnMut(usize, &mut [u8]) {
    move |offset, buffer| {
        for (index, byte) in buffer.iter_mut().enumerate() {
            let at = offset + index;
            *byte = seed.wrapping_add((at * 13 + at / 251) as u8);
        }
    }
}

/// A real v1 archive of domain 0: a collection, a tagged document, a
/// two-revision multi-chunk document and an empty one. Returns the bytes and
/// the exporter's own summary.
fn real_archive(big: usize) -> (Vec<u8>, archive::Summary) {
    let mut disk = Mem(vec![[0; VSECTOR]; VOLUME_SECTORS as usize]);
    let mut service = Service::new(EPOCH, Table::EMPTY, false);
    let mut c = Client::new(
        Local {
            service: &mut service,
            disk: &mut disk,
        },
        EPOCH,
        100,
    );
    c.collection(0, 1, 0, b"Werk").unwrap();
    let r = c.write(0, 0, b"Notes", 300, fill(1)).unwrap().0;
    c.meta(0, r, 0x55, 7, 1).unwrap();
    c.write(1, 0, b"Plan", 40, fill(2)).unwrap();
    c.write(1, 1, b"Plan", big, fill(3)).unwrap();
    c.write(5, 0, b"Empty", 0, fill(4)).unwrap();
    let mut out = Bytes(Vec::new());
    let summary = archive::export(&mut c, 0, &mut out).unwrap();
    (out.0, summary)
}

/// Serves bytes in uneven pieces, then optionally fails.
struct Feed<'a> {
    bytes: &'a [u8],
    at: usize,
    fail_at: Option<(usize, Status)>,
}
impl Source for Feed<'_> {
    fn get(&mut self, buffer: &mut [u8]) -> Result<usize, Status> {
        if let Some((at, status)) = self.fail_at {
            if self.at >= at {
                return Err(status);
            }
        }
        let amount = buffer.len().min(self.bytes.len() - self.at).min(77);
        buffer[..amount].copy_from_slice(&self.bytes[self.at..self.at + amount]);
        self.at += amount;
        Ok(amount)
    }
}
fn feed(bytes: &[u8]) -> Feed<'_> {
    Feed {
        bytes,
        at: 0,
        fail_at: None,
    }
}

fn real(mut h: H) -> H {
    h.verifier = AnyVerifier::Real(Box::default());
    h
}

/// The adapter accepts a real archive and reports the exporter's own
/// generation and object count.
/// Kills: a summary that is not taken from `archive::verify`.
#[test]
fn accepts_a_real_archive_with_its_summary() {
    let (bytes, summary) = real_archive(9000);
    let mut verifier = VolumeVerifier::new();
    let got = verifier.verify(&mut feed(&bytes)).unwrap();
    assert_eq!(got.generation, summary.generation);
    assert_eq!(got.objects, summary.objects as u64);
    assert_eq!(got.objects, 3);
    // No state carried between calls.
    assert_eq!(verifier.verify(&mut feed(&bytes)).unwrap(), got);
}

/// A stand-in-shaped archive (v1 magic, valid trailer FNV) is not a v1
/// archive; nor is a real one with a byte flipped, cut short or extended.
/// Kills: an adapter that accepts on the magic and trailer alone, or that
/// maps a `verify` refusal to `Ok`.
#[test]
fn refuses_everything_that_is_not_exactly_a_v1_archive() {
    let mut verifier = VolumeVerifier::new();
    assert_eq!(
        verifier.verify(&mut feed(&archive(1, 1, 300, 2))),
        Err(Status::Corrupt)
    );
    let (bytes, _) = real_archive(5000);
    for at in [
        0,
        9,
        70,
        200,
        bytes.len() / 2,
        bytes.len() - 9,
        bytes.len() - 1,
    ] {
        let mut flipped = bytes.clone();
        flipped[at] ^= 0x10;
        assert_eq!(
            verifier.verify(&mut feed(&flipped)),
            Err(Status::Corrupt),
            "flip at {at}"
        );
    }
    for cut in [0, 1, 63, 64, bytes.len() - 1] {
        assert_eq!(
            verifier.verify(&mut feed(&bytes[..cut])),
            Err(Status::Corrupt),
            "cut at {cut}"
        );
    }
    let mut extended = bytes.clone();
    extended.push(0);
    assert_eq!(verifier.verify(&mut feed(&extended)), Err(Status::Corrupt));
    assert!(verifier.verify(&mut feed(&bytes)).is_ok());
}

/// A source error is returned unchanged (2.3), for every service status and
/// at every phase of the walk, including statuses that `verify` itself also
/// produces (`Corrupt`, `Protocol`).
/// Equivalent mutants (measured 24 November 2026): dropping the adapter's own
/// passthrough, or keeping the last instead of the first source error,
/// survives, because `archive::verify` today returns a source error unchanged
/// and stops at the first one, and both enums share their codes. The adapter
/// keeps the passthrough so that a later change to `verify` cannot alter it;
/// this test then detects such a change.
#[test]
fn returns_a_source_error_unchanged() {
    let (bytes, _) = real_archive(5000);
    let statuses = [
        Status::Invalid,
        Status::NoSys,
        Status::Range,
        Status::Access,
        Status::Stale,
        Status::Io,
        Status::Timeout,
        Status::NoDevice,
        Status::Protocol,
        Status::Unknown,
        Status::Full,
        Status::Corrupt,
        Status::NotFound,
        Status::Busy,
    ];
    let mut verifier = VolumeVerifier::new();
    for status in statuses {
        // 0: header; 100: records; 1000: content; len: the end-of-stream probe.
        for at in [0, 100, 1000, bytes.len()] {
            let mut source = Feed {
                bytes: &bytes,
                at: 0,
                fail_at: Some((at, status)),
            };
            assert_eq!(
                verifier.verify(&mut source),
                Err(status),
                "{status:?} at {at}"
            );
        }
    }
    // The next call starts clean.
    assert!(verifier.verify(&mut feed(&bytes)).is_ok());
}

/// The service seals, lists, verifies and restores a real archive with the
/// production verifier, byte-identical after a cold boot.
#[test]
fn service_seals_and_verifies_a_real_archive() {
    let (bytes, summary) = real_archive(9000);
    let mut h = real(H::formatted());
    let e1 = h.export(1, &bytes);
    let mut h = real(h.reboot());
    assert_eq!(h.list_ids(0), (Status::Ok, vec![e1]));
    assert_eq!(h.bind(1, wire::BIND_VERIFY, e1), Status::Ok);
    assert_eq!(h.verify_entry(1, e1), Status::Ok);
    assert_eq!(h.bind(2, wire::BIND_RESTORE, e1), Status::Ok);
    let read = h.read_entry(2, e1, bytes.len() as u64).unwrap();
    assert_eq!(read, bytes);
    // The portal's own check over READ accepts the same bytes (2.3).
    let got = VolumeVerifier::new().verify(&mut feed(&read)).unwrap();
    assert_eq!(got.generation, summary.generation);
}

/// N7 with the production verifier: one byte flipped in a sealed extent;
/// VERIFY `Corrupt`, and the portal's `archive::verify` over READ refuses it.
#[test]
fn n7_corrupted_entry_real_verifier() {
    let (bytes, _) = real_archive(9000);
    let mut h = real(H::formatted());
    let e1 = h.export(1, &bytes);
    let first = h.svc.catalog().find(e1).unwrap().first;
    let mut image = h.disk.durable.clone();
    image[first as usize + 3][100] ^= 0x40;
    let mut h = real(H::cold(image));
    assert_eq!(h.bind(1, wire::BIND_VERIFY, e1), Status::Ok);
    assert_eq!(h.verify_entry(1, e1), Status::Corrupt);
    assert_eq!(h.bind(2, wire::BIND_RESTORE, e1), Status::Ok);
    let read = h.read_entry(2, e1, bytes.len() as u64).unwrap();
    assert_eq!(
        VolumeVerifier::new().verify(&mut feed(&read)),
        Err(Status::Corrupt)
    );
}

/// N10 with the production verifier: a stand-in-shaped archive, a real one
/// with one byte flipped and a real one with trailing bytes are each refused
/// by SEAL with `Corrupt`; nothing is listed and `next_free` is unchanged
/// after a reboot.
#[test]
fn n10_non_archive_bytes_real_verifier() {
    let (good, _) = real_archive(5000);
    let mut flipped = good.clone();
    flipped[good.len() / 2] ^= 1;
    let mut padded = good.clone();
    padded.extend_from_slice(&[0; 5]);
    for bytes in [archive(1, 1, 300, 2), flipped, padded] {
        let mut h = real(H::formatted());
        let before = h.svc.catalog().next_free;
        assert_eq!(h.bind(1, wire::BIND_EXPORT, 0), Status::Ok);
        let entry = h.write_all(1, &bytes);
        assert_eq!(h.seal(1, entry, bytes.len() as u64).status, Status::Corrupt);
        assert!(!h.svc.safe_path(), "a failed verify is not a device error");
        assert_eq!(h.list_ids(0).1, Vec::<u64>::new());
        let cold = h.reboot();
        assert_eq!(cold.svc.catalog().next_free, before);
    }
    // The unmodified archive seals.
    let mut h = real(H::formatted());
    h.export(1, &good);
}
