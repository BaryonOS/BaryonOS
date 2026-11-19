use app_package_core::{
    abi_version, Manifest, SourceId, SourceRegistry, TrustAnchors, TrustReject, MANIFEST_LEN,
};
use nieuw_os_sdk_app_management::{
    catalog::CatalogError,
    installer::{InstallRequest, Installer, InstallerError},
    journal::{slot_len, Journal, JournalError, JournalMedia, MediaError},
    protocol::UploadAction,
    store::{
        ArtifactDescriptor, ArtifactRef, ArtifactWriter, StoreError, TransactionKey,
        TransactionOutcome, READ_CHUNK_MAX,
    },
};
use sha2::{Digest, Sha256};

const V1: &[u8] = include_bytes!("fixtures/test-v1.pkg");
const V2: &[u8] = include_bytes!("fixtures/test-v2.pkg");
const ALT: &[u8] = include_bytes!("fixtures/alternate-v2.pkg");
const SOURCE: SourceId = SourceId::from_bytes([1; 32]);
const OTHER: SourceId = SourceId::from_bytes([2; 32]);

fn anchors(alternate: bool) -> TrustAnchors {
    if alternate {
        TrustAnchors {
            ed25519: *include_bytes!("fixtures/alternate-ed25519.bin"),
            mldsa65: *include_bytes!("fixtures/alternate-mldsa65.bin"),
        }
    } else {
        TrustAnchors {
            ed25519: *include_bytes!("fixtures/test-ed25519.bin"),
            mldsa65: *include_bytes!("fixtures/test-mldsa65.bin"),
        }
    }
}

fn registry() -> SourceRegistry {
    let mut registry = SourceRegistry::new();
    registry.add(SOURCE, anchors(false)).unwrap();
    registry.add(OTHER, anchors(true)).unwrap();
    registry
}

fn key(request_id: u64) -> TransactionKey {
    TransactionKey {
        caller: 9,
        request_id,
        service_epoch: 7,
    }
}

fn manifest(package: &[u8]) -> Manifest {
    Manifest::decode(package[..MANIFEST_LEN].try_into().unwrap()).unwrap()
}

fn request(package: &[u8], request_id: u64, generation: u64) -> InstallRequest<'_> {
    let manifest = manifest(package);
    InstallRequest {
        key: key(request_id),
        expected_generation: generation,
        action: if generation == 1 {
            UploadAction::Install
        } else {
            UploadAction::Update
        },
        source_id: SOURCE,
        source_generation: 1,
        current_abi: abi_version(1, 0),
        package,
        artifact: ArtifactDescriptor {
            reference: ArtifactRef {
                artifact_id: [request_id as u8; 32],
                immutable_generation: 1,
            },
            length: u64::from(manifest.image_length),
            sha256: manifest.image_sha256,
        },
    }
}

struct Media {
    slots: [Vec<u8>; 2],
    fail_write: bool,
    fail_flush: bool,
    fail_read: bool,
    writes: usize,
}

impl Media {
    fn formatted() -> Self {
        let mut media = Self {
            slots: [vec![0; slot_len(2)], vec![0; slot_len(2)]],
            fail_write: false,
            fail_flush: false,
            fail_read: false,
            writes: 0,
        };
        Journal::new().format(&mut media).unwrap();
        media
    }
}

impl JournalMedia for Media {
    fn read(&mut self, slot: usize, offset: usize, out: &mut [u8]) -> Result<(), MediaError> {
        if self.fail_read {
            return Err(MediaError);
        }
        out.copy_from_slice(&self.slots[slot][offset..offset + out.len()]);
        Ok(())
    }
    fn write(&mut self, slot: usize, offset: usize, data: &[u8]) -> Result<(), MediaError> {
        self.writes += 1;
        if self.fail_write {
            return Err(MediaError);
        }
        self.slots[slot][offset..offset + data.len()].copy_from_slice(data);
        Ok(())
    }
    fn flush(&mut self) -> Result<(), MediaError> {
        if self.fail_flush {
            Err(MediaError)
        } else {
            Ok(())
        }
    }
}

#[derive(Default)]
struct Writer {
    bytes: Vec<u8>,
    descriptor: Option<ArtifactDescriptor>,
    calls: usize,
    commits: usize,
    cancels: usize,
    fail_begin: Option<StoreError>,
    fail_write: Option<StoreError>,
    fail_commit: Option<StoreError>,
    fail_cancel: Option<StoreError>,
    wrong_ack: bool,
    wrong_ref: bool,
    truncate: bool,
    corrupt: bool,
}

