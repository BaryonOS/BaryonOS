//! Cold-freshness protocol for durable network grants (ADR-NET-ISSUER-20261125,
//! decision 5B with the M1 protected-slot route; B's contract in
//! `docs/implementation/COLD-FRESHNESS-ANCHOR.md`, extend chain).
//!
//! HOST PROTOTYPE ONLY: no native anchor is wired, so 5A (every grant denied
//! after a cold boot) remains the product behaviour. The anchor is reached only
//! through [`FreshnessAnchor`], the network-domain capability that workstream B
//! owns; this module never touches a TPM. Store authentication, the store
//! digest `D(S)` and atomic store replacement (torn A1, #529) are the store
//! owner's and stay outside this module.
//!
//! The anchor is a hash chain: every acknowledged commit extends it with
//! `E(S)`, so it only moves forward and no input an attacker holds leads back
//! to an earlier store. A store records `prev`, the chain value it was
//! committed on top of, and is acknowledged only after `extend` succeeds
//! (commit then extend). A store also binds the anchor instance, so a replaced
//! or re-provisioned TPM yields `Foreign` until authorised recovery.

use sha2::{Digest as _, Sha256};

/// SHA-256 digest: store digests, chain values and the anchor instance.
pub type Digest = [u8; 32];

/// `AnchorDomain::Network` in B's domain table.
pub const NETWORK_DOMAIN: u8 = 0;

const EVENT_EXT: &[u8] = b"BARYON-ANCHOR-EXT-V1";

fn sha(parts: &[&[u8]]) -> Digest {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    hasher.finalize().into()
}

/// `E(S) = SHA-256("BARYON-ANCHOR-EXT-V1" ‖ dom ‖ D(S))`; the kernel derives it,
/// the caller never supplies it.
pub fn extend_event(store: &Digest) -> Digest {
    sha(&[EVENT_EXT, &[NETWORK_DOMAIN], store])
}

/// The chain value `NV_Extend(event)` leaves on an index holding `prev`.
pub fn chain_step(prev: &Digest, event: &Digest) -> Digest {
    sha(&[prev, event])
}

/// Chain value once store digest `store` is anchored on top of `prev`.
pub fn chain_after(prev: &Digest, store: &Digest) -> Digest {
    chain_step(prev, &extend_event(store))
}

/// Errors of the kernel anchor service; none of them is ever an acceptance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorError {
    /// Degraded boot, TPM absent or faulty, index missing, never extended or
    /// with a changed public area.
    Unavailable,
    /// The read-back after an extend differs from the expected chain value.
    Corrupt,
    /// The chain differs from the caller's expected value; nothing written.
    Conflict,
    /// The per-domain extend budget of this boot is spent; nothing written.
    RateLimited,
}

/// Bounded network-domain capability; no other anchor domain is reachable.
pub trait FreshnessAnchor {
    /// The current chain value.
    fn read(&mut self) -> Result<Digest, AnchorError>;
    /// Compare-and-extend: if the chain equals `expected`, extend it with
    /// `E(store)`. A lost or failed response is never retried; the next
    /// `read` decides.
    fn extend(&mut self, expected: &Digest, store: &Digest) -> Result<(), AnchorError>;
    /// Identity of this anchor instance; it changes when B re-provisions the
    /// domain, e.g. after authorised recovery from a TPM replacement (C665).
    fn instance(&mut self) -> Result<Digest, AnchorError>;
}

/// An authenticated store state as loaded from durable storage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreState {
    /// Anchor instance this store belongs to.
    pub instance: Digest,
    /// Chain value this state was committed on top of.
    pub prev: Digest,
    /// `D(S)`: digest of the exact authenticated grants and tombstones.
    pub digest: Digest,
}

impl StoreState {
    /// Chain value once this state is acknowledged.
    pub fn anchored(&self) -> Digest {
        chain_after(&self.prev, &self.digest)
    }
}

