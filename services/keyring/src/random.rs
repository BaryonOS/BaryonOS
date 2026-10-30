//! RANDOM-only policy around the maintained RustCrypto ChaCha20Rng.
//! Raw bytes are obtained only through the owning Service's EntropySource and
//! continuous health tests. No caller can seed, serialize or seek this RNG.
use crate::{EntropySource, HealthFailure, Service, State, Status, RESEED_ENTROPY};
use chacha20_rng::ChaCha20Rng;
use hkdf_protected::Hkdf;
use rand_core_rng::{Rng, SeedableRng};
use sha2_kdf::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

#[cfg(all(target_os = "none", not(chacha20_backend = "soft")))]
compile_error!("Key Service RNG requires --cfg chacha20_backend=\"soft\"; no SIMD fallback");

pub const VERSION: u16 = 1;
pub const MAX_BYTES: usize = 224;
pub const DOMAINS: usize = 4;
pub const WINDOW_TICKS: u64 = 100;
pub const WINDOW_BYTES: u32 = 8192;
pub const WINDOW_CALLS: u32 = 64;
pub const RESEED_BYTES: u32 = 4096;
pub const RESEED_CALLS: u32 = 64;
const SALT: &[u8] = b"nieuw-os random entropy v1";
const LABEL: &[u8] = b"nieuw-os random domain v1";

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    pub calls: u64,
    pub bytes: u64,
    pub seeds: u64,
}

struct Domain {
    generator: Option<ChaCha20Rng>,
    epoch: u64,
    epoch_calls: u32,
    epoch_bytes: u32,
    window_end: u64,
    window_calls: u32,
    window_bytes: u32,
    last_request: u64,
}
impl Domain {
    const fn new() -> Self {
        Self {
            generator: None,
            epoch: 0,
            epoch_calls: 0,
            epoch_bytes: 0,
            window_end: 0,
            window_calls: 0,
            window_bytes: 0,
            last_request: 0,
        }
    }
}

pub(super) struct Pool {
    domains: [Domain; DOMAINS],
    last_tick: Option<u64>,
    counters: Counters,
}
impl Pool {
    pub(super) const fn new() -> Self {
        Self {
            domains: [const { Domain::new() }; DOMAINS],
            last_tick: None,
            counters: Counters {
                calls: 0,
                bytes: 0,
                seeds: 0,
            },
        }
    }
    pub(super) fn clear_generators(&mut self) {
        // Both the generator key/core and its buffered blocks have upstream
        // zeroizing Drop implementations when the reviewed feature is enabled.
        fn require_zeroizing<T: ZeroizeOnDrop>() {}
        require_zeroizing::<ChaCha20Rng>();
        for domain in &mut self.domains {
            domain.generator = None;
        }
    }
}

impl Service {
    pub fn random_counters(&self) -> Counters {
        self.random.counters
    }