impl ArtifactWriter for Writer {
    fn begin(
        &mut self,
        _key: TransactionKey,
        descriptor: &ArtifactDescriptor,
    ) -> Result<(), StoreError> {
        self.calls += 1;
        if let Some(error) = self.fail_begin {
            return Err(error);
        }
        self.descriptor = Some(*descriptor);
        self.bytes.clear();
        Ok(())
    }
    fn write_chunk(
        &mut self,
        _key: TransactionKey,
        offset: u64,
        bytes: &[u8],
    ) -> Result<u64, StoreError> {
        self.calls += 1;
        if let Some(error) = self.fail_write {
            return Err(error);
        }
        assert!(!bytes.is_empty() && bytes.len() <= READ_CHUNK_MAX);
        assert_eq!(offset as usize, self.bytes.len());
        self.bytes.extend_from_slice(bytes);
        if self.wrong_ack {
            return Ok(offset);
        }
        Ok(self.bytes.len() as u64)
    }
    fn commit(
        &mut self,
        _key: TransactionKey,
        descriptor: &ArtifactDescriptor,
    ) -> Result<ArtifactRef, StoreError> {
        self.calls += 1;
        self.commits += 1;
        if let Some(error) = self.fail_commit {
            return Err(error);
        }
        assert_eq!(self.descriptor, Some(*descriptor));
        if self.truncate {
            self.bytes.pop();
        }
        if self.corrupt {
            self.bytes[0] ^= 1;
        }
        if self.bytes.len() as u64 != descriptor.length {
            return Err(StoreError::Truncated);
        }
        if Sha256::digest(&self.bytes)[..] != descriptor.sha256 {
            return Err(StoreError::Corrupt);
        }
        if self.wrong_ref {
            return Ok(ArtifactRef {
                immutable_generation: 2,
                ..descriptor.reference
            });
        }
        Ok(descriptor.reference)
    }
    fn cancel(&mut self, _key: TransactionKey) -> Result<(), StoreError> {
        self.calls += 1;
        self.cancels += 1;
        if let Some(error) = self.fail_cancel {
            return Err(error);
        }
        self.bytes.clear();
        Ok(())
    }
    fn outcome(&mut self, _key: TransactionKey) -> Result<TransactionOutcome, StoreError> {
        panic!("installer must not query or automatically replay uncertain work")
    }
}

fn setup() -> (Installer<2>, Media, Writer, SourceRegistry) {
    let mut media = Media::formatted();
    let installer = Installer::open(&mut media, 7).unwrap();
    (installer, media, Writer::default(), registry())
}

fn install(
    installer: &mut Installer<2>,
    media: &mut Media,
    writer: &mut Writer,
    registry: &SourceRegistry,
) {
    installer
        .stage(registry, writer, request(V1, 1, 1))
        .unwrap();
    assert_eq!(installer.generation(), Ok(1));
    installer.publish(registry, media, key(1)).unwrap();
}

#[test]
fn install_update_remove_persist_exact_images_and_grant_nothing() {
    let (mut installer, mut media, mut writer, registry) = setup();
    install(&mut installer, &mut media, &mut writer, &registry);
    let app_id = *manifest(V1).app_id.as_bytes();
    let installed = *installer.lookup(&app_id).unwrap().unwrap();
    assert_eq!(writer.bytes, V1[MANIFEST_LEN..]);
    assert_eq!(installed.identity.version(), 1);
    assert_eq!(installed.identity.source_id(), SOURCE.as_bytes());
    assert_eq!(installed.identity.source_generation(), 1);
    // CatalogEntry has only provenance and artifact metadata, no grant field or capability.
    let nieuw_os_sdk_app_management::catalog::CatalogEntry {
        identity: _,
        artifact: _,
    } = installed;
    let mut reopened = Installer::<2>::open(&mut media, 8).unwrap();
    assert_eq!(reopened.lookup(&app_id), Ok(Some(&installed)));
    let mut update = request(V2, 2, 2);
    update.key.service_epoch = 8;
    reopened.stage(&registry, &mut writer, update).unwrap();
    let update_key = TransactionKey {
        service_epoch: 8,
        ..key(2)
    };
    let change = reopened.publish(&registry, &mut media, update_key).unwrap();
    assert_eq!(change.released, Some(installed.artifact.reference));
    assert_eq!(writer.bytes, V2[MANIFEST_LEN..]);
    assert_eq!(
        reopened
            .lookup(&app_id)
            .unwrap()
            .unwrap()
            .identity
            .version(),
        2
    );
    let removed = reopened
        .remove(
            &mut media,
            TransactionKey {
                service_epoch: 8,
                ..key(3)
            },
            3,
            &app_id,
        )
        .unwrap();
    assert_eq!(removed.released.unwrap().artifact_id, [2; 32]);
    assert_eq!(writer.commits, 2);
    assert_eq!(
        Installer::<2>::open(&mut media, 9).unwrap().lookup(&app_id),
        Ok(None)
    );
}