/// Why durable grants are unavailable; the caller then applies 5A (all denied).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Denied {
    /// The anchor failed or refused; fail closed (5A).
    Anchor(AnchorError),
    /// The store belongs to another anchor instance (replaced or reset TPM);
    /// only authorised recovery may start a new genesis store.
    Foreign,
    /// The chain is neither this store's anchored value nor its `prev`:
    /// an older, sibling or foreign-history store, indistinguishable here.
    Stale,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Startup {
    /// The store matches the chain.
    Accept,
    /// The store was committed before a crash and never acknowledged; the
    /// chain was extended to it now.
    RolledForward,
}

fn same_instance(anchor: &mut impl FreshnessAnchor, store: &StoreState) -> Result<(), Denied> {
    if anchor.instance().map_err(Denied::Anchor)? == store.instance {
        Ok(())
    } else {
        Err(Denied::Foreign)
    }
}

/// Decides whether a loaded store may be used after a cold boot.
pub fn startup(anchor: &mut impl FreshnessAnchor, store: &StoreState) -> Result<Startup, Denied> {
    same_instance(anchor, store)?;
    let chain = anchor.read().map_err(Denied::Anchor)?;
    if chain == store.anchored() {
        return Ok(Startup::Accept);
    }
    if chain != store.prev {
        return Err(Denied::Stale);
    }
    anchor
        .extend(&store.prev, &store.digest)
        .map_err(Denied::Anchor)?;
    Ok(Startup::RolledForward)
}

/// Called after `next` is durably committed on top of `next.prev`; success is
/// the acknowledgement. One extend, never retried.
pub fn acknowledge(anchor: &mut impl FreshnessAnchor, next: &StoreState) -> Result<(), Denied> {
    same_instance(anchor, next)?;
    anchor
        .extend(&next.prev, &next.digest)
        .map_err(Denied::Anchor)
}

#[cfg(test)]
mod tests {
    use super::*;

    const EVENT_GENESIS: &[u8] = b"BARYON-ANCHOR-GENESIS-V1";
    const EVENT_INSTANCE: &[u8] = b"BARYON-ANCHOR-INSTANCE-V1";

    fn instance_of(enroll: &Digest, nonce: &Digest) -> Digest {
        sha(&[EVENT_INSTANCE, enroll, &[NETWORK_DOMAIN], nonce])
    }

    fn genesis_event(instance: &Digest) -> Digest {
        sha(&[EVENT_GENESIS, &[NETWORK_DOMAIN], instance])
    }

    /// First extend of a freshly defined index starts from zero (B's U21).
    fn genesis_chain(instance: &Digest) -> Digest {
        chain_step(&[0; 32], &genesis_event(instance))
    }

    /// Kernel anchor service for the network domain over a modelled
    /// `TPM_NT_EXTEND` index, serialised per domain (C87).
    struct Fake {
        /// `None`: index missing, or defined but never extended.
        chain: Option<Digest>,
        instance: Digest,
        budget: u32,
        fail: Option<AnchorError>,
        lose_next_response: bool,
        corrupt_next_write: bool,
        writes: u32,
    }

    impl Fake {
        /// TEST provisioning: define, then the single genesis extend.
        fn provisioned(instance: Digest) -> Self {
            Self {
                chain: Some(genesis_chain(&instance)),
                instance,
                budget: 16,
                fail: None,
                lose_next_response: false,
                corrupt_next_write: false,
                writes: 0,
            }
        }

        /// T3: a raw `NV_Extend` of attacker-chosen data, outside the capability.
        fn hostile_extend(&mut self, data: &Digest) {
            let chain = self.chain.expect("index");
            self.chain = Some(chain_step(&chain, data));
        }
    }

    impl FreshnessAnchor for Fake {
        fn read(&mut self) -> Result<Digest, AnchorError> {
            if let Some(error) = self.fail {
                return Err(error);
            }
            self.chain.ok_or(AnchorError::Unavailable)
        }

        fn extend(&mut self, expected: &Digest, store: &Digest) -> Result<(), AnchorError> {
            let chain = self.read()?;
            if chain != *expected {
                return Err(AnchorError::Conflict);
            }
            if self.budget == 0 {
                return Err(AnchorError::RateLimited);
            }
            self.budget -= 1;
            let mut event = extend_event(store);
            if core::mem::take(&mut self.corrupt_next_write) {
                event[0] ^= 1;
            }
            self.chain = Some(chain_step(&chain, &event));
            self.writes += 1;
            if core::mem::take(&mut self.lose_next_response) {
                return Err(AnchorError::Unavailable);
            }
            if self.read()? != chain_after(expected, store) {
                return Err(AnchorError::Corrupt);
            }
            Ok(())
        }

