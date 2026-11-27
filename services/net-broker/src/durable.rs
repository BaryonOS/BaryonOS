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
    /// Run-time TPM fault or clear, or the N04 index missing, squatted, never
    /// extended or with a changed public area. A TPM absent, cleared or
    /// replaced before boot halts the loader (REL-05 §7.9), so N04 never runs.
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
    /// No slot holds an authentic store. Never treated as genesis (L2).
    Missing,
    /// A store slot could not be read; fail closed rather than guess (C143).
    Media,
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

/// Decides whether a loaded store may be used after a cold boot: the
/// single-candidate case of [`load`]'s decision.
pub fn startup(anchor: &mut impl FreshnessAnchor, store: &StoreState) -> Result<Startup, Denied> {
    decide(anchor, [Some(*store), None]).map(|(_, _, how)| how)
}

/// Called after `next` is durably committed on top of `next.prev`; success is
/// the acknowledgement. One extend, never retried.
pub fn acknowledge(anchor: &mut impl FreshnessAnchor, next: &StoreState) -> Result<(), Denied> {
    same_instance(anchor, next)?;
    anchor
        .extend(&next.prev, &next.digest)
        .map_err(Denied::Anchor)
}

/// A slot read, write or flush failed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MediaError;

/// Two independent record slots holding the authenticated store (C143).
///
/// The slots are volume sectors. HOST MODEL ASSUMPTIONS, owned by the volume
/// and block layer (not N, not B) and not established natively: a write can
/// tear only the slot it targets, leaving any prefix of the new bytes over the
/// old contents; `flush` returning `Ok` makes every earlier write durable.
pub trait SlotMedia {
    /// Fills `out` with the slot's bytes; records are self-delimiting.
    fn read(&mut self, slot: usize, out: &mut [u8]) -> Result<(), MediaError>;
    fn write(&mut self, slot: usize, record: &[u8]) -> Result<(), MediaError>;
    fn flush(&mut self) -> Result<(), MediaError>;
}

/// Authenticates and decodes one slot's bytes (B02 secure records). Torn,
/// truncated or forged bytes must yield `None`. The expected context and
/// length come from the verifier's own state, never from the slot bytes. A
/// tear before the first differing byte leaves the older record intact and
/// authentic; `load` rejects it through the chain, not through the verifier.
pub trait RecordVerifier {
    fn open(&mut self, slot_bytes: &[u8]) -> Option<StoreState>;
}

