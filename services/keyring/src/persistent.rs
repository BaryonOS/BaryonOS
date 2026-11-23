//! Versioned encrypted key-state snapshots for B02.
//!
//! This module is a codec, not a disk writer. The native storage service must
//! durably commit the returned blob before acknowledging key changes. An
//! independently trusted minimum generation can reject an old snapshot; the
//! blob and generation on the same replayable disk are not such an anchor.

use chacha20poly1305::{
    aead::{inout::InOutBuf, AeadInOut, KeyInit},
    Tag, XChaCha20Poly1305, XNonce,
};
use hkdf_protected::Hkdf;
use sha2_kdf::Sha256;
use zeroize::Zeroizing;

use crate::{
    Domain, EntropySource, Service, Status, DOMAINS, KEYS_PER_DOMAIN, KEY_BYTES, NONCE_BYTES,
    SECRET_BYTES, TAG_BYTES,
};

pub const PERSISTENT_VERSION: u16 = 1;
pub const PERSISTENT_SUITE: u16 = 1;
pub const PERSISTENT_HEADER: usize = 32;
pub const PERSISTENT_PLAIN: usize = KEY_BYTES + 8 + DOMAINS * 4;
pub const PERSISTENT_BLOB: usize = PERSISTENT_HEADER + NONCE_BYTES + PERSISTENT_PLAIN + TAG_BYTES;
const WRAP_INFO: &[u8] = b"nieuw-os persistent key-state v1";

/// What a storage adapter reports for one durable key-state write.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StoreError {
    /// Certainly not written: durable state is unchanged.
    Failed,
    /// The write may or may not be durable. Resolve by reading durable state
    /// in a fresh instance, never by replaying the write.
    Unknown,
}

/// Crash-consistent volume transaction for one key-state snapshot. `Ok` means
/// the blob of `generation` is durable; the service confirms nothing earlier.
///
/// A store has no identity the service can check, so the service never
/// confirms a durable operation from an earlier commit: every confirmation is
/// made by a successful commit to the store passed in that same call.
pub trait KeyStateStore {
    fn commit(&mut self, generation: u64, blob: &[u8; PERSISTENT_BLOB]) -> Result<(), StoreError>;
}

/// Failure of a write-ahead operation. Carries no key, secret or plaintext.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WriteAheadError {
    /// Rejected by the key service before or without a durable write.
    Key(Status),
    /// The store certainly did not commit; an explicit retry is allowed.
    Failed,
    /// Durability is uncertain; this instance refuses further durable work.
    Unknown,
}

/// Nonsecret namespace state; no key material can be obtained from this value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SnapshotInfo {
    pub generation: u64,
    pub created: [u16; DOMAINS],
    pub revoked: [u16; DOMAINS],
}

fn header(volume: &[u8; 16]) -> [u8; PERSISTENT_HEADER] {
    let mut out = [0u8; PERSISTENT_HEADER];
    out[..8].copy_from_slice(b"NOSKEY01");
    out[8..10].copy_from_slice(&PERSISTENT_VERSION.to_le_bytes());
    out[10..12].copy_from_slice(&PERSISTENT_SUITE.to_le_bytes());
    out[12..28].copy_from_slice(volume);
    out
}

fn wrapping_key(secret: &[u8; SECRET_BYTES]) -> Zeroizing<[u8; KEY_BYTES]> {
    let hk = Hkdf::<Sha256>::new(Some(WRAP_INFO), secret);
    let mut key = Zeroizing::new([0u8; KEY_BYTES]);
    hk.expand(b"wrap", &mut *key)
        .expect("32 bytes is a valid HKDF output length");
    key
}

fn allocated_mask(created: u16) -> u16 {
    if usize::from(created) == KEYS_PER_DOMAIN {
        u16::MAX
    } else {
        (1u16 << created) - 1
    }
}

impl Service {
    pub fn snapshot_info(&self) -> SnapshotInfo {
        SnapshotInfo {
            generation: self.key_generation,
            created: core::array::from_fn(|i| self.domains[i].created),
            revoked: core::array::from_fn(|i| self.domains[i].revoked),
        }
    }