#[test]
fn signatures_framing_and_source_policy_fail_before_storage() {
    let mut invalid_ed = V1.to_vec();
    invalid_ed[144] ^= 1;
    let mut invalid_ml = V1.to_vec();
    invalid_ml[208] ^= 1;
    let mut invalid_image = V1.to_vec();
    invalid_image[MANIFEST_LEN] ^= 1;
    for package in [
        &invalid_ed[..],
        &invalid_ml[..],
        &invalid_image[..],
        &V1[..V1.len() - 1],
    ] {
        let (mut installer, _, mut writer, registry) = setup();
        assert!(matches!(
            installer.stage(&registry, &mut writer, request(package, 1, 1)),
            Err(InstallerError::Source(TrustReject::Executable(_)))
        ));
        assert_eq!(writer.calls, 0);
        assert_eq!(installer.generation(), Ok(1));
    }
    for (source, generation, revoked, expected) in [
        (OTHER, 1, false, None),
        (
            SourceId::from_bytes([3; 32]),
            1,
            false,
            Some(TrustReject::UnknownSource),
        ),
        (SOURCE, 2, false, Some(TrustReject::StaleSourceGeneration)),
        (SOURCE, 1, true, Some(TrustReject::RevokedSource)),
    ] {
        let (mut installer, _, mut writer, mut registry) = setup();
        if revoked {
            registry.revoke(SOURCE, 1).unwrap();
        }
        let mut input = request(V1, 1, 1);
        input.source_id = source;
        input.source_generation = generation;
        let error = installer.stage(&registry, &mut writer, input).unwrap_err();
        if let Some(expected) = expected {
            assert_eq!(error, InstallerError::Source(expected));
        } else {
            assert!(matches!(
                error,
                InstallerError::Source(TrustReject::Executable(_))
            ));
        }
        assert_eq!(writer.calls, 0);
    }
}

#[test]
fn rotation_or_revocation_after_store_commit_prevents_catalog_mutation() {
    for revoked in [false, true] {
        for update in [false, true] {
            let (mut installer, mut media, mut writer, _) = setup();
            let mut registry = SourceRegistry::new();
            registry.add(SOURCE, anchors(false)).unwrap();
            if update {
                install(&mut installer, &mut media, &mut writer, &registry);
            }
            let (package, id, generation) = if update { (V2, 2, 2) } else { (V1, 1, 1) };
            installer
                .stage(&registry, &mut writer, request(package, id, generation))
                .unwrap();
            let before = media.slots.clone();
            let expected = if revoked {
                registry.revoke(SOURCE, 1).unwrap();
                TrustReject::RevokedSource
            } else {
                registry.rotate(SOURCE, 1, anchors(true)).unwrap();
                TrustReject::StaleSourceGeneration
            };
            assert_eq!(
                installer.publish(&registry, &mut media, key(id)),
                Err(InstallerError::Source(expected))
            );
            assert_eq!(installer.generation(), Ok(generation));
            assert_eq!(media.slots, before);
            assert_eq!(
                installer
                    .lookup(manifest(V1).app_id.as_bytes())
                    .unwrap()
                    .map(|entry| entry.identity.version()),
                if update { Some(1) } else { None }
            );
            assert_eq!(
                installer.publish(&registry, &mut media, key(id)),
                Err(InstallerError::Catalog(CatalogError::NotFound))
            );
        }
    }
}

#[test]
fn downgrade_source_switch_and_descriptor_substitution_are_rejected() {
    for source_switch in [false, true] {
        let (mut installer, mut media, mut writer, registry) = setup();
        install(&mut installer, &mut media, &mut writer, &registry);
        let mut input = request(if source_switch { ALT } else { V1 }, 2, 2);
        if source_switch {
            input.source_id = OTHER;
        }
        installer.stage(&registry, &mut writer, input).unwrap();
        let before = media.slots.clone();
        let expected = if source_switch {
            CatalogError::SourceMismatch
        } else {
            CatalogError::VersionRollback
        };
        assert_eq!(
            installer.publish(&registry, &mut media, key(2)),
            Err(InstallerError::Catalog(expected))
        );
        assert_eq!(installer.generation(), Ok(2));
        assert_eq!(media.slots, before);
    }
    for digest in [false, true] {
        let (mut installer, _, mut writer, registry) = setup();
        let mut input = request(V1, 1, 1);
        if digest {
            input.artifact.sha256[0] ^= 1;
        } else {
            input.artifact.length += 1;
        }
        assert_eq!(
            installer.stage(&registry, &mut writer, input),
            Err(InstallerError::Catalog(CatalogError::ArtifactMismatch))
        );
        assert_eq!(writer.calls, 0);
    }
    for corrupt in [false, true] {
        let (mut installer, _, mut writer, registry) = setup();
        writer.truncate = !corrupt;
        writer.corrupt = corrupt;
        let expected = if corrupt {
            StoreError::Corrupt
        } else {
            StoreError::Truncated
        };
        assert_eq!(
            installer.stage(&registry, &mut writer, request(V1, 1, 1)),
            Err(InstallerError::Store(expected))
        );
        assert_eq!(writer.cancels, 1);
        assert_eq!(installer.generation(), Ok(1));
    }
}