/// The store accepted at startup and the slot holding it. The other slot is
/// the only one a commit may write.
#[derive(Debug)]
pub struct Session {
    slot: usize,
    state: StoreState,
    /// Set after a failed acknowledge: the chain may or may not have moved, so
    /// the writable slot is unknown until the next `load` re-reads it.
    reload: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommitError {
    /// The anchored slot is untouched. The record may still have become
    /// durable (for example a complete write with a failed flush), so the next
    /// `load` may roll it forward: its outcome is unknown until then, unless a
    /// later successful commit supersedes it. Never report it as refused.
    Media(MediaError),
    /// `next` is not on top of the accepted state, or `record` does not
    /// authenticate to exactly `next`; nothing written.
    Rejected,
    /// A previous acknowledge failed, or the anchor no longer holds (or could
    /// not confirm) the accepted state; nothing written. `load` again before
    /// any write.
    Reload,
    /// The acknowledge failed after the record was durable; the extend may
    /// or may not have happened. The outcome is unknown until the next `load`,
    /// which accepts, rolls forward or denies; never retried blindly.
    Denied(Denied),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Class {
    Anchored,
    Pending,
    Stale,
    Foreign,
}

fn classify(instance: &Digest, chain: &Digest, state: &StoreState) -> Class {
    if state.instance != *instance {
        Class::Foreign
    } else if *chain == state.anchored() {
        Class::Anchored
    } else if *chain == state.prev {
        Class::Pending
    } else {
        Class::Stale
    }
}

/// Startup over both slots. `buf` is one slot long. A never-acknowledged
/// store wins over the anchored one, so a durable commit (for example a
/// revocation) is rolled forward rather than dropped; if that extend fails the
/// result is a denial, never the older store. The slots are flushed before any
/// decision, so a read can never roll the anchor onto bytes that are not yet
/// durable; a failed flush is a denial.
pub fn load(
    media: &mut impl SlotMedia,
    verifier: &mut impl RecordVerifier,
    anchor: &mut impl FreshnessAnchor,
    buf: &mut [u8],
) -> Result<(Session, Startup), Denied> {
    let mut states = [None; 2];
    for (slot, state) in states.iter_mut().enumerate() {
        media.read(slot, buf).map_err(|_| Denied::Media)?;
        *state = verifier.open(buf);
    }
    media.flush().map_err(|_| Denied::Media)?;
    let (slot, state, how) = decide(anchor, states)?;
    Ok((
        Session {
            slot,
            state,
            reload: false,
        },
        how,
    ))
}

/// The one startup rule over up to two authenticated candidates. A
/// never-acknowledged store wins over the anchored one; a failed roll-forward
/// is a denial, never a fallback.
fn decide(
    anchor: &mut impl FreshnessAnchor,
    states: [Option<StoreState>; 2],
) -> Result<(usize, StoreState, Startup), Denied> {
    let instance = anchor.instance().map_err(Denied::Anchor)?;
    let chain = anchor.read().map_err(Denied::Anchor)?;
    let class = |slot: usize| states[slot].map(|s| classify(&instance, &chain, &s));
    let pending: [bool; 2] = [0, 1].map(|slot| class(slot) == Some(Class::Pending));
    if pending[0] && pending[1] && states[0] != states[1] {
        return Err(Denied::Stale); // two unacknowledged siblings: undecidable
    }
    if let Some(slot) = (0..2).find(|&slot| pending[slot]) {
        let state = states[slot].expect("classified");
        anchor
            .extend(&state.prev, &state.digest)
            .map_err(Denied::Anchor)?;
        return Ok((slot, state, Startup::RolledForward));
    }
    if let Some(slot) = (0..2).find(|&slot| class(slot) == Some(Class::Anchored)) {
        let state = states[slot].expect("classified");
        return Ok((slot, state, Startup::Accept));
    }
    match [class(0), class(1)] {
        [None, None] => Err(Denied::Missing),
        [a, b]
            if [a, b]
                .iter()
                .all(|c| matches!(c, None | Some(Class::Foreign))) =>
        {
            Err(Denied::Foreign)
        }
        _ => Err(Denied::Stale),
    }
}

impl Session {
    /// The accepted (anchored) store state.
    pub fn state(&self) -> &StoreState {
        &self.state
    }

    /// Commit-then-advance into the slot that does not hold the anchored
    /// store; that copy stays readable until the anchor has moved past it.
    pub fn commit(
        &mut self,
        media: &mut impl SlotMedia,
        verifier: &mut impl RecordVerifier,
        anchor: &mut impl FreshnessAnchor,
        record: &[u8],
        next: &StoreState,
    ) -> Result<(), CommitError> {
        if self.reload {
            return Err(CommitError::Reload);
        }
        if next.instance != self.state.instance
            || next.prev != self.state.anchored()
            || verifier.open(record) != Some(*next)
        {
            return Err(CommitError::Rejected);
        }
        // A stale session's writable slot may hold the anchored store now.
        if anchor.read().ok() != Some(self.state.anchored()) {
            self.reload = true;
            return Err(CommitError::Reload);
        }
        let slot = 1 - self.slot;
        media
            .write(slot, record)
            .and_then(|()| media.flush())
            .map_err(CommitError::Media)?;
        if let Err(denied) = acknowledge(anchor, next) {
            self.reload = true;
            return Err(CommitError::Denied(denied));
        }
        self.slot = slot;
        self.state = *next;
        Ok(())
    }
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
        /// B re-provisions the domain right after the next `instance()` answer.
        reprovision_after_instance: Option<Digest>,
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
                reprovision_after_instance: None,
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
            let answer = self.instance;
            if let Some(next) = self.reprovision_after_instance.take() {
                self.instance = next;
                self.chain = Some(genesis_chain(&next));
            }
            Ok(answer)
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

    /// Re-provisioning between the instance check and the read or extend of
    /// one operation (CodeRabbit on #543): the new genesis chain binds the new
    /// instance, so no store of the old instance matches it. A same-instance
    /// re-genesis is excluded by B's fresh-nonce rule, not by this module.
    #[test]
    fn reprovisioning_after_the_instance_check_never_accepts_an_old_store() {
        let mut anchor = Fake::provisioned(I0);
        anchor.reprovision_after_instance = Some(I1);
        assert_eq!(startup(&mut anchor, &first(I0, 1)), Err(Denied::Stale));
        let (mut anchor, a) = at_a();
        anchor.reprovision_after_instance = Some(I1);
        assert_eq!(startup(&mut anchor, &a), Err(Denied::Stale));
        let (mut anchor, a) = at_a();
        anchor.reprovision_after_instance = Some(I1);
        assert_eq!(startup(&mut anchor, &child(&a, 3)), Err(Denied::Stale));
        let (mut anchor, a) = at_a();
        anchor.reprovision_after_instance = Some(I1);
        let writes = anchor.writes;
        assert_eq!(
            acknowledge(&mut anchor, &child(&a, 2)),
            Err(Denied::Anchor(AnchorError::Conflict))
        );
        assert_eq!(anchor.writes, writes);
        assert_eq!(anchor.read(), Ok(genesis_chain(&I1)));
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

    // ---- C143: two-slot store replacement ----

    /// One slot: instance, prev, digest, then a TEST-ONLY tag standing in for
    /// B02's record authentication.
    const LEN: usize = 128;
    const TEST_KEY: &[u8] = b"TEST-ONLY-N04-SLOT-KEY";

    fn record(state: &StoreState) -> [u8; LEN] {
        let mut out = [0; LEN];
        out[..32].copy_from_slice(&state.instance);
        out[32..64].copy_from_slice(&state.prev);
        out[64..96].copy_from_slice(&state.digest);
        let tag = sha(&[TEST_KEY, &out[..96]]);
        out[96..].copy_from_slice(&tag);
        out
    }

    struct Mac;

    impl RecordVerifier for Mac {
        fn open(&mut self, bytes: &[u8]) -> Option<StoreState> {
            let bytes: &[u8; LEN] = bytes.try_into().ok()?;
            if sha(&[TEST_KEY, &bytes[..96]]) != bytes[96..] {
                return None;
            }
            let field = |i: usize| -> Digest { bytes[i..i + 32].try_into().unwrap() };
            Some(StoreState {
                instance: field(0),
                prev: field(32),
                digest: field(64),
            })
        }
    }

    /// Two slots; `budget` bytes of the next write land, then it fails (tear).
    struct Disk {
        slots: [[u8; LEN]; 2],
        budget: Option<usize>,
        unreadable: Option<usize>,
        fail_flush: bool,
    }

    impl Disk {
        fn with(a: Option<&StoreState>, b: Option<&StoreState>) -> Self {
            let fill = |s: Option<&StoreState>| s.map_or([0xEE; LEN], record);
            Self {
                slots: [fill(a), fill(b)],
                budget: None,
                unreadable: None,
                fail_flush: false,
            }
        }
    }

    impl SlotMedia for Disk {
        fn read(&mut self, slot: usize, out: &mut [u8]) -> Result<(), MediaError> {
            if self.unreadable == Some(slot) {
                return Err(MediaError);
            }
            out.copy_from_slice(&self.slots[slot]);
            Ok(())
        }

        fn write(&mut self, slot: usize, record: &[u8]) -> Result<(), MediaError> {
            let n = self
                .budget
                .take()
                .map_or(record.len(), |b| b.min(record.len()));
            self.slots[slot][..n].copy_from_slice(&record[..n]);
            if n < record.len() {
                Err(MediaError)
            } else {
                Ok(())
            }
        }

        fn flush(&mut self) -> Result<(), MediaError> {
            if self.fail_flush {
                Err(MediaError)
            } else {
                Ok(())
            }
        }
    }

    fn boot(disk: &mut Disk, anchor: &mut Fake) -> Result<(Session, Startup), Denied> {
        load(disk, &mut Mac, anchor, &mut [0; LEN])
    }

    /// S0 rolled forward from genesis into slot 0, then S1 acknowledged in slot 1.
    fn at_s1() -> (Disk, Fake, Session, StoreState, StoreState) {
        let mut anchor = Fake::provisioned(I0);
        let s0 = first(I0, 1);
        let mut disk = Disk::with(Some(&s0), None);
        let (mut session, how) = boot(&mut disk, &mut anchor).unwrap();
        assert_eq!((how, session.slot), (Startup::RolledForward, 0));
        let s1 = child(&s0, 2);
        session
            .commit(&mut disk, &mut Mac, &mut anchor, &record(&s1), &s1)
            .unwrap();
        assert_eq!(session.slot, 1);
        (disk, anchor, session, s0, s1)
    }

    /// C143: S2 torn into slot 0 (which still holds S0, two commits back) at
    /// every byte boundary, then a crash before the extend. Below the full
    /// length the anchored S1 is accepted with no extend; the complete record
    /// (here: written, flush reported a failure) rolls forward over the intact
    /// S1, so a `Media` error is not a refusal.
    #[test]
    fn torn_commit_at_every_byte_keeps_the_anchored_store() {
        for cut in 0..=LEN {
            let (mut disk, mut anchor, mut session, _, s1) = at_s1();
            let s2 = child(&s1, 3);
            disk.budget = Some(cut);
            disk.fail_flush = true;
            let pre = anchor.writes;
            assert_eq!(
                session.commit(&mut disk, &mut Mac, &mut anchor, &record(&s2), &s2),
                Err(CommitError::Media(MediaError)),
                "cut {cut}"
            );
            assert_eq!(anchor.writes, pre, "failed commit extended at cut {cut}");
            assert_eq!(
                disk.slots[1],
                record(&s1),
                "anchored copy touched at cut {cut}"
            );
            disk.fail_flush = false;
            let (booted, how) = boot(&mut disk, &mut anchor).unwrap();
            if cut < LEN {
                assert_eq!(
                    (how, booted.slot, booted.state),
                    (Startup::Accept, 1, s1),
                    "cut {cut}"
                );
                assert_eq!(anchor.writes, pre, "cut {cut}");
            } else {
                assert_eq!(
                    (how, booted.slot, booted.state),
                    (Startup::RolledForward, 0, s2)
                );
                assert_eq!(anchor.writes, pre + 1);
            }
        }
    }

    /// The M1 hostile extend over two slots: neither copy is revived.
    #[test]
    fn hostile_extend_denies_both_slots() {
        let (mut disk, mut anchor, _, _, _) = at_s1();
        anchor.hostile_extend(&[0x66; 32]);
        let writes = anchor.writes;
        assert_eq!(boot(&mut disk, &mut anchor).err(), Some(Denied::Stale));
        assert_eq!(anchor.writes, writes);
    }

    /// A lost acknowledge response may have moved the chain onto the slot the
    /// session would write next; no write happens until `load` re-reads it.
    #[test]
    fn lost_acknowledge_blocks_writes_until_reload() {
        let (mut disk, mut anchor, mut session, _, s1) = at_s1();
        let s2 = child(&s1, 3);
        anchor.lose_next_response = true;
        assert_eq!(
            session.commit(&mut disk, &mut Mac, &mut anchor, &record(&s2), &s2),
            Err(CommitError::Denied(Denied::Anchor(
                AnchorError::Unavailable
            )))
        );
        let s3 = child(&s1, 4);
        let slots = disk.slots;
        assert_eq!(
            session.commit(&mut disk, &mut Mac, &mut anchor, &record(&s3), &s3),
            Err(CommitError::Reload)
        );
        assert_eq!(disk.slots, slots, "anchored S2 overwritten");
        let (mut session, how) = boot(&mut disk, &mut anchor).unwrap();
        assert_eq!((how, session.slot, session.state), (Startup::Accept, 0, s2));
        let s3 = child(&s2, 4);
        session
            .commit(&mut disk, &mut Mac, &mut anchor, &record(&s3), &s3)
            .unwrap();
        assert_eq!(disk.slots[0], record(&s2), "previous copy kept");
    }

    #[test]
    fn one_bad_slot_does_not_deny_the_acceptable_one() {
        let (_, mut anchor, _, s0, s1) = at_s1();
        let mut garbage = Disk::with(None, Some(&s1));
        let foreign = StoreState { instance: I1, ..s1 };
        let mut alien = Disk::with(Some(&foreign), Some(&s1));
        let mut stale = Disk::with(Some(&s0), Some(&s1));
        for disk in [&mut garbage, &mut alien, &mut stale] {
            let (session, how) = boot(disk, &mut anchor).unwrap();
            assert_eq!((how, session.slot, session.state), (Startup::Accept, 1, s1));
        }
    }

    #[test]
    fn no_acceptable_slot_is_denied_never_genesis() {
        let (_, mut anchor, _, s0, s1) = at_s1();
        let foreign = StoreState { instance: I1, ..s1 };
        let cases = [
            (Disk::with(None, None), Denied::Missing),
            (Disk::with(Some(&foreign), None), Denied::Foreign),
            (Disk::with(Some(&s0), Some(&foreign)), Denied::Stale),
        ];
        for (mut disk, denied) in cases {
            let writes = anchor.writes;
            assert_eq!(boot(&mut disk, &mut anchor).err(), Some(denied));
            assert_eq!(anchor.writes, writes);
        }
        let mut disk = Disk::with(None, Some(&s1));
        disk.unreadable = Some(0);
        assert_eq!(boot(&mut disk, &mut anchor).err(), Some(Denied::Media));
    }

    #[test]
    fn two_unacknowledged_siblings_are_denied_without_an_extend() {
        let (_, mut anchor, _, _, s1) = at_s1();
        let mut disk = Disk::with(Some(&child(&s1, 3)), Some(&child(&s1, 4)));
        let writes = anchor.writes;
        assert_eq!(boot(&mut disk, &mut anchor).err(), Some(Denied::Stale));
        assert_eq!(anchor.writes, writes);
    }

    /// A failed roll-forward never falls back to the older anchored copy.
    #[test]
    fn failed_roll_forward_over_an_anchored_copy_is_a_denial() {
        let (mut disk, mut anchor, _, _, s1) = at_s1();
        disk.slots[0] = record(&child(&s1, 3));
        anchor.budget = 0;
        assert_eq!(
            boot(&mut disk, &mut anchor).err(),
            Some(Denied::Anchor(AnchorError::RateLimited))
        );
    }

    /// Two sessions from one anchored store: once the first commits, the
    /// second may only reload, never write over the newly anchored slot.
    #[test]
    fn stale_session_never_overwrites_the_anchored_slot() {
        let (mut disk, mut anchor, mut first, _, s1) = at_s1();
        let (mut stale, _) = boot(&mut disk, &mut anchor).unwrap();
        let s2 = child(&s1, 3);
        first
            .commit(&mut disk, &mut Mac, &mut anchor, &record(&s2), &s2)
            .unwrap();
        let slots = disk.slots;
        let other = child(&s1, 4);
        assert_eq!(
            stale.commit(&mut disk, &mut Mac, &mut anchor, &record(&other), &other),
            Err(CommitError::Reload)
        );
        assert_eq!(disk.slots, slots, "anchored S2 overwritten");
        let (booted, how) = boot(&mut disk, &mut anchor).unwrap();
        assert_eq!((how, booted.state), (Startup::Accept, s2));
    }

    /// Bytes read back but not flushed are never rolled forward.
    #[test]
    fn failed_flush_at_startup_denies_without_a_roll_forward() {
        let (mut disk, mut anchor, mut session, _, s1) = at_s1();
        let s2 = child(&s1, 3);
        disk.fail_flush = true;
        assert_eq!(
            session.commit(&mut disk, &mut Mac, &mut anchor, &record(&s2), &s2),
            Err(CommitError::Media(MediaError))
        );
        let writes = anchor.writes;
        assert_eq!(boot(&mut disk, &mut anchor).err(), Some(Denied::Media));
        assert_eq!(anchor.writes, writes);
    }

    #[test]
    fn commit_rejects_a_mismatched_record_or_parent_without_writing() {
        let (mut disk, mut anchor, mut session, s0, s1) = at_s1();
        let slots = disk.slots;
        let s2 = child(&s1, 3);
        let off_parent = child(&s0, 9);
        for (bytes, next) in [
            (record(&child(&s1, 4)), s2),
            (record(&off_parent), off_parent),
        ] {
            assert_eq!(
                session.commit(&mut disk, &mut Mac, &mut anchor, &bytes, &next),
                Err(CommitError::Rejected)
            );
        }
        assert_eq!(disk.slots, slots);
    }
}
