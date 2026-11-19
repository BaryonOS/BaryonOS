//! Bounded installer composition for host adapters, without installation or launch authority.
//!
//! The caller supplies the live source registry and separately authorizes each operation.
//! `stage` verifies and commits exact image bytes; `publish` checks the current source
//! before catalog publication. Rejected publication can leave an unreferenced artifact.
//! Journal media supplies host-model recovery only, not authenticated cold recovery.

use app_package_core::{
    SourceId, SourceRecord, SourceRegistry, SourceStatus, TrustAnchors, TrustReject, APP_ID_LEN,
    MAX_SOURCES,
};

use crate::catalog::{CatalogChange, CatalogEntry, CatalogError, CatalogOp, InstallCatalog};
use crate::journal::{Journal, JournalError, JournalMedia};
use crate::protocol::UploadAction;
use crate::store::{
    ArtifactDescriptor, ArtifactWriter, InstallState, InstallTransaction, StoreError,
    TransactionKey, READ_CHUNK_MAX,
};
use crate::PackageIdentity;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InstallerError {
    Source(TrustReject),
    Store(StoreError),
    Catalog(CatalogError),
    Journal(JournalError),
}

impl From<CatalogError> for InstallerError {
    fn from(error: CatalogError) -> Self {
        Self::Catalog(error)
    }
}

/// Untrusted operation inputs. Sender and epoch must come from the future trusted adapter.
pub struct InstallRequest<'a> {
    pub key: TransactionKey,
    pub expected_generation: u64,
    pub action: UploadAction,
    pub source_id: SourceId,
    pub source_generation: u32,
    pub current_abi: u32,
    pub package: &'a [u8],
    pub artifact: ArtifactDescriptor,
}

struct Pending {
    key: TransactionKey,
    expected_generation: u64,
    op: CatalogOp,
    entry: CatalogEntry,
}

/// Owns its catalogue so uncertain speculative state cannot escape via a raw accessor.
pub struct Installer<const N: usize> {
    catalog: InstallCatalog<N>,
    journal: Journal,
    transaction: InstallTransaction,
    pending: Option<Pending>,
    last_request_id: u64,
    uncertain: bool,
}

impl<const N: usize> Installer<N> {
    /// Opens an already formatted journal. Never silently formats or falls back to empty state.
    pub fn open<M: JournalMedia>(
        media: &mut M,
        service_epoch: u64,
    ) -> Result<Self, InstallerError> {
        if service_epoch == 0 {
            return Err(CatalogError::Stale.into());
        }
        let mut catalog = InstallCatalog::new(service_epoch);
        let mut journal = Journal::new();
        journal
            .recover_into(media, &mut catalog)
            .map_err(InstallerError::Journal)?;
        Ok(Self {
            catalog,
            journal,
            transaction: InstallTransaction::new(),
            pending: None,
            last_request_id: 0,
            uncertain: false,
        })
    }

    pub fn generation(&self) -> Result<u64, InstallerError> {
        self.check_certain()?;
        Ok(self.catalog.generation())
    }

    /// Source revocation does not remove an installed entry or issue any launch grant.
    pub fn lookup(
        &self,
        app_id: &[u8; APP_ID_LEN],
    ) -> Result<Option<&CatalogEntry>, InstallerError> {
        self.check_certain()?;
        self.catalog.lookup(app_id).map_err(InstallerError::Catalog)
    }

    /// Verifies only through SourceRegistry and streams only its immutable verified image.
    pub fn stage<W: ArtifactWriter>(
        &mut self,
        registry: &SourceRegistry,
        writer: &mut W,
        request: InstallRequest<'_>,
    ) -> Result<(), InstallerError> {
        self.start_request(request.key, request.expected_generation)?;
        let verified = registry
            .verify_executable(
                request.source_id,
                request.source_generation,
                request.package,
                request.current_abi,
            )
            .map_err(InstallerError::Source)?;
        let identity = PackageIdentity::from_verified(&verified);
        if request.artifact.length != u64::from(identity.image_length())
            || request.artifact.sha256 != *identity.image_sha256()
        {
            return Err(CatalogError::ArtifactMismatch.into());
        }
        let key = request.key;
        let stored = (|| {
            self.transaction.begin(
                writer,
                key.caller,
                key.request_id,
                key.service_epoch,
                request.artifact,
            )?;
            for chunk in verified.executable().image().chunks(READ_CHUNK_MAX) {
                self.transaction.append(
                    writer,
                    key.caller,
                    key.request_id,
                    key.service_epoch,
                    self.transaction.received(),
                    chunk,
                )?;
            }
            self.transaction
                .commit(writer, key.caller, key.request_id, key.service_epoch)
        })();
        if let Err(error) = stored {
            if self.transaction.state() == InstallState::Unknown {
                self.uncertain = true;
            } else if matches!(
                self.transaction.state(),
                InstallState::Failed | InstallState::Receiving | InstallState::Ready
            ) {
                // Definitive failure may release staging, but an uncertain cancellation must latch too.
                if let Err(cancel_error) =
                    self.transaction
                        .cancel(writer, key.caller, key.request_id, key.service_epoch)
                {
                    self.uncertain = true;
                    return Err(InstallerError::Store(cancel_error));
                }
            }
            return Err(InstallerError::Store(error));
        }
        self.pending = Some(Pending {
            key,
            expected_generation: request.expected_generation,
            op: match request.action {
                UploadAction::Install => CatalogOp::Install,
                UploadAction::Update => CatalogOp::Update,
            },
            entry: CatalogEntry {
                identity,
                artifact: request.artifact,
            },
        });
        Ok(())
    }