#[test]
fn uncertain_store_outcomes_block_lookup_replay_and_recommit_until_reconciled() {
    for fault in 0..6 {
        let (mut installer, mut media, mut writer, registry) = setup();
        match fault {
            0 => writer.fail_begin = Some(StoreError::Unknown),
            1 => writer.fail_write = Some(StoreError::Unknown),
            2 => writer.fail_commit = Some(StoreError::Unknown),
            3 => writer.wrong_ack = true,
            4 => writer.wrong_ref = true,
            _ => {
                writer.fail_write = Some(StoreError::Full);
                writer.fail_cancel = Some(StoreError::Unknown);
            }
        }
        assert!(installer
            .stage(&registry, &mut writer, request(V1, 1, 1))
            .is_err());
        let calls = writer.calls;
        assert_eq!(
            installer.lookup(manifest(V1).app_id.as_bytes()),
            Err(InstallerError::Catalog(CatalogError::Unknown))
        );
        for id in [1, 2] {
            assert_eq!(
                installer.stage(&registry, &mut writer, request(V1, id, 1)),
                Err(InstallerError::Catalog(CatalogError::Unknown))
            );
        }
        assert_eq!(
            installer.publish(&registry, &mut media, key(1)),
            Err(InstallerError::Catalog(CatalogError::Unknown))
        );
        assert_eq!(
            installer.remove(&mut media, key(2), 1, manifest(V1).app_id.as_bytes()),
            Err(InstallerError::Catalog(CatalogError::Unknown))
        );
        assert_eq!(writer.calls, calls);
        media.fail_read = true;
        assert_eq!(
            installer.reconcile(&mut media, 8),
            Err(InstallerError::Journal(JournalError::Media))
        );
        assert_eq!(
            installer.generation(),
            Err(InstallerError::Catalog(CatalogError::Unknown))
        );
        media.fail_read = false;
        installer.reconcile(&mut media, 8).unwrap();
        assert_eq!(installer.lookup(manifest(V1).app_id.as_bytes()), Ok(None));
        assert_eq!(writer.calls, calls);
        let mut replay = request(V1, 1, 1);
        replay.key.service_epoch = 8;
        assert_eq!(
            installer.stage(&registry, &mut writer, replay),
            Err(InstallerError::Store(StoreError::Replay))
        );
    }
}

#[test]
fn journal_failures_hide_speculative_state_without_automatic_retry() {
    for flush in [false, true] {
        for remove in [false, true] {
            let (mut installer, mut media, mut writer, registry) = setup();
            if remove {
                install(&mut installer, &mut media, &mut writer, &registry);
            } else {
                installer
                    .stage(&registry, &mut writer, request(V1, 1, 1))
                    .unwrap();
            }
            media.fail_write = !flush;
            media.fail_flush = flush;
            let result = if remove {
                installer.remove(&mut media, key(2), 2, manifest(V1).app_id.as_bytes())
            } else {
                installer.publish(&registry, &mut media, key(1))
            };
            assert_eq!(result, Err(InstallerError::Journal(JournalError::Media)));
            let writes = media.writes;
            let calls = writer.calls;
            assert_eq!(
                installer.lookup(manifest(V1).app_id.as_bytes()),
                Err(InstallerError::Catalog(CatalogError::Unknown))
            );
            assert_eq!(
                installer.stage(&registry, &mut writer, request(V2, 3, 2)),
                Err(InstallerError::Catalog(CatalogError::Unknown))
            );
            assert_eq!(
                installer.publish(&registry, &mut media, key(1)),
                Err(InstallerError::Catalog(CatalogError::Unknown))
            );
            assert_eq!(media.writes, writes);
            assert_eq!(writer.calls, calls);
            media.fail_write = false;
            media.fail_flush = false;
            installer.reconcile(&mut media, 8).unwrap();
            // A failed flush may already have persisted the new slot; recovery decides, not replay.
            assert_eq!(
                installer.generation(),
                Ok(if flush {
                    if remove {
                        3
                    } else {
                        2
                    }
                } else if remove {
                    2
                } else {
                    1
                })
            );
            assert_eq!(writer.calls, calls);
        }
    }
}

