#![no_std]
#![forbid(unsafe_code)]
//! B01 Key Service core (ADR-043): entropy health tests, a fast-key-erasure
//! DRBG on ChaCha20, HKDF-SHA-256 key derivation, XChaCha20-Poly1305 sealing
//! with context binding, a bounded per-domain key table, and recovery of the
//! master key under a caller-held secret. No cryptographic primitive is
//! implemented here; everything comes from the pinned RustCrypto crates.
//! The kernel never sees this code, and apps never see key bytes.
use chacha20::{
    cipher::{KeyIvInit, StreamCipher},
    ChaCha20,
};
use chacha20poly1305::{
    aead::{AeadInPlace, KeyInit},
    Key, Tag, XChaCha20Poly1305, XNonce,
};
use hkdf::Hkdf;
use sha2::Sha256;
use zeroize::Zeroize;

pub mod health;
pub mod wire;

pub use health::{Health, HealthFailure};

pub const KEY_BYTES: usize = 32;
pub const NONCE_BYTES: usize = 24;
pub const TAG_BYTES: usize = 16;
pub const SECRET_BYTES: usize = 32;
/// Plaintext per SEAL: fits a 256-byte IPC payload with nonce and tag.
pub const MAX_PLAINTEXT: usize = 128;
pub const MAX_SEALED: usize = NONCE_BYTES + MAX_PLAINTEXT + TAG_BYTES;
pub const DOMAINS: usize = 2;
pub const KEYS_PER_DOMAIN: usize = 16;
/// Startup entropy: four broker requests of 256 bytes.
pub const STARTUP_ENTROPY: usize = 1024;
/// Bytes requested per reseed and the operation budget between reseeds.
pub const RESEED_ENTROPY: usize = 256;
pub const RESEED_INTERVAL: u32 = 64;
/// Recovery blob: nonce ‖ enc(master ‖ revoked bitmaps) ‖ tag.
pub const RECOVERY_PLAIN: usize = KEY_BYTES + DOMAINS * 2;
pub const RECOVERY_BLOB: usize = NONCE_BYTES + RECOVERY_PLAIN + TAG_BYTES;

