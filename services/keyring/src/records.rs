//! Context-bound record encryption for the B02 storage service.
//!
//! The caller must obtain `domain` from its authenticated ingress endpoint and
//! the expected context from an authenticated volume manifest, never from the
//! untrusted record itself. AAD is not stored as plaintext in the sealed body.
//! A current revision must be anchored separately: this codec cannot detect a
//! whole-volume rollback when both ciphertext and expected context are replayed.

use chacha20poly1305::{
    aead::{AeadInPlace, KeyInit},
    Key, Tag, XChaCha20Poly1305, XNonce,
};
use zeroize::Zeroizing;

use crate::{
    EntropySource, Service, Status, DOMAINS, MAX_PLAINTEXT, MAX_SEALED, NONCE_BYTES, TAG_BYTES,
};

pub const RECORD_VERSION: u16 = 1;
pub const RECORD_SUITE: u16 = 1;
const AAD_BYTES: usize = 64;

/// Disjoint authenticated namespaces for content and sensitive metadata.
/// A native adapter must select the kind for the authorized operation, not
/// accept an app-provided kind as authority to read another namespace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum RecordKind {
    Content = 1,
    Name = 2,
    ObjectMetadata = 3,
    VolumeManifest = 4,
    SearchIndex = 5,
}

impl RecordKind {
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            1 => Some(Self::Content),
            2 => Some(Self::Name),
            3 => Some(Self::ObjectMetadata),
            4 => Some(Self::VolumeManifest),
            5 => Some(Self::SearchIndex),
            _ => None,
        }
    }
}

/// Object identity and exact position inside a volume. Names, tags and other
/// sensitive metadata belong in encrypted records, not in this identifier.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RecordContext {
    pub volume: [u8; 16],
    pub domain: u8,
    pub object: u64,
    pub revision: u64,
    pub chunk: u32,
    pub length: u16,
    pub kind: RecordKind,
    /// The canonical encoding version within this record kind; zero is invalid.
    pub schema: u16,
}

impl RecordContext {
    pub(crate) fn validate(self, caller_domain: usize) -> Result<(), Status> {
        if caller_domain >= DOMAINS
            || usize::from(self.domain) != caller_domain
            || usize::from(self.length) > MAX_PLAINTEXT
            || self.schema == 0
        {
            return Err(Status::Invalid);
        }
        Ok(())
    }

    fn aad(self, index: u8) -> [u8; AAD_BYTES] {
        let mut aad = [0u8; AAD_BYTES];
        aad[..8].copy_from_slice(b"NOSREC01");
        aad[8..10].copy_from_slice(&RECORD_VERSION.to_le_bytes());
        aad[10..12].copy_from_slice(&RECORD_SUITE.to_le_bytes());
        aad[12] = self.domain;
        aad[13] = index;
        aad[14] = self.kind as u8;
        // Byte 15 and bytes 56..64 are canonical reserved zeros.
        aad[16..32].copy_from_slice(&self.volume);
        aad[32..40].copy_from_slice(&self.object.to_le_bytes());
        aad[40..48].copy_from_slice(&self.revision.to_le_bytes());
        aad[48..52].copy_from_slice(&self.chunk.to_le_bytes());
        aad[52..54].copy_from_slice(&self.length.to_le_bytes());
        aad[54..56].copy_from_slice(&self.schema.to_le_bytes());
        aad
    }
}

impl Service {
    /// Seal one bounded chunk as nonce | ciphertext | tag. The output and key
    /// namespace remain unchanged on failure. Entropy failure can deliberately
    /// degrade the service, as for every other B01 operation.
    pub fn seal_record(
        &mut self,
        domain: usize,
        index: u8,
        context: RecordContext,
        plaintext: &[u8],
        source: &mut impl EntropySource,
        out: &mut [u8; MAX_SEALED],
    ) -> Result<usize, Status> {
        context.validate(domain)?;
        let length = usize::from(context.length);
        if plaintext.len() != length {
            return Err(Status::Invalid);
        }
        self.account(source)?;
        let key = Zeroizing::new(self.key(domain, index)?);
        let mut nonce = [0u8; NONCE_BYTES];
        self.drbg.as_mut().ok_or(Status::NotReady)?.fill(&mut nonce);
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&*key));
        let mut body = Zeroizing::new([0u8; MAX_PLAINTEXT]);
        body[..length].copy_from_slice(plaintext);
        let tag = cipher
            .encrypt_in_place_detached(
                XNonce::from_slice(&nonce),
                &context.aad(index),
                &mut body[..length],
            )
            .map_err(|_| Status::Invalid)?;
        let end = NONCE_BYTES + length;
        out[..NONCE_BYTES].copy_from_slice(&nonce);
        out[NONCE_BYTES..end].copy_from_slice(&body[..length]);
        out[end..end + TAG_BYTES].copy_from_slice(&tag);
        self.counters.sealed += 1;
        Ok(end + TAG_BYTES)
    }

    /// Authenticate the expected identity, revision, position and length before
    /// exposing any plaintext. All failed operations leave `out` unchanged.
    pub fn unseal_record(
        &mut self,
        domain: usize,
        index: u8,
        expected: RecordContext,
        sealed: &[u8],
        source: &mut impl EntropySource,
        out: &mut [u8; MAX_PLAINTEXT],
    ) -> Result<usize, Status> {
        expected.validate(domain)?;
        let length = usize::from(expected.length);
        if sealed.len() != NONCE_BYTES + length + TAG_BYTES {
            return Err(Status::Invalid);
        }
        self.account(source)?;
        let key = Zeroizing::new(self.key(domain, index)?);
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&*key));
        let end = NONCE_BYTES + length;
        let mut body = Zeroizing::new([0u8; MAX_PLAINTEXT]);
        body[..length].copy_from_slice(&sealed[NONCE_BYTES..end]);
        if cipher
            .decrypt_in_place_detached(
                XNonce::from_slice(&sealed[..NONCE_BYTES]),
                &expected.aad(index),
                &mut body[..length],
                Tag::from_slice(&sealed[end..]),
            )
            .is_err()
        {
            self.counters.auth_failures += 1;
            return Err(Status::Auth);
        }
        out[..length].copy_from_slice(&body[..length]);
        self.counters.unsealed += 1;
        Ok(length)
    }
}
