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
//! store nor a different store at the same generation is accepted.

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
}

/// An authenticated store state as loaded from durable storage. `parent` is the
/// digest of the state at `generation - 1` that this state was derived from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StoreState {
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
            }
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
    }

    const D0: Digest = [0; 32];
    const D1: Digest = [1; 32];
    const D2: Digest = [2; 32];

    fn state(generation: u64, digest: Digest, parent: Digest) -> StoreState {
        StoreState {
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
}