const SEED_SALT: &[u8] = b"nieuw-os keyring seed v1";
const SEED_INFO: &[u8] = b"drbg";
const KEY_INFO: &[u8] = b"nieuw-os key v1";
const RECOVERY_INFO: &[u8] = b"nieuw-os recovery v1";
const SEAL_AAD: &[u8] = b"nieuw-os seal v1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Status {
    Ok = 0,
    NotReady = -1,
    Quota = -2,
    BadHandle = -3,
    Revoked = -4,
    Auth = -5,
    Invalid = -6,
    Degraded = -7,
    Entropy = -8,
}
impl Status {
    pub const fn code(self) -> i32 {
        self as i32
    }
    pub const fn from_code(code: i32) -> Self {
        match code {
            0 => Self::Ok,
            -1 => Self::NotReady,
            -2 => Self::Quota,
            -3 => Self::BadHandle,
            -4 => Self::Revoked,
            -5 => Self::Auth,
            -7 => Self::Degraded,
            -8 => Self::Entropy,
            _ => Self::Invalid,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum State {
    Starting = 0,
    Ready = 1,
    Blocked = 2,
    Degraded = 3,
}

/// Raw entropy as the kernel broker delivers it. An implementation returns
/// `Err` when the device is absent, timed out or answered short.
pub trait EntropySource {
    fn read(&mut self, out: &mut [u8]) -> Result<(), Status>;
}

/// Fast-key-erasure generator on the ChaCha20 keystream (Bernstein, 2017):
/// each call derives the next key from the keystream and wipes the old one.
pub struct Drbg {
    key: [u8; KEY_BYTES],
    generated: u32,
}
impl Drbg {
    pub fn from_entropy(raw: &[u8]) -> Self {
        let hk = Hkdf::<Sha256>::new(Some(SEED_SALT), raw);
        let mut key = [0u8; KEY_BYTES];
        hk.expand(SEED_INFO, &mut key)
            .expect("32 bytes is a valid HKDF output length");
        Self { key, generated: 0 }
    }
    /// Mix fresh raw entropy into the state without discarding it.
    pub fn reseed(&mut self, raw: &[u8]) {
        let hk = Hkdf::<Sha256>::new(Some(&self.key), raw);
        let mut key = [0u8; KEY_BYTES];
        hk.expand(SEED_INFO, &mut key)
            .expect("32 bytes is a valid HKDF output length");
        self.key.zeroize();
        self.key = key;
        self.generated = 0;
    }
    pub fn fill(&mut self, out: &mut [u8]) {
        let mut cipher = ChaCha20::new((&self.key).into(), (&[0u8; 12]).into());
        let mut next = [0u8; KEY_BYTES];
        cipher.apply_keystream(&mut next);
        for byte in out.iter_mut() {
            *byte = 0;
        }
        cipher.apply_keystream(out);
        self.key.zeroize();
        self.key = next;
        self.generated = self.generated.saturating_add(1);
    }
    pub const fn generated(&self) -> u32 {
        self.generated
    }
}
impl Drop for Drbg {
    fn drop(&mut self) {
        self.key.zeroize();
    }
}

fn derive_key(master: &[u8; KEY_BYTES], domain: u8, index: u8) -> [u8; KEY_BYTES] {
    let hk = Hkdf::<Sha256>::from_prk(master).expect("32-byte PRK");
    let mut info = [0u8; 17];
    info[..15].copy_from_slice(KEY_INFO);
    info[15] = domain;
    info[16] = index;
    let mut key = [0u8; KEY_BYTES];
    hk.expand(&info, &mut key)
        .expect("32 bytes is a valid HKDF output length");
    key
}

fn seal_aad(domain: u8, index: u8) -> [u8; 18] {
    let mut aad = [0u8; 18];
    aad[..16].copy_from_slice(SEAL_AAD);
    aad[16] = domain;
    aad[17] = index;
    aad
}

/// Per-domain key table: `created` indices exist, `revoked` indices are
/// permanently unusable and never reissued.
#[derive(Clone, Copy, Default)]
struct Domain {
    created: u16,
    revoked: u16,
}

pub struct Service {
    state: State,
    reason: HealthFailure,
    drbg: Option<Drbg>,
    master: [u8; KEY_BYTES],
    domains: [Domain; DOMAINS],
    operations: u32,
    pub counters: Counters,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    pub entropy_bytes: u64,
    pub reseeds: u64,
    pub generated: u64,
    pub sealed: u64,
    pub unsealed: u64,
    pub auth_failures: u64,
    pub revoked: u64,
    pub denied: u64,
    pub restores: u64,
}

impl Default for Service {
    fn default() -> Self {
        Self::new()
    }
}

impl Service {
    pub const fn new() -> Self {
        Self {
            state: State::Starting,
            reason: HealthFailure::None,
            drbg: None,
            master: [0; KEY_BYTES],
            domains: [Domain {
                created: 0,
                revoked: 0,
            }; DOMAINS],
            operations: 0,
            counters: Counters {
                entropy_bytes: 0,
                reseeds: 0,
                generated: 0,
                sealed: 0,
                unsealed: 0,
                auth_failures: 0,
                revoked: 0,
                denied: 0,
                restores: 0,
            },
        }
    }
    pub const fn state(&self) -> State {
        self.state
    }
    pub const fn reason(&self) -> HealthFailure {
        self.reason
    }

    /// Startup: 1024 raw bytes through the health tests, then seed the DRBG
    /// and draw a fresh master key. Any failure leaves the service Blocked
    /// with the reason recorded; no key material exists afterwards.
    pub fn start(&mut self, source: &mut impl EntropySource) -> Result<(), Status> {
        if self.state != State::Starting {
            return Err(Status::Invalid);
        }
        let mut raw = [0u8; STARTUP_ENTROPY];
        let mut health = Health::new();
        for chunk in raw.chunks_mut(RESEED_ENTROPY) {
            if source.read(chunk).is_err() {
                self.block(HealthFailure::Source);
                raw.zeroize();
                return Err(Status::Entropy);
            }
            self.counters.entropy_bytes += chunk.len() as u64;
            if let Err(failure) = health.feed(chunk) {
                self.block(failure);
                raw.zeroize();
                return Err(Status::Entropy);
            }
        }
        let mut drbg = Drbg::from_entropy(&raw);
        raw.zeroize();
        drbg.fill(&mut self.master);
        self.drbg = Some(drbg);
        self.state = State::Ready;
        Ok(())
    }

    fn block(&mut self, reason: HealthFailure) {
        self.state = State::Blocked;
        self.reason = reason;
        self.master.zeroize();
        self.drbg = None;
    }

    /// Operation budget: after `RESEED_INTERVAL` key operations the DRBG is
    /// reseeded with 256 health-tested bytes; a failing reseed degrades the
    /// service, which then refuses further key operations.
    fn account(&mut self, source: &mut impl EntropySource) -> Result<(), Status> {
        match self.state {
            State::Ready => {}
            State::Starting | State::Blocked => return Err(Status::NotReady),
            State::Degraded => return Err(Status::Degraded),
        }
        self.operations += 1;
        if self.operations < RESEED_INTERVAL {
            return Ok(());
        }
        self.operations = 0;
        let mut raw = [0u8; RESEED_ENTROPY];
        let mut health = Health::new();
        let outcome = source.read(&mut raw).map_err(|_| HealthFailure::Source);
        let outcome = outcome.and_then(|_| health.feed(&raw));
        match outcome {
            Ok(()) => {
                self.counters.entropy_bytes += raw.len() as u64;
                self.counters.reseeds += 1;
                if let Some(drbg) = self.drbg.as_mut() {
                    drbg.reseed(&raw);
                }
                raw.zeroize();
                Ok(())
            }
            Err(failure) => {
                raw.zeroize();
                self.state = State::Degraded;
                self.reason = failure;
                Err(Status::Degraded)
            }
        }
    }

    fn domain(&self, domain: usize) -> Result<Domain, Status> {
        self.domains.get(domain).copied().ok_or(Status::Invalid)
    }

    fn key(&self, domain: usize, index: u8) -> Result<[u8; KEY_BYTES], Status> {
        let table = self.domain(domain)?;
        if usize::from(index) >= KEYS_PER_DOMAIN || index >= table.created as u8 {
            return Err(Status::BadHandle);
        }
        if table.revoked & (1 << index) != 0 {
            return Err(Status::Revoked);
        }
        Ok(derive_key(&self.master, domain as u8, index))
    }

    pub fn generate(
        &mut self,
        domain: usize,
        source: &mut impl EntropySource,
    ) -> Result<u8, Status> {
        self.account(source)?;
        let table = self.domains.get_mut(domain).ok_or(Status::Invalid)?;
        if usize::from(table.created) >= KEYS_PER_DOMAIN {
            return Err(Status::Quota);
        }
        let index = table.created as u8;
        table.created += 1;
        self.counters.generated += 1;
        Ok(index)
    }

    /// `out` receives nonce ‖ ciphertext ‖ tag and the written length is returned.
    pub fn seal(
        &mut self,
        domain: usize,
        index: u8,
        plaintext: &[u8],
        source: &mut impl EntropySource,
        out: &mut [u8; MAX_SEALED],
    ) -> Result<usize, Status> {
        if plaintext.len() > MAX_PLAINTEXT {
            return Err(Status::Invalid);
        }
        self.account(source)?;
        let mut key = self.key(domain, index)?;
        let drbg = self.drbg.as_mut().ok_or(Status::NotReady)?;
        let mut nonce = [0u8; NONCE_BYTES];
        drbg.fill(&mut nonce);
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let body = &mut out[NONCE_BYTES..NONCE_BYTES + plaintext.len()];
        body.copy_from_slice(plaintext);
        let tag = cipher
            .encrypt_in_place_detached(
                XNonce::from_slice(&nonce),
                &seal_aad(domain as u8, index),
                body,
            )
            .map_err(|_| Status::Invalid)?;
        out[..NONCE_BYTES].copy_from_slice(&nonce);
        let end = NONCE_BYTES + plaintext.len();
        out[end..end + TAG_BYTES].copy_from_slice(&tag);
        key.zeroize();
        self.counters.sealed += 1;
        Ok(end + TAG_BYTES)
    }

    /// Verifies the tag before any plaintext byte becomes visible; on failure
    /// `out` is left zeroed.
    pub fn unseal(
        &mut self,
        domain: usize,
        index: u8,
        sealed: &[u8],
        source: &mut impl EntropySource,
        out: &mut [u8; MAX_PLAINTEXT],
    ) -> Result<usize, Status> {
        if sealed.len() < NONCE_BYTES + TAG_BYTES || sealed.len() > MAX_SEALED {
            return Err(Status::Invalid);
        }
        self.account(source)?;
        let mut key = self.key(domain, index)?;
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&key));
        let length = sealed.len() - NONCE_BYTES - TAG_BYTES;
        let mut body = [0u8; MAX_PLAINTEXT];
        body[..length].copy_from_slice(&sealed[NONCE_BYTES..NONCE_BYTES + length]);
        let tag = Tag::from_slice(&sealed[NONCE_BYTES + length..]);
        let result = cipher.decrypt_in_place_detached(
            XNonce::from_slice(&sealed[..NONCE_BYTES]),
            &seal_aad(domain as u8, index),
            &mut body[..length],
            tag,
        );
        key.zeroize();
        match result {
            Ok(()) => {
                out[..length].copy_from_slice(&body[..length]);
                body.zeroize();
                self.counters.unsealed += 1;
                Ok(length)
            }
            Err(_) => {
                body.zeroize();
                self.counters.auth_failures += 1;
                Err(Status::Auth)
            }
        }
    }

    pub fn revoke(&mut self, domain: usize, index: u8) -> Result<(), Status> {
        if !matches!(self.state, State::Ready | State::Degraded) {
            return Err(Status::NotReady);
        }
        let table = self.domains.get_mut(domain).ok_or(Status::Invalid)?;
        if usize::from(index) >= KEYS_PER_DOMAIN || index >= table.created as u8 {
            return Err(Status::BadHandle);
        }
        if table.revoked & (1 << index) != 0 {
            return Err(Status::Revoked);
        }
        table.revoked |= 1 << index;
        self.counters.revoked += 1;
        Ok(())
    }

    /// A fresh recovery secret for the caller and the master plus revocation
    /// state sealed under it. The service keeps neither the secret nor the blob.
    pub fn export_recovery(
        &mut self,
        source: &mut impl EntropySource,
        secret: &mut [u8; SECRET_BYTES],
        blob: &mut [u8; RECOVERY_BLOB],
    ) -> Result<(), Status> {
        self.account(source)?;
        let drbg = self.drbg.as_mut().ok_or(Status::NotReady)?;
        drbg.fill(secret);
        let mut nonce = [0u8; NONCE_BYTES];
        drbg.fill(&mut nonce);
        let mut plain = [0u8; RECOVERY_PLAIN];
        plain[..KEY_BYTES].copy_from_slice(&self.master);
        for (i, table) in self.domains.iter().enumerate() {
            plain[KEY_BYTES + i * 2..KEY_BYTES + i * 2 + 2]
                .copy_from_slice(&table.revoked.to_le_bytes());
        }
        let mut wrap = recovery_key(secret);
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&wrap));
        let body = &mut blob[NONCE_BYTES..NONCE_BYTES + RECOVERY_PLAIN];
        body.copy_from_slice(&plain);
        let tag = cipher
            .encrypt_in_place_detached(XNonce::from_slice(&nonce), RECOVERY_INFO, body)
            .map_err(|_| Status::Invalid)?;
        blob[..NONCE_BYTES].copy_from_slice(&nonce);
        blob[NONCE_BYTES + RECOVERY_PLAIN..].copy_from_slice(&tag);
        plain.zeroize();
        wrap.zeroize();
        Ok(())
    }

    /// Replace this instance's master and revocation state with the recovered
    /// ones. Keys generated in this instance before the restore are gone.
    /// The created counts become "all indices exist", so an app can address
    /// every key the exporting instance had.
    pub fn restore(
        &mut self,
        secret: &[u8; SECRET_BYTES],
        blob: &[u8; RECOVERY_BLOB],
        source: &mut impl EntropySource,
    ) -> Result<usize, Status> {
        self.account(source)?;
        let mut wrap = recovery_key(secret);
        let cipher = XChaCha20Poly1305::new(Key::from_slice(&wrap));
        let mut body = [0u8; RECOVERY_PLAIN];
        body.copy_from_slice(&blob[NONCE_BYTES..NONCE_BYTES + RECOVERY_PLAIN]);
        let result = cipher.decrypt_in_place_detached(
            XNonce::from_slice(&blob[..NONCE_BYTES]),
            RECOVERY_INFO,
            &mut body,
            Tag::from_slice(&blob[NONCE_BYTES + RECOVERY_PLAIN..]),
        );
        wrap.zeroize();
        if result.is_err() {
            body.zeroize();
            self.counters.auth_failures += 1;
            return Err(Status::Auth);
        }
        self.master.zeroize();
        self.master.copy_from_slice(&body[..KEY_BYTES]);
        for (i, table) in self.domains.iter_mut().enumerate() {
            let mut word = [0u8; 2];
            word.copy_from_slice(&body[KEY_BYTES + i * 2..KEY_BYTES + i * 2 + 2]);
            table.revoked = u16::from_le_bytes(word);
            table.created = KEYS_PER_DOMAIN as u16;
        }
        body.zeroize();
        self.counters.restores += 1;
        Ok(DOMAINS)
    }

    pub fn keys_created(&self, domain: usize) -> u16 {
        self.domains.get(domain).map_or(0, |d| d.created)
    }
}

impl Drop for Service {
    fn drop(&mut self) {
        self.master.zeroize();
    }
}

fn recovery_key(secret: &[u8; SECRET_BYTES]) -> [u8; KEY_BYTES] {
    let hk = Hkdf::<Sha256>::new(Some(RECOVERY_INFO), secret);
    let mut key = [0u8; KEY_BYTES];
    hk.expand(b"wrap", &mut key)
        .expect("32 bytes is a valid HKDF output length");
    key
}