    /// Seal the exact allocation and revocation state under a caller-held
    /// random secret. This is separate from legacy B01 recovery, which marks
    /// every index allocated on restore. The service retains no copy of the
    /// caller's secret. Export binds the live instance to this volume.
    ///
    /// Export is allowed in write-ahead mode, including while allocations or
    /// revocations are pending after `Failed` and after the uncertain latch is
    /// set by `Unknown`. It changes no namespace state and the live state is a
    /// monotonic superset of every committed snapshot, so such a blob can never
    /// make an index issuable again. It is, however, live state and not the
    /// committed state: it must not be treated as, or persisted in place of,
    /// the durable snapshot, because that is the replay `StoreError::Unknown`
    /// forbids. Recovery uses a fresh instance restoring the on-disk blob.
    pub fn export_persistent(
        &mut self,
        volume: &[u8; 16],
        secret: &[u8; SECRET_BYTES],
        source: &mut impl EntropySource,
        out: &mut [u8; PERSISTENT_BLOB],
    ) -> Result<SnapshotInfo, Status> {
        if self.persistent_volume.is_some_and(|known| known != *volume) {
            return Err(Status::Invalid);
        }
        self.account(source)?;
        let mut plain = Zeroizing::new([0u8; PERSISTENT_PLAIN]);
        plain[..KEY_BYTES].copy_from_slice(&self.master);
        plain[KEY_BYTES..KEY_BYTES + 8].copy_from_slice(&self.key_generation.to_le_bytes());
        for (i, domain) in self.domains.iter().enumerate() {
            let offset = KEY_BYTES + 8 + i * 4;
            plain[offset..offset + 2].copy_from_slice(&domain.created.to_le_bytes());
            plain[offset + 2..offset + 4].copy_from_slice(&domain.revoked.to_le_bytes());
        }
        let wrap = wrapping_key(secret);
        let cipher = XChaCha20Poly1305::new((&*wrap).into());
        let aad = header(volume);
        let mut nonce = [0u8; NONCE_BYTES];
        self.drbg.as_mut().ok_or(Status::NotReady)?.fill(&mut nonce);
        let tag = cipher
            .encrypt_inout_detached((&nonce).into(), &aad, InOutBuf::from(&mut plain[..]))
            .map_err(|_| Status::Invalid)?;
        let body = PERSISTENT_HEADER + NONCE_BYTES;
        let end = body + PERSISTENT_PLAIN;
        out[..PERSISTENT_HEADER].copy_from_slice(&aad);
        out[PERSISTENT_HEADER..body].copy_from_slice(&nonce);
        out[body..end].copy_from_slice(&*plain);
        out[end..].copy_from_slice(&tag);
        self.persistent_volume = Some(*volume);
        Ok(self.snapshot_info())
    }

    /// Write-ahead GENERATE (B02 item 4): the index is allocated, the snapshot
    /// recording it is committed through `store`, and only then is the index
    /// returned. Until that commit returns `Ok` the index is pending: it cannot
    /// seal or unseal, and it is never reissued by this instance. The first
    /// call switches the instance to write-ahead mode.
    ///
    /// A `Failed` commit consumes the index: the caller never receives it and
    /// there is no API to retry that allocation. While it is pending the gate
    /// in `Service::key` refuses every seal and unseal under it, so no
    /// ciphertext is created under it before the next successful commit. That
    /// later commit records it as allocated but unreturned (it is never
    /// reissued) and lifts the pending gate; from then on it is an ordinary
    /// allocated index that no caller was handed. It counts against the
    /// domain quota of `KEYS_PER_DOMAIN`.
    pub fn generate_durable(
        &mut self,
        domain: usize,
        volume: &[u8; 16],
        secret: &[u8; SECRET_BYTES],
        store: &mut impl KeyStateStore,
        source: &mut impl EntropySource,
    ) -> Result<u8, WriteAheadError> {
        self.check_write_ahead(volume)?;
        let index = self
            .allocate(domain, source)
            .map_err(WriteAheadError::Key)?;
        self.write_ahead = true;
        self.pending[domain] |= 1 << index;
        self.commit_durable(volume, secret, store, source)?;
        Ok(index)
    }