        fn instance(&mut self) -> Result<Digest, AnchorError> {
            if let Some(error) = self.fail {
                return Err(error);
            }
            Ok(self.instance)
        }
    }

    const I0: Digest = [0xA0; 32];
    const I1: Digest = [0xA1; 32];

    fn first(instance: Digest, tag: u8) -> StoreState {
        StoreState {
            instance,
            prev: genesis_chain(&instance),
            digest: [tag; 32],
        }
    }

    fn child(parent: &StoreState, tag: u8) -> StoreState {
        StoreState {
            prev: parent.anchored(),
            digest: [tag; 32],
            ..*parent
        }
    }

    /// Anchor with store `A` acknowledged on top of genesis.
    fn at_a() -> (Fake, StoreState) {
        let mut anchor = Fake::provisioned(I0);
        let a = first(I0, 1);
        assert_eq!(acknowledge(&mut anchor, &a), Ok(()));
        (anchor, a)
    }

    #[test]
    fn matching_store_is_accepted_without_a_write() {
        let (mut anchor, a) = at_a();
        let writes = anchor.writes;
        assert_eq!(startup(&mut anchor, &a), Ok(Startup::Accept));
        assert_eq!(anchor.writes, writes);
    }

    /// Review finding M1: a stale authentic store X left by a crash before the
    /// extend, a replayed A and a hostile extend must not displace the
    /// acknowledged Y. B's model test of the same name; formerly
    /// `hostile_slot_write_cannot_revive_a_stale_store_at_the_same_generation`.
    #[test]
    fn hostile_extend_cannot_revive_a_stale_store_at_the_same_position() {
        let (mut anchor, a) = at_a();
        let x = child(&a, 3); // committed to disk, crash before the extend
        // Disk attacker replays A inside the rule (c) window: accepted.
        assert_eq!(startup(&mut anchor, &a), Ok(Startup::Accept));
        let y = child(&a, 2); // carries an acknowledged revocation
        assert_eq!(acknowledge(&mut anchor, &y), Ok(()));
        assert_eq!(startup(&mut anchor, &x), Err(Denied::Stale));
        assert_eq!(startup(&mut anchor, &y), Ok(Startup::Accept));
        // Illustration (SHA-256 preimage resistance, not a test of this
        // module): T3 can only extend forward; no input it holds leads back to X.
        let now = anchor.read().unwrap();
        for data in [
            extend_event(&x.digest),
            extend_event(&a.digest),
            x.digest,
            x.prev,
            a.anchored(),
        ] {
            let chain = chain_step(&now, &data);
            assert_ne!(chain, x.anchored());
            assert_ne!(chain, x.prev);
        }
        anchor.hostile_extend(&extend_event(&x.digest));
        assert_eq!(startup(&mut anchor, &x), Err(Denied::Stale));
        // A stray extend denies every store until genesis recovery (fail closed).
        assert_eq!(startup(&mut anchor, &y), Err(Denied::Stale));
    }

    /// Formerly `protected_slot_refuses_both_multi_boot_attacks` and the two
    /// `reseal_multi_boot_*` residuals, which need no reseal under the chain.
    #[test]
    fn crashed_sibling_cannot_displace_a_rolled_forward_store() {
        let (mut anchor, a) = at_a();
        let x = child(&a, 3);
        let z = child(&a, 4);
        assert_eq!(startup(&mut anchor, &x), Ok(Startup::RolledForward));
        assert_eq!(startup(&mut anchor, &z), Err(Denied::Stale));
        assert_eq!(startup(&mut anchor, &a), Err(Denied::Stale));
        let y = child(&x, 2);
        assert_eq!(acknowledge(&mut anchor, &y), Ok(()));
        for stale in [a, x, z, child(&z, 5)] {
            assert_eq!(startup(&mut anchor, &stale), Err(Denied::Stale));
        }
        assert_eq!(startup(&mut anchor, &y), Ok(Startup::Accept));
    }

