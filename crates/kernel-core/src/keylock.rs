//! B01-LOCK (ADR-KEYRING-LOCK-20261126): the F1 erasure predicate, pure so the
//! host tests it. The kernel's `keylock::erased()` fills it from its counters.

/// Kernel state at the moment of the question.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Erasure {
    /// LOCKED latched: the teardown ran and the exit-path scrub read back zero.
    pub locked: bool,
    /// Successful LOCK calls.
    pub locks: u64,
    /// Whole-stack scrubs that `m3_restore` completed.
    pub scrubs: u64,
    /// Scrubs that read back a non-zero word.
    pub residue: u64,
    /// A scrub was requested and has not run yet.
    pub scrub_pending: bool,
    /// The entropy broker holds no pending or answered bytes; its delivery
    /// buffer is wiped once a request is delivered.
    pub broker_quiet: bool,
}

/// F1 counts as proven only after the stack and buffer erasure actually
/// completed (project-owner decision of 26 November 2026).
pub const fn erased(e: Erasure) -> bool {
    e.locked
        && e.locks >= 1
        && !e.scrub_pending
        && e.scrubs >= e.locks
        && e.residue == 0
        && e.broker_quiet
}

#[cfg(test)]
mod tests {
    use super::*;

    const DONE: Erasure = Erasure {
        locked: true,
        locks: 1,
        scrubs: 1,
        residue: 0,
        scrub_pending: false,
        broker_quiet: true,
    };

    #[test]
    fn erased_needs_every_condition() {
        assert!(erased(DONE));
        for missing in [
            Erasure {
                locked: false,
                ..DONE
            },
            Erasure { locks: 0, ..DONE },
            Erasure { scrubs: 0, ..DONE },
            Erasure { residue: 1, ..DONE },
            Erasure {
                scrub_pending: true,
                ..DONE
            },
            Erasure {
                broker_quiet: false,
                ..DONE
            },
            Erasure { locks: 2, ..DONE },
        ] {
            assert!(!erased(missing), "{missing:?}");
        }
    }
}