    /// Write-ahead REVOKE: the key is unusable at once, but the revocation is
    /// confirmed only after the snapshot recording it is committed. After a
    /// certain `Failed`, calling this again for the same index retries the
    /// commit explicitly.
    ///
    /// `Ok` means that the `store` passed in this call durably holds the
    /// revocation: every confirmation is made by a successful commit to that
    /// store, never by an earlier commit to any store. For an index that is
    /// already revoked while the instance is settled, the live state is
    /// committed again through `store` at the next generation, and the result
    /// of that commit is returned (`Unknown` latches as elsewhere). It never
    /// returns `Ok` for an already-revoked index while any allocation or
    /// revocation is pending, before the first write-ahead commit, or once the
    /// instance is uncertain.
    pub fn revoke_durable(
        &mut self,
        domain: usize,
        index: u8,
        volume: &[u8; 16],
        secret: &[u8; SECRET_BYTES],
        store: &mut impl KeyStateStore,
        source: &mut impl EntropySource,
    ) -> Result<(), WriteAheadError> {
        self.check_write_ahead(volume)?;
        let retry = usize::from(index) < KEYS_PER_DOMAIN
            && self
                .revoke_pending
                .get(domain)
                .is_some_and(|bits| bits & (1 << index) != 0);
        if !retry {
            match self.revoke_entry(domain, index) {
                Ok(()) => {
                    self.write_ahead = true;
                    self.revoke_pending[domain] |= 1 << index;
                }
                // Already revoked and settled: the earlier `Ok` may have come
                // from another store, and the trait carries no store identity,
                // so re-commit the live state through this `store` at a fresh
                // generation instead of confirming without a store call.
                Err(Status::Revoked) if self.write_ahead_settled() => {
                    self.key_generation = self
                        .key_generation
                        .checked_add(1)
                        .ok_or(WriteAheadError::Key(Status::Quota))?;
                }
                Err(status) => return Err(WriteAheadError::Key(status)),
            }
        }
        self.commit_durable(volume, secret, store, source)?;
        Ok(())
    }

    /// True after a commit returned `Unknown`; recovery is a fresh instance
    /// restoring the durable snapshot.
    pub const fn write_ahead_uncertain(&self) -> bool {
        self.uncertain
    }

    /// Write-ahead mode with nothing pending: the last commit returned `Ok`.
    fn write_ahead_settled(&self) -> bool {
        self.write_ahead
            && !self.uncertain
            && self.pending.iter().all(|&bits| bits == 0)
            && self.revoke_pending.iter().all(|&bits| bits == 0)
    }

    /// Every check that can be made before mutating: a rejected call leaves
    /// the namespace and the mode unchanged.
    fn check_write_ahead(&self, volume: &[u8; 16]) -> Result<(), WriteAheadError> {
        if self.uncertain {
            return Err(WriteAheadError::Unknown);
        }
        if self.persistent_volume.is_some_and(|known| known != *volume) {
            return Err(WriteAheadError::Key(Status::Invalid));
        }
        Ok(())
    }

    /// One store call per invocation, never repeated here. The snapshot covers
    /// all live state, so `Ok` confirms every pending allocation and revocation.
    fn commit_durable(
        &mut self,
        volume: &[u8; 16],
        secret: &[u8; SECRET_BYTES],
        store: &mut impl KeyStateStore,
        source: &mut impl EntropySource,
    ) -> Result<SnapshotInfo, WriteAheadError> {
        let mut blob = [0u8; PERSISTENT_BLOB];
        let info = self
            .export_persistent(volume, secret, source, &mut blob)
            .map_err(WriteAheadError::Key)?;
        match store.commit(info.generation, &blob) {
            Ok(()) => {
                self.pending = [0; DOMAINS];
                self.revoke_pending = [0; DOMAINS];
                Ok(info)
            }
            Err(StoreError::Failed) => Err(WriteAheadError::Failed),
            Err(StoreError::Unknown) => {
                self.uncertain = true;
                Err(WriteAheadError::Unknown)
            }
        }
    }