    /// Every write prefix of one commit: A1 (store write), then the extend.
    #[test]
    fn every_crash_prefix_of_a_commit() {
        // Cut before A1 (a torn A1 restores A: store requirement, #529).
        let (mut anchor, a) = at_a();
        assert_eq!(startup(&mut anchor, &a), Ok(Startup::Accept));
        // Cut after A1: X rolls forward exactly once.
        let x = child(&a, 3);
        assert_eq!(startup(&mut anchor, &x), Ok(Startup::RolledForward));
        assert_eq!(startup(&mut anchor, &x), Ok(Startup::Accept));
        assert_eq!(
            anchor.extend(&x.prev, &x.digest),
            Err(AnchorError::Conflict)
        );
        // Cut after the extend: accept.
        let (mut anchor, a) = at_a();
        let x = child(&a, 3);
        assert_eq!(acknowledge(&mut anchor, &x), Ok(()));
        assert_eq!(startup(&mut anchor, &x), Ok(Startup::Accept));
    }

    #[test]
    fn lost_response_is_settled_by_the_next_read_never_by_a_blind_retry() {
        let (mut anchor, a) = at_a();
        let y = child(&a, 2);
        anchor.lose_next_response = true;
        assert_eq!(
            acknowledge(&mut anchor, &y),
            Err(Denied::Anchor(AnchorError::Unavailable))
        );
        // The compare step turns a retry into Conflict, not a second extend.
        assert_eq!(
            acknowledge(&mut anchor, &y),
            Err(Denied::Anchor(AnchorError::Conflict))
        );
        assert_eq!(startup(&mut anchor, &y), Ok(Startup::Accept));
        // A blind second extend would move past every store: permanent denial.
        anchor.hostile_extend(&extend_event(&y.digest));
        assert_eq!(startup(&mut anchor, &y), Err(Denied::Stale));
    }

    #[test]
    fn spent_budget_writes_nothing() {
        let (mut anchor, a) = at_a();
        let before = anchor.read().unwrap();
        anchor.budget = 0;
        let y = child(&a, 2);
        assert_eq!(
            acknowledge(&mut anchor, &y),
            Err(Denied::Anchor(AnchorError::RateLimited))
        );
        assert_eq!(
            startup(&mut anchor, &y),
            Err(Denied::Anchor(AnchorError::RateLimited))
        );
        assert_eq!(anchor.read(), Ok(before));
        anchor.budget = 1; // next boot
        assert_eq!(acknowledge(&mut anchor, &y), Ok(()));
    }

    #[test]
    fn anchor_failures_deny_and_never_acknowledge() {
        // Index missing, or defined but never extended: never a genesis.
        let mut anchor = Fake::provisioned(I0);
        anchor.chain = None;
        let a = first(I0, 1);
        assert_eq!(
            startup(&mut anchor, &a),
            Err(Denied::Anchor(AnchorError::Unavailable))
        );
        assert_eq!(
            acknowledge(&mut anchor, &a),
            Err(Denied::Anchor(AnchorError::Unavailable))
        );
        for error in [AnchorError::Unavailable, AnchorError::Corrupt] {
            let (mut anchor, a) = at_a();
            anchor.fail = Some(error);
            assert_eq!(startup(&mut anchor, &a), Err(Denied::Anchor(error)));
            assert_eq!(
                acknowledge(&mut anchor, &child(&a, 2)),
                Err(Denied::Anchor(error))
            );
        }
        // Read-back mismatch after the extend.
        let (mut anchor, a) = at_a();
        anchor.corrupt_next_write = true;
        let y = child(&a, 2);
        assert_eq!(
            acknowledge(&mut anchor, &y),
            Err(Denied::Anchor(AnchorError::Corrupt))
        );
        assert_eq!(startup(&mut anchor, &y), Err(Denied::Stale));
        // A commit on a stale `prev` is a Conflict and writes nothing.
        let (mut anchor, a) = at_a();
        let writes = anchor.writes;
        assert_eq!(
            acknowledge(&mut anchor, &first(I0, 9)),
            Err(Denied::Anchor(AnchorError::Conflict))
        );
        assert_eq!(startup(&mut anchor, &a), Ok(Startup::Accept));
        assert_eq!(anchor.writes, writes);
    }

