//! Versioned encrypted key-state snapshots for B02.
//!
//! This module is a codec, not a disk writer. The native storage service must
//! durably commit the returned blob before acknowledging key changes. An
//! independently trusted minimum generation can reject an old snapshot; the
//! blob and generation on the same replayable disk are not such an anchor.

use chacha20poly1305::{
    aead::{AeadInPlace, KeyInit},
    Key, Tag, XChaCha20Poly1305, XNonce,
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
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&*wrap));
        let aad = header(volume);
        let mut nonce = [0u8; NONCE_BYTES];
        self.drbg.as_mut().ok_or(Status::NotReady)?.fill(&mut nonce);
        let tag = cipher
            .encrypt_in_place_detached(XNonce::from_slice(&nonce), &aad, &mut *plain)
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

    /// Authenticate and fully validate a snapshot before committing its state.
    /// Live state is merged monotonically: allocations never become issuable
    /// again and revocations are unioned. Different master/volume lineages may
    /// only be imported into an unused instance. `minimum_generation` must come
    /// from an independently trusted source to protect a cold boot from replay.
    pub fn restore_persistent(
        &mut self,
        volume: &[u8; 16],
        secret: &[u8; SECRET_BYTES],
        blob: &[u8],
        minimum_generation: u64,
        source: &mut impl EntropySource,
    ) -> Result<SnapshotInfo, Status> {
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
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&*wrap));
        let body = PERSISTENT_HEADER + NONCE_BYTES;
        let end = body + PERSISTENT_PLAIN;
        let mut plain = Zeroizing::new([0u8; PERSISTENT_PLAIN]);
        plain.copy_from_slice(&blob[body..end]);
        if cipher
            .decrypt_in_place_detached(
                XNonce::from_slice(&blob[PERSISTENT_HEADER..body]),
                &aad,
                &mut *plain,
                Tag::from_slice(&blob[end..]),
            )
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