    /// Authenticate and fully validate a snapshot before committing its state.
    /// Live state is merged monotonically: allocations never become issuable
    /// again and revocations are unioned. Different master/volume lineages may
    /// only be imported into an unused instance. `minimum_generation` must come
    /// from an independently trusted source to protect a cold boot from replay.
    ///
    /// Refused with `Invalid` once write-ahead mode is engaged: a merged
    /// snapshot was not committed through this instance's store, so its state
    /// must not count as durable. Write-ahead recovery is a fresh instance.
    pub fn restore_persistent(
        &mut self,
        volume: &[u8; 16],
        secret: &[u8; SECRET_BYTES],
        blob: &[u8],
        minimum_generation: u64,
        source: &mut impl EntropySource,
    ) -> Result<SnapshotInfo, Status> {
        if self.write_ahead {
            return Err(Status::Invalid);
        }
        if blob.len() != PERSISTENT_BLOB {
            return Err(Status::Invalid);
        }
        let aad = header(volume);
        if blob[..PERSISTENT_HEADER] != aad {
            return Err(Status::Auth);
        }
        if self.persistent_volume.is_some_and(|known| known != *volume) {
            return Err(Status::Invalid);
        }
        let wrap = wrapping_key(secret);
        let cipher = XChaCha20Poly1305::new((&*wrap).into());
        let body = PERSISTENT_HEADER + NONCE_BYTES;
        let end = body + PERSISTENT_PLAIN;
        let mut plain = Zeroizing::new([0u8; PERSISTENT_PLAIN]);
        plain.copy_from_slice(&blob[body..end]);
        let nonce: &XNonce = blob[PERSISTENT_HEADER..body]
            .try_into()
            .map_err(|_| Status::Invalid)?;
        let tag: &Tag = blob[end..].try_into().map_err(|_| Status::Invalid)?;
        if cipher
            .decrypt_inout_detached(nonce, &aad, InOutBuf::from(&mut plain[..]), tag)
            .is_err()
        {
            self.counters.auth_failures += 1;
            return Err(Status::Auth);
        }
        let mut generation_bytes = [0u8; 8];
        generation_bytes.copy_from_slice(&plain[KEY_BYTES..KEY_BYTES + 8]);
        let recovered_generation = u64::from_le_bytes(generation_bytes);
        if recovered_generation < minimum_generation {
            return Err(Status::Auth);
        }
        let same_master = self.master == plain[..KEY_BYTES];
        if !same_master
            && (self.domains.iter().any(|domain| domain.created != 0)
                || self.persistent_volume.is_some())
        {
            return Err(Status::Invalid);
        }
        let mut merged = [Domain::default(); DOMAINS];
        let mut includes_local_state = false;
        let mut changes_local_state = !same_master;
        for (i, domain) in merged.iter_mut().enumerate() {
            let offset = KEY_BYTES + 8 + i * 4;
            let created = u16::from_le_bytes([plain[offset], plain[offset + 1]]);
            let revoked = u16::from_le_bytes([plain[offset + 2], plain[offset + 3]]);
            if usize::from(created) > KEYS_PER_DOMAIN || revoked & !allocated_mask(created) != 0 {
                return Err(Status::Invalid);
            }
            let current = if same_master {
                self.domains[i]
            } else {
                Domain::default()
            };
            domain.created = created.max(current.created);
            domain.revoked = revoked | current.revoked;
            includes_local_state |= domain.created != created || domain.revoked != revoked;
            changes_local_state |=
                domain.created != current.created || domain.revoked != current.revoked;
        }
        let mut generation = if same_master {
            recovered_generation.max(self.key_generation)
        } else {
            recovered_generation
        };
        if (includes_local_state && generation == recovered_generation)
            || (same_master && changes_local_state && generation == self.key_generation)
        {
            generation = generation.checked_add(1).ok_or(Status::Quota)?;
        }
        // All authentication, bounds, lineage and overflow checks precede the
        // entropy-accounting boundary and the indivisible namespace commit.
        self.account(source)?;
        self.master.copy_from_slice(&plain[..KEY_BYTES]);
        self.domains = merged;
        self.key_generation = generation;
        self.persistent_volume = Some(*volume);
        self.counters.restores += 1;
        Ok(self.snapshot_info())
    }
}