    #[test]
    fn failed_roll_forward_is_a_denial() {
        let (mut anchor, a) = at_a();
        let x = child(&a, 3);
        anchor.corrupt_next_write = true;
        assert_eq!(
            startup(&mut anchor, &x),
            Err(Denied::Anchor(AnchorError::Corrupt))
        );
        assert_eq!(startup(&mut anchor, &x), Err(Denied::Stale));
        let (mut anchor, a) = at_a();
        let x = child(&a, 3);
        anchor.lose_next_response = true;
        assert_eq!(
            startup(&mut anchor, &x),
            Err(Denied::Anchor(AnchorError::Unavailable))
        );
        // The extend landed: the next boot accepts without a second write.
        let writes = anchor.writes;
        assert_eq!(startup(&mut anchor, &x), Ok(Startup::Accept));
        assert_eq!(anchor.writes, writes);
    }

    #[test]
    fn replaced_tpm_rejects_the_store_until_authorised_genesis() {
        let (_, a) = at_a();
        let store = child(&a, 2);
        // Authorised recovery provisions the new TPM with a new instance.
        let mut anchor = Fake::provisioned(I1);
        assert_eq!(startup(&mut anchor, &store), Err(Denied::Foreign));
        assert_eq!(acknowledge(&mut anchor, &store), Err(Denied::Foreign));
        assert_eq!(anchor.writes, 0);
        // The same digest under the new instance cannot ride the old chain.
        let moved = StoreState {
            instance: I1,
            ..store
        };
        assert_eq!(startup(&mut anchor, &moved), Err(Denied::Stale));
        // A fresh genesis store carries nothing of the old grants or tombstones.
        let genesis = first(I1, 7);
        assert_eq!(startup(&mut anchor, &genesis), Ok(Startup::RolledForward));
        assert_eq!(startup(&mut anchor, &genesis), Ok(Startup::Accept));
    }

    fn unhex(text: &str) -> Digest {
        let bytes = text.as_bytes();
        assert_eq!(bytes.len(), 64);
        let nibble = |c: u8| match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            _ => panic!("hex"),
        };
        let mut out = [0; 32];
        for (i, byte) in out.iter_mut().enumerate() {
            *byte = nibble(bytes[2 * i]) << 4 | nibble(bytes[2 * i + 1]);
        }
        out
    }

    /// B's golden vectors (ENROLL 11³², nonce 22³², domain 0, D(S) 33³²),
    /// reproduced through the fake's compare-and-extend.
    #[test]
    fn golden_vectors() {
        let instance = instance_of(&[0x11; 32], &[0x22; 32]);
        let store = [0x33; 32];
        assert_eq!(
            instance,
            unhex("bfe1bb14b6e07bf6c176d752ebaa1425c915d3b9e54b567fadbe2d3bd6b15a71")
        );
        assert_eq!(
            genesis_event(&instance),
            unhex("695716ce7f9f7ca92945c51f3425cfbd4d88ed4976dad3a09a1717c015247cf9")
        );
        let mut anchor = Fake::provisioned(instance);
        assert_eq!(
            anchor.read(),
            Ok(unhex(
                "f5f47a2563aae97e4a3b116dbdf449bb78b89889475fa9a2c6995f9183f2261f"
            ))
        );
        assert_eq!(
            extend_event(&store),
            unhex("a0e8380d73d61df74dc648499898091350f03a83e59de072a234188679bb011c")
        );
        let s = StoreState {
            instance,
            prev: anchor.read().unwrap(),
            digest: store,
        };
        assert_eq!(acknowledge(&mut anchor, &s), Ok(()));
        assert_eq!(
            anchor.read(),
            Ok(unhex(
                "ba929ee819c99d8f0fb01bcec7e0ff999f4e8b3d651079ad127add6b75cd7d96"
            ))
        );
        assert_eq!(startup(&mut anchor, &s), Ok(Startup::Accept));
    }
}
