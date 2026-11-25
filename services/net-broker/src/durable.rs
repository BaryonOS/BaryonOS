//! Cold-freshness protocol for durable network grants (ADR-NET-ISSUER-20261125,
//! decision 5B; contract in `docs/implementation/COLD-FRESHNESS-ANCHOR.md`).
//!
//! HOST PROTOTYPE ONLY: no native anchor is wired, so 5A (every grant denied
//! after a cold boot) remains the product behaviour. The anchor is reached only
//! through [`FreshnessAnchor`], the network-domain capability that workstream B
//! owns; this module never touches a TPM. Store authentication and the state
//! digest are the caller's (B02 secure records) and stay opaque here.
//!
//! Ordering is commit-then-advance: a mutation is acknowledged only after the
//! anchor advance succeeds. Every anchor generation carries the digest of the
//! exact authenticated store state (grants and tombstones), so neither an older
//! store nor a different store at the same generation is accepted, provided
//! the slot is the protected slot of the M1 decision: with the unprotected
//! contract a hostile slot write can select a sibling store (ignored residual
//! tests below). A store also binds the anchor instance, so a replaced or
//! reset TPM yields `Foreign` until authorised recovery re-anchors from genesis.

/// Digest of one authenticated store state, computed by the store owner.
pub type Digest = [u8; 32];

/// What the anchor holds for the network domain.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AnchorState {
    pub generation: u64,
    pub binding: Digest,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorError {
    Unavailable,
    Corrupt,
    /// The compare-and-swap saw a generation other than the expected one.
    Conflict,
    /// The counter cannot advance further.
    Exhausted,
    /// The anchor owner's per-domain write budget is spent (NV endurance).
    RateLimited,
}

/// One anchor read. The backend writes the next binding before it increments
/// its hardware-monotonic counter, so a crash between the two leaves `Pending`:
/// the binding of `generation` is gone and only the pending state can complete.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorRead {
    Current(AnchorState),
    Pending {
        generation: u64,
        next_binding: Digest,
    },
}

/// Bounded network-domain capability; no other anchor domain is reachable.
pub trait FreshnessAnchor {
    fn read(&mut self) -> Result<AnchorRead, AnchorError>;
    /// Atomic compare-and-swap from `expected` to `expected + 1`, storing `binding`.
    fn advance(&mut self, expected: u64, binding: Digest) -> Result<(), AnchorError>;
    /// Identity of this anchor instance; it changes when B re-provisions the
    /// domain, e.g. after authorised recovery from a TPM replacement (C665).
    fn instance(&mut self) -> Result<Digest, AnchorError>;
}

/// An authenticated store state as loaded from durable storage. `parent` is the
/// digest of the state at `generation - 1` that this state was derived from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreState {
    /// Anchor instance this store was anchored to at genesis.
    pub instance: Digest,
    pub generation: u64,
    pub digest: Digest,
    pub parent: Digest,
}

/// Why durable grants are unavailable; the caller then applies 5A (all denied).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Denied {
    Anchor(AnchorError),
    /// The store is not newer than the anchor where it must be.
    Rollback,
    /// The store is more than one generation ahead, impossible under this protocol.
    Ahead,
    /// Same or next generation, but not the state the anchor is bound to.
    Fork,
    /// The store belongs to another anchor instance (replaced or reset TPM);
    /// only authorised recovery may re-anchor it from genesis.
    Foreign,
}