    /// The shared registry borrow serializes the final check with prepare/apply.
    /// The adapter must supply the same live authority, not an older reconstructed registry.
    pub fn publish<M: JournalMedia>(
        &mut self,
        registry: &SourceRegistry,
        media: &mut M,
        key: TransactionKey,
    ) -> Result<CatalogChange, InstallerError> {
        self.check_certain()?;
        let pending = self.pending.as_ref().ok_or(CatalogError::NotFound)?;
        if pending.key != key {
            return Err(CatalogError::Stale.into());
        }
        let pending = self.pending.take().ok_or(CatalogError::NotFound)?;
        check_source(registry, &pending.entry.identity).map_err(InstallerError::Source)?;
        let intent = self.catalog.prepare(
            pending.op,
            key.service_epoch,
            pending.expected_generation,
            pending.entry,
        )?;
        let change = self
            .catalog
            .apply(&intent, Some(pending.entry.artifact.reference))?;
        self.persist(media, change)
    }

    pub fn remove<M: JournalMedia>(
        &mut self,
        media: &mut M,
        key: TransactionKey,
        expected_generation: u64,
        app_id: &[u8; APP_ID_LEN],
    ) -> Result<CatalogChange, InstallerError> {
        self.start_request(key, expected_generation)?;
        let entry = self
            .catalog
            .lookup(app_id)?
            .copied()
            .ok_or(CatalogError::NotFound)?;
        let intent = self.catalog.prepare(
            CatalogOp::Remove,
            key.service_epoch,
            expected_generation,
            entry,
        )?;
        let change = self.catalog.apply(&intent, None)?;
        self.persist(media, change)
    }

    /// Explicit authoritative host-model reconciliation; never replays a store operation.
    /// The adapter must establish freshness/authenticity separately for production use.
    pub fn reconcile<M: JournalMedia>(
        &mut self,
        media: &mut M,
        new_service_epoch: u64,
    ) -> Result<(), InstallerError> {
        if new_service_epoch <= self.catalog.service_epoch() {
            return Err(CatalogError::Stale.into());
        }
        let recovered = Self::open(media, new_service_epoch)?;
        let last_request_id = self.last_request_id;
        *self = recovered;
        self.last_request_id = last_request_id;
        Ok(())
    }

    fn check_certain(&self) -> Result<(), InstallerError> {
        if self.uncertain {
            Err(CatalogError::Unknown.into())
        } else {
            Ok(())
        }
    }

    fn start_request(
        &mut self,
        key: TransactionKey,
        generation: u64,
    ) -> Result<(), InstallerError> {
        self.check_certain()?;
        if self.pending.is_some() {
            return Err(CatalogError::Busy.into());
        }
        if key.caller == 0 || key.request_id == 0 {
            return Err(CatalogError::Invalid.into());
        }
        if key.service_epoch != self.catalog.service_epoch()
            || generation != self.catalog.generation()
        {
            return Err(CatalogError::Stale.into());
        }
        if key.request_id <= self.last_request_id {
            return Err(InstallerError::Store(StoreError::Replay));
        }
        self.last_request_id = key.request_id;
        Ok(())
    }

    fn persist<M: JournalMedia>(
        &mut self,
        media: &mut M,
        change: CatalogChange,
    ) -> Result<CatalogChange, InstallerError> {
        if let Err(error) = self.journal.commit(media, &self.catalog) {
            self.uncertain = true;
            return Err(InstallerError::Journal(error));
        }
        Ok(change)
    }
}

fn check_source(registry: &SourceRegistry, identity: &PackageIdentity) -> Result<(), TrustReject> {
    // Scratch metadata is never registered as an anchor or accepted as authority.
    let blank = SourceRecord {
        id: SourceId::from_bytes([0; 32]),
        anchors: TrustAnchors {
            ed25519: [0; 32],
            mldsa65: [0; 1952],
        },
        generation: 0,
        state: SourceStatus::Revoked,
    };
    let mut records = [blank; MAX_SOURCES];
    let count = registry
        .export_snapshot(&mut records)
        .map_err(|_| TrustReject::UnknownSource)?;
    let source = records[..count]
        .iter()
        .find(|source| source.id.as_bytes() == identity.source_id())
        .ok_or(TrustReject::UnknownSource)?;
    if source.state == SourceStatus::Revoked {
        return Err(TrustReject::RevokedSource);
    }
    if source.generation != identity.source_generation() {
        return Err(TrustReject::StaleSourceGeneration);
    }
    Ok(())
}