#[test]
fn revoked_source_keeps_installed_lookup_but_blocks_new_install_and_update() {
    let (mut installer, mut media, mut writer, mut registry) = setup();
    install(&mut installer, &mut media, &mut writer, &registry);
    let app_id = *manifest(V1).app_id.as_bytes();
    let before = *installer.lookup(&app_id).unwrap().unwrap();
    registry.revoke(SOURCE, 1).unwrap();
    assert_eq!(installer.lookup(&app_id), Ok(Some(&before)));
    for action in [UploadAction::Install, UploadAction::Update] {
        let mut input = request(
            V2,
            if action == UploadAction::Install {
                2
            } else {
                3
            },
            2,
        );
        input.action = action;
        assert_eq!(
            installer.stage(&registry, &mut writer, input),
            Err(InstallerError::Source(TrustReject::RevokedSource))
        );
    }
    installer.remove(&mut media, key(4), 2, &app_id).unwrap();
    assert_eq!(installer.lookup(&app_id), Ok(None));
}

#[test]
fn transaction_binding_busy_and_replay_checks_precede_io() {
    let (mut installer, mut media, mut writer, registry) = setup();
    for input in [
        InstallRequest {
            key: TransactionKey {
                caller: 0,
                ..key(1)
            },
            ..request(V1, 1, 1)
        },
        InstallRequest {
            key: TransactionKey {
                service_epoch: 8,
                ..key(1)
            },
            ..request(V1, 1, 1)
        },
        request(V1, 1, 2),
    ] {
        assert!(installer.stage(&registry, &mut writer, input).is_err());
    }
    assert_eq!(writer.calls, 0);
    installer
        .stage(&registry, &mut writer, request(V1, 1, 1))
        .unwrap();
    let calls = writer.calls;
    assert_eq!(
        installer.stage(&registry, &mut writer, request(V2, 2, 1)),
        Err(InstallerError::Catalog(CatalogError::Busy))
    );
    for wrong in [
        TransactionKey {
            caller: 10,
            ..key(1)
        },
        key(2),
        TransactionKey {
            service_epoch: 8,
            ..key(1)
        },
    ] {
        assert_eq!(
            installer.publish(&registry, &mut media, wrong),
            Err(InstallerError::Catalog(CatalogError::Stale))
        );
    }
    installer.publish(&registry, &mut media, key(1)).unwrap();
    assert_eq!(
        installer.stage(&registry, &mut writer, request(V2, 1, 2)),
        Err(InstallerError::Store(StoreError::Replay))
    );
    assert_eq!(writer.calls, calls);
}

#[test]
fn open_never_formats_or_accepts_invalid_recovery() {
    let mut media = Media::formatted();
    let writes = media.writes;
    assert!(matches!(
        Installer::<2>::open(&mut media, 0),
        Err(InstallerError::Catalog(CatalogError::Stale))
    ));
    media.slots.iter_mut().for_each(|slot| slot.fill(0));
    assert!(matches!(
        Installer::<2>::open(&mut media, 7),
        Err(InstallerError::Journal(JournalError::NotFormatted))
    ));
    media.slots[0][0] = 1;
    assert!(matches!(
        Installer::<2>::open(&mut media, 7),
        Err(InstallerError::Journal(JournalError::Corrupt))
    ));
    assert_eq!(media.writes, writes);
}

#[test]
fn a_new_verification_after_rotation_can_update_the_installed_app() {
    let (mut installer, mut media, mut writer, _) = setup();
    let mut registry = SourceRegistry::new();
    registry.add(SOURCE, anchors(false)).unwrap();
    install(&mut installer, &mut media, &mut writer, &registry);
    registry.rotate(SOURCE, 1, anchors(true)).unwrap();
    let mut update = request(ALT, 2, 2);
    update.source_generation = 2;
    installer.stage(&registry, &mut writer, update).unwrap();
    installer.publish(&registry, &mut media, key(2)).unwrap();
    let entry = installer
        .lookup(manifest(ALT).app_id.as_bytes())
        .unwrap()
        .unwrap();
    assert_eq!(entry.identity.source_generation(), 2);
    assert_eq!(entry.identity.version(), 2);
}