    /// `domain` and `now` are trusted adapter inputs, never request fields.
    /// The caller owns an explicitly granted RANDOM-only ingress. On every
    /// failure `out` stays unchanged. Quota/format/clock failures do not consume
    /// entropy or advance an RNG; a failing required seed deliberately degrades
    /// the service and wipes RNG caches, while retaining existing master/keys.
    pub fn random_at(
        &mut self,
        domain: usize,
        request: u64,
        length: usize,
        now: u64,
        source: &mut impl EntropySource,
        out: &mut [u8; MAX_BYTES],
    ) -> Result<usize, Status> {
        if domain >= DOMAINS || request == 0 || !(1..=MAX_BYTES).contains(&length) {
            return Err(Status::Invalid);
        }
        match self.state {
            State::Ready => {}
            State::Starting | State::Blocked => return Err(Status::NotReady),
            State::Degraded => return Err(Status::Degraded),
        }
        let window_end = now.checked_add(WINDOW_TICKS).ok_or(Status::Invalid)?;
        if self.random.last_tick.is_some_and(|last| now < last) {
            return Err(Status::Invalid);
        }
        let current = &self.random.domains[domain];
        if request <= current.last_request {
            return Err(Status::Invalid);
        }
        let reset_window = now >= current.window_end;
        let calls = if reset_window {
            0
        } else {
            current.window_calls
        };
        let bytes = if reset_window {
            0
        } else {
            current.window_bytes
        };
        let calls = calls.checked_add(1).ok_or(Status::Quota)?;
        let bytes = bytes.checked_add(length as u32).ok_or(Status::Quota)?;
        if calls > WINDOW_CALLS || bytes > WINDOW_BYTES {
            return Err(Status::Quota);
        }
        let reseed = current.generator.is_none()
            || current.epoch_calls >= RESEED_CALLS
            || current
                .epoch_bytes
                .checked_add(length as u32)
                .ok_or(Status::Quota)?
                > RESEED_BYTES;
        let next_calls = self
            .random
            .counters
            .calls
            .checked_add(1)
            .ok_or(Status::Quota)?;
        let next_bytes = self
            .random
            .counters
            .bytes
            .checked_add(length as u64)
            .ok_or(Status::Quota)?;
        let next_seed = if reseed {
            self.random
                .counters
                .seeds
                .checked_add(1)
                .ok_or(Status::Quota)?
        } else {
            self.random.counters.seeds
        };
        let epoch = if reseed {
            current.epoch.checked_add(1).ok_or(Status::Quota)?
        } else {
            current.epoch
        };
        let entropy_bytes = if reseed {
            self.counters
                .entropy_bytes
                .checked_add(RESEED_ENTROPY as u64)
                .ok_or(Status::Quota)?
        } else {
            self.counters.entropy_bytes
        };
        if reseed {
            let mut raw = Zeroizing::new([0u8; RESEED_ENTROPY]);
            let outcome = source
                .read(raw.as_mut())
                .map_err(|_| HealthFailure::Source)
                .and_then(|_| self.health.feed(raw.as_ref()));
            if let Err(reason) = outcome {
                self.state = State::Degraded;
                self.reason = reason;
                self.random.clear_generators();
                return Err(Status::Degraded);
            }
            // HKDF is only seed extraction/domain separation, not a new random
            // generator. The monotone epoch also separates repeated raw blocks
            // within this live instance; it is not a cold rollback anchor.
            let extract = Hkdf::<Sha256>::new(Some(SALT), raw.as_ref());
            let mut info = [0u8; LABEL.len() + 1 + 8];
            info[..LABEL.len()].copy_from_slice(LABEL);
            info[LABEL.len()] = domain as u8;
            info[LABEL.len() + 1..].copy_from_slice(&epoch.to_le_bytes());
            let mut seed = Zeroizing::new([0u8; 32]);
            extract
                .expand(&info, seed.as_mut())
                .expect("bounded HKDF seed");
            let generator = ChaCha20Rng::from_seed(*seed);
            seed.zeroize();
            let current = &mut self.random.domains[domain];
            current.generator = Some(generator); // drops/zeroizes the prior cache
            current.epoch = epoch;
            current.epoch_calls = 0;
            current.epoch_bytes = 0;
            self.counters.entropy_bytes = entropy_bytes;
        }
        let current = &mut self.random.domains[domain];
        let mut bytes_out = Zeroizing::new([0u8; MAX_BYTES]);
        current
            .generator
            .as_mut()
            .expect("seeded above")
            .fill_bytes(&mut bytes_out[..length]);
        out[..length].copy_from_slice(&bytes_out[..length]);
        current.epoch_calls += 1;
        current.epoch_bytes += length as u32;
        current.window_calls = calls;
        current.window_bytes = bytes;
        if reset_window {
            current.window_end = window_end;
        }
        current.last_request = request;
        self.random.last_tick = Some(now);
        self.random.counters = Counters {
            calls: next_calls,
            bytes: next_bytes,
            seeds: next_seed,
        };
        Ok(length)
    }
}

/// Erase current owned protocol buffers. This does not claim to erase every
/// compiler/register/transport copy or any bytes already given to a caller.
pub fn clear_bytes(bytes: &mut [u8]) {
    bytes.zeroize();
}