fn same_instance(anchor: &mut impl FreshnessAnchor, store: &StoreState) -> Result<(), Denied> {
    if anchor.instance().map_err(Denied::Anchor)? == store.instance {
        Ok(())
    } else {
        Err(Denied::Foreign)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Startup {
    /// The store matches the anchor.
    Accept,
    /// A commit landed before a crash; the anchor was advanced to it now.
    RolledForward,
}

/// Completes an interrupted advance, or reports the current anchor state.
fn settle(anchor: &mut impl FreshnessAnchor, store: &StoreState) -> Result<Settled, Denied> {
    match anchor.read().map_err(Denied::Anchor)? {
        AnchorRead::Current(current) => Ok(Settled::Current(current)),
        AnchorRead::Pending {
            generation,
            next_binding,
        } => {
            if store.generation != generation.checked_add(1).ok_or(Denied::Ahead)?
                || store.digest != next_binding
            {
                return Err(Denied::Fork);
            }
            anchor
                .advance(generation, next_binding)
                .map_err(Denied::Anchor)?;
            Ok(Settled::Completed)
        }
    }
}

enum Settled {
    Current(AnchorState),
    Completed,
}

/// Decides whether a loaded store may be used after a cold boot.
pub fn startup(anchor: &mut impl FreshnessAnchor, store: &StoreState) -> Result<Startup, Denied> {
    same_instance(anchor, store)?;
    let current = match settle(anchor, store)? {
        Settled::Current(current) => current,
        Settled::Completed => return Ok(Startup::RolledForward),
    };
    if store.generation < current.generation {
        return Err(Denied::Rollback);
    }
    if store.generation == current.generation {
        return if store.digest == current.binding {
            Ok(Startup::Accept)
        } else {
            Err(Denied::Fork)
        };
    }
    advance(anchor, current, store)?;
    Ok(Startup::RolledForward)
}

/// Called after `next` is durably committed; success is the acknowledgement.
pub fn acknowledge(anchor: &mut impl FreshnessAnchor, next: &StoreState) -> Result<(), Denied> {
    same_instance(anchor, next)?;
    let current = match settle(anchor, next)? {
        Settled::Current(current) => current,
        Settled::Completed => return Ok(()),
    };
    if next.generation <= current.generation {
        return Err(Denied::Rollback);
    }
    advance(anchor, current, next)
}

fn advance(
    anchor: &mut impl FreshnessAnchor,
    current: AnchorState,
    next: &StoreState,
) -> Result<(), Denied> {
    let Some(expected_next) = current.generation.checked_add(1) else {
        return Err(Denied::Anchor(AnchorError::Exhausted));
    };
    if next.generation != expected_next {
        return Err(Denied::Ahead);
    }
    if next.parent != current.binding {
        return Err(Denied::Fork);
    }
    anchor
        .advance(current.generation, next.digest)
        .map_err(Denied::Anchor)
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake {
        state: AnchorState,
        fail_read: Option<AnchorError>,
        fail_advance: Option<AnchorError>,
        /// Slot written, counter not yet incremented (crash between the writes).
        pending: Option<Digest>,
        writes: u32,
        instance: Digest,
        /// Protected slot (M1 decision): writes need B's authorisation, which
        /// the modelled attacker lacks. Unprotected models today's contract.
        protected: bool,
    }

    impl Fake {
        fn at(generation: u64, binding: Digest) -> Self {
            Self {
                state: AnchorState {
                    generation,
                    binding,
                },
                fail_read: None,
                fail_advance: None,
                pending: None,
                writes: 0,
                instance: I0,
                protected: false,
            }
        }

        fn protected_at(generation: u64, binding: Digest) -> Self {
            Self {
                protected: true,
                ..Self::at(generation, binding)
            }
        }

        /// An attacker's direct slot write outside the capability interface.
        fn hostile_slot_write(&mut self, binding: Digest) -> bool {
            if self.protected {
                return false;
            }
            self.state.binding = binding;
            true
        }
    }

    impl FreshnessAnchor for Fake {
        fn read(&mut self) -> Result<AnchorRead, AnchorError> {
            if let Some(error) = self.fail_read {
                return Err(error);
            }
            Ok(match self.pending {
                Some(next_binding) => AnchorRead::Pending {
                    generation: self.state.generation,
                    next_binding,
                },
                None => AnchorRead::Current(self.state),
            })
        }
        fn advance(&mut self, expected: u64, binding: Digest) -> Result<(), AnchorError> {
            if let Some(error) = self.fail_advance {
                return Err(error);
            }
            if expected != self.state.generation
                || self.pending.is_some_and(|pending| pending != binding)
            {
                return Err(AnchorError::Conflict);
            }
            self.pending = None;
            self.state = AnchorState {
                generation: expected + 1,
                binding,
            };
            self.writes += 1;
            Ok(())
        }
        fn instance(&mut self) -> Result<Digest, AnchorError> {
            if let Some(error) = self.fail_read {
                return Err(error);
            }
            Ok(self.instance)
        }
    }

    const D0: Digest = [0; 32];
    const D1: Digest = [1; 32];
    const D2: Digest = [2; 32];
    const I0: Digest = [0xA0; 32];
    const I1: Digest = [0xA1; 32];

    fn state(generation: u64, digest: Digest, parent: Digest) -> StoreState {
        StoreState {
            instance: I0,
            generation,
            digest,
            parent,
        }
    }

    #[test]
    fn matching_store_is_accepted_without_a_write() {
        let mut anchor = Fake::at(5, D1);
        assert_eq!(startup(&mut anchor, &state(5, D1, D0)), Ok(Startup::Accept));
        assert_eq!(anchor.writes, 0);
    }

    /// B's provisioning convention: slot `{gen: c, binding: 0^32}`; the first
    /// store is `c + 1` with an all-zero parent.
    #[test]
    fn genesis_slot_accepts_only_a_first_store_with_zero_parent() {
        let mut anchor = Fake::at(7, D0);
        assert_eq!(
            acknowledge(&mut anchor, &state(8, D1, D2)),
            Err(Denied::Fork)
        );
        assert_eq!(acknowledge(&mut anchor, &state(8, D1, D0)), Ok(()));
        assert_eq!(anchor.state.generation, 8);
        assert_eq!(anchor.state.binding, D1);
    }

    #[test]
    fn commit_then_crash_rolls_forward_once() {
        let mut anchor = Fake::at(5, D1);
        let committed = state(6, D2, D1);
        assert_eq!(startup(&mut anchor, &committed), Ok(Startup::RolledForward));
        assert_eq!(
            anchor.state,
            AnchorState {
                generation: 6,
                binding: D2
            }
        );
        assert_eq!(startup(&mut anchor, &committed), Ok(Startup::Accept));
        assert_eq!(anchor.writes, 1);
    }

    #[test]
    fn older_store_is_rollback() {
        let mut anchor = Fake::at(6, D2);
        assert_eq!(
            startup(&mut anchor, &state(5, D1, D0)),
            Err(Denied::Rollback)
        );
    }

    #[test]
    fn store_two_ahead_is_rejected() {
        let mut anchor = Fake::at(5, D1);
        assert_eq!(startup(&mut anchor, &state(7, D2, D1)), Err(Denied::Ahead));
        assert_eq!(anchor.writes, 0);
    }

    /// Commit X at A+1, crash, replay the store at A, commit revocation Y at
    /// A+1 and acknowledge it: presenting X afterwards must not revive it.
    #[test]
    fn same_generation_fork_cannot_revive_a_revoked_grant() {
        let x = [0x58; 32];
        let y = [0x59; 32];
        let mut anchor = Fake::at(5, D1);
        assert_eq!(startup(&mut anchor, &state(5, D1, D0)), Ok(Startup::Accept));
        assert_eq!(acknowledge(&mut anchor, &state(6, y, D1)), Ok(()));
        assert_eq!(startup(&mut anchor, &state(6, x, D1)), Err(Denied::Fork));
    }

    #[test]
    fn next_state_not_derived_from_the_bound_state_is_a_fork() {
        let mut anchor = Fake::at(5, D1);
        assert_eq!(
            startup(&mut anchor, &state(6, D2, [9; 32])),
            Err(Denied::Fork)
        );
        assert_eq!(
            acknowledge(&mut anchor, &state(6, D2, [9; 32])),
            Err(Denied::Fork)
        );
        assert_eq!(anchor.writes, 0);
    }

    #[test]
    fn anchor_failures_deny_and_never_acknowledge() {
        for error in [AnchorError::Unavailable, AnchorError::Corrupt] {
            let mut anchor = Fake::at(5, D1);
            anchor.fail_read = Some(error);
            assert_eq!(
                startup(&mut anchor, &state(5, D1, D0)),
                Err(Denied::Anchor(error))
            );
            assert_eq!(
                acknowledge(&mut anchor, &state(6, D2, D1)),
                Err(Denied::Anchor(error))
            );
        }
        let mut anchor = Fake::at(5, D1);
        anchor.fail_advance = Some(AnchorError::Unavailable);
        assert_eq!(
            acknowledge(&mut anchor, &state(6, D2, D1)),
            Err(Denied::Anchor(AnchorError::Unavailable))
        );
        assert_eq!(anchor.state.generation, 5);
    }

    #[test]
    fn concurrent_advance_loses_the_compare_and_swap() {
        let mut anchor = Fake::at(5, D1);
        let AnchorRead::Current(stale) = anchor.read().unwrap() else {
            panic!("pending")
        };
        assert_eq!(acknowledge(&mut anchor, &state(6, D2, D1)), Ok(()));
        assert_eq!(
            advance(&mut anchor, stale, &state(6, [3; 32], D1)),
            Err(Denied::Anchor(AnchorError::Conflict))
        );
        assert_eq!(anchor.state.binding, D2);
    }

    #[test]
    fn exhausted_counter_denies_instead_of_wrapping() {
        let mut anchor = Fake::at(u64::MAX - 1, D1);
        assert_eq!(acknowledge(&mut anchor, &state(u64::MAX, D2, D1)), Ok(()));
        let AnchorRead::Current(top) = anchor.read().unwrap() else {
            panic!("pending")
        };
        assert_eq!(
            advance(&mut anchor, top, &state(0, [3; 32], D2)),
            Err(Denied::Anchor(AnchorError::Exhausted))
        );
        assert_eq!(
            acknowledge(&mut anchor, &state(0, [3; 32], D2)),
            Err(Denied::Rollback)
        );
        assert_eq!(
            anchor.state,
            AnchorState {
                generation: u64::MAX,
                binding: D2
            }
        );
    }

    /// Crash after the slot write, before the counter increment: the binding
    /// of A is overwritten, so only the store at A+1 with that digest completes.
    #[test]
    fn interrupted_advance_completes_only_for_the_pending_state() {
        let mut anchor = Fake::at(5, D1);
        anchor.pending = Some(D2);
        assert_eq!(startup(&mut anchor, &state(5, D1, D0)), Err(Denied::Fork));
        assert_eq!(
            startup(&mut anchor, &state(6, [3; 32], D1)),
            Err(Denied::Fork)
        );
        assert_eq!(startup(&mut anchor, &state(7, D2, D1)), Err(Denied::Fork));
        assert_eq!(anchor.writes, 0);
        assert_eq!(
            startup(&mut anchor, &state(6, D2, D1)),
            Ok(Startup::RolledForward)
        );
        assert_eq!(
            anchor.read(),
            Ok(AnchorRead::Current(AnchorState {
                generation: 6,
                binding: D2
            }))
        );
    }

    #[test]
    fn interrupted_advance_is_completed_by_the_matching_acknowledgement() {
        let mut anchor = Fake::at(5, D1);
        anchor.pending = Some(D2);
        assert_eq!(
            acknowledge(&mut anchor, &state(6, [3; 32], D1)),
            Err(Denied::Fork)
        );
        assert_eq!(acknowledge(&mut anchor, &state(6, D2, D1)), Ok(()));
        assert_eq!(anchor.state.generation, 6);
    }

    #[test]
    fn rate_limited_advance_is_not_an_acknowledgement() {
        let mut anchor = Fake::at(5, D1);
        anchor.fail_advance = Some(AnchorError::RateLimited);
        assert_eq!(
            acknowledge(&mut anchor, &state(6, D2, D1)),
            Err(Denied::Anchor(AnchorError::RateLimited))
        );
        assert_eq!(
            anchor.state,
            AnchorState {
                generation: 5,
                binding: D1
            }
        );
    }

    #[test]
    fn acknowledging_a_stale_generation_is_rollback() {
        let mut anchor = Fake::at(6, D2);
        assert_eq!(
            acknowledge(&mut anchor, &state(6, D2, D1)),
            Err(Denied::Rollback)
        );
        assert_eq!(
            acknowledge(&mut anchor, &state(5, D1, D0)),
            Err(Denied::Rollback)
        );
    }

    /// Review finding M1 (B, #524): a stale authentic store X at A+1 left by a
    /// crash before W1, plus a replayed S=A and a hostile slot write (T3),
    /// must not displace the acknowledged Y at A+1.
    #[test]
    #[ignore = "M1 residual without a protected slot (unprotected contract)"]
    fn hostile_slot_write_cannot_revive_a_stale_store_at_the_same_generation() {
        let x = state(6, [3; 32], D1); // committed to disk, crash before W1
        let mut anchor = Fake::at(5, D1);
        // Disk attacker replays S=A; accepted under rule (c).
        assert_eq!(startup(&mut anchor, &state(5, D1, D0)), Ok(Startup::Accept));
        // Y carries an acknowledged revocation.
        assert_eq!(acknowledge(&mut anchor, &state(6, D2, D1)), Ok(()));
        // T3 rewrites the slot binding and puts X back on disk.
        assert!(anchor.hostile_slot_write(x.digest));
        assert_ne!(startup(&mut anchor, &x), Ok(Startup::Accept));
    }

    /// M1 reseal candidate, single-boot cases only: after a cold Accept at
    /// S=A the policy process acknowledges a content-identical reseal Z at A+1.
    /// This does NOT bound a hostile slot write across several boots; see the
    /// two ignored `reseal_multi_boot_*` tests below.
    #[test]
    fn reseal_single_boot_cases() {
        let x = state(6, [3; 32], D1); // stale, never acknowledged
        let z = state(6, [4; 32], D1); // reseal: content of S=A, no new change
        let y = state(7, D2, z.digest); // carries the acknowledged revocation
        // Hostile write inside the reseal window: X wins, but X only carries
        // unacknowledged changes and Y can no longer chain to it.
        let mut anchor = Fake::at(5, D1);
        assert_eq!(startup(&mut anchor, &state(5, D1, D0)), Ok(Startup::Accept));
        assert_eq!(acknowledge(&mut anchor, &z), Ok(()));
        assert!(anchor.hostile_slot_write(x.digest));
        assert_eq!(startup(&mut anchor, &x), Ok(Startup::Accept));
        assert_eq!(acknowledge(&mut anchor, &y), Err(Denied::Fork));
        // Hostile write after Y is acknowledged: X is now a rollback.
        let mut anchor = Fake::at(5, D1);
        assert_eq!(startup(&mut anchor, &state(5, D1, D0)), Ok(Startup::Accept));
        assert_eq!(acknowledge(&mut anchor, &z), Ok(()));
        assert_eq!(acknowledge(&mut anchor, &y), Ok(()));
        assert!(anchor.hostile_slot_write(x.digest));
        assert_eq!(startup(&mut anchor, &x), Err(Denied::Rollback));
        // Crash mid-reseal (Z on disk, slot not written) and a replay of S=A:
        // accepted again, so the next boot reseals again; X rolls forward only
        // as the legitimate pre-crash commit.
        let mut anchor = Fake::at(5, D1);
        assert_eq!(startup(&mut anchor, &state(5, D1, D0)), Ok(Startup::Accept));
        assert_eq!(startup(&mut anchor, &x), Ok(Startup::RolledForward));
    }

    /// Boot 1 accepts S=A and writes reseal Z at A+1, then crashes before the
    /// slot write. Boot 2 rolls X forward, so X's revocation is in force. T3
    /// then rewrites the slot to D(Z) and presents Z.
    #[test]
    #[ignore = "M1 residual without a protected slot: reseal is insufficient"]
    fn reseal_multi_boot_crashed_reseal_revives_over_rolled_forward_change() {
        let x = state(6, [3; 32], D1); // carries revocation R
        let z = state(6, [4; 32], D1); // crashed reseal of S=A
        let mut anchor = Fake::at(5, D1);
        assert_eq!(startup(&mut anchor, &state(5, D1, D0)), Ok(Startup::Accept));
        assert_eq!(startup(&mut anchor, &x), Ok(Startup::RolledForward));
        assert!(anchor.hostile_slot_write(z.digest));
        assert_ne!(startup(&mut anchor, &z), Ok(Startup::Accept));
    }

    /// Reseal after every cold outcome. Boot 1 rolls Z forward and crashes in
    /// its reseal Z3. Boot 2 accepts X under a forged slot and acknowledges
    /// reseal X2 (R durable). T3 then forges D(Z3).
    #[test]
    #[ignore = "M1 residual without a protected slot: reseal is insufficient"]
    fn reseal_multi_boot_sibling_reseal_revives_over_acknowledged_change() {
        let x = state(6, [3; 32], D1); // carries revocation R
        let z = state(6, [4; 32], D1);
        let z3 = state(7, [5; 32], z.digest); // crashed reseal of Z
        let x2 = state(7, [6; 32], x.digest); // acknowledged reseal of X
        let mut anchor = Fake::at(5, D1);
        assert_eq!(startup(&mut anchor, &z), Ok(Startup::RolledForward));
        assert!(anchor.hostile_slot_write(x.digest));
        assert_eq!(startup(&mut anchor, &x), Ok(Startup::Accept));
        assert_eq!(acknowledge(&mut anchor, &x2), Ok(()));
        assert!(anchor.hostile_slot_write(z3.digest));
        assert_ne!(startup(&mut anchor, &z3), Ok(Startup::Accept));
    }

    // M1 protected-slot decision: the same attack sequences with B's
    // authorised slot. Host model of the guarantee B must demonstrate natively.

    #[test]
    fn protected_slot_refuses_the_single_boot_m1_attack() {
        let x = state(6, [3; 32], D1);
        let mut anchor = Fake::protected_at(5, D1);
        assert_eq!(startup(&mut anchor, &state(5, D1, D0)), Ok(Startup::Accept));
        assert_eq!(acknowledge(&mut anchor, &state(6, D2, D1)), Ok(()));
        assert!(!anchor.hostile_slot_write(x.digest));
        assert_eq!(startup(&mut anchor, &x), Err(Denied::Fork));
    }

    #[test]
    fn protected_slot_refuses_both_multi_boot_attacks() {
        let x = state(6, [3; 32], D1);
        let z = state(6, [4; 32], D1);
        // Rolled-forward X must not be displaced by a crashed sibling Z.
        let mut anchor = Fake::protected_at(5, D1);
        assert_eq!(startup(&mut anchor, &x), Ok(Startup::RolledForward));
        assert!(!anchor.hostile_slot_write(z.digest));
        assert_eq!(startup(&mut anchor, &z), Err(Denied::Fork));
        // Replay across further boots: older and sibling stores stay refused.
        assert_eq!(
            startup(&mut anchor, &state(5, D1, D0)),
            Err(Denied::Rollback)
        );
        let y = state(7, D2, x.digest);
        assert_eq!(acknowledge(&mut anchor, &y), Ok(()));
        for stale in [x, z, state(7, [5; 32], z.digest)] {
            assert!(!anchor.hostile_slot_write(stale.digest));
            assert_ne!(startup(&mut anchor, &stale), Ok(Startup::Accept));
        }
        assert_eq!(startup(&mut anchor, &y), Ok(Startup::Accept));
    }

    #[test]
    fn protected_slot_interrupted_commit_completes_only_once() {
        let mut anchor = Fake::protected_at(5, D1);
        anchor.pending = Some(D2);
        assert_eq!(startup(&mut anchor, &state(5, D1, D0)), Err(Denied::Fork));
        assert_eq!(
            startup(&mut anchor, &state(6, [3; 32], D1)),
            Err(Denied::Fork)
        );
        assert_eq!(
            startup(&mut anchor, &state(6, D2, D1)),
            Ok(Startup::RolledForward)
        );
        assert_eq!(anchor.state.generation, 6);
        assert_eq!(startup(&mut anchor, &state(6, D2, D1)), Ok(Startup::Accept));
    }

    #[test]
    fn replaced_tpm_rejects_the_store_until_authorised_genesis() {
        let store = state(9, D2, D1);
        // B re-provisions the domain after authorised recovery: new instance.
        let mut anchor = Fake::protected_at(40, D0);
        anchor.instance = I1;
        assert_eq!(startup(&mut anchor, &store), Err(Denied::Foreign));
        assert_eq!(acknowledge(&mut anchor, &store), Err(Denied::Foreign));
        assert_eq!(anchor.writes, 0);
        // Recovery creates a fresh genesis store on the new instance; nothing
        // of the old store (grants or tombstones) is carried across.
        let genesis = StoreState {
            instance: I1,
            ..state(41, D1, D0)
        };
        assert_eq!(acknowledge(&mut anchor, &genesis), Ok(()));
        assert_eq!(startup(&mut anchor, &genesis), Ok(Startup::Accept));
    }

    #[test]
    fn unavailable_instance_denies() {
        let mut anchor = Fake::protected_at(5, D1);
        anchor.fail_read = Some(AnchorError::Unavailable);
        assert_eq!(
            startup(&mut anchor, &state(5, D1, D0)),
            Err(Denied::Anchor(AnchorError::Unavailable))
        );
    }
}
