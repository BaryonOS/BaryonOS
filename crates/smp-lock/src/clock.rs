//! PIT edge accounting for the idle clock check (K06 step 5c-2, ADR-082).
//!
//! After every wake of the idle BSP the kernel samples how many PIT periods
//! passed (counter latch), how many IRQ0s it counted and whether one is pending
//! in the PIC. This module keeps the books: which periods never became an
//! IRQ0, which of those the kernel could not have prevented, and which counted
//! IRQ0s have no period behind them. It never advances a clock (ADR-053).

/// What one sample saw since the previous one.
pub struct Sample {
    /// PIT periods that passed.
    pub periods: u64,
    /// IRQ0s the kernel counted.
    pub counted: u64,
    /// An IRQ0 is pending in the PIC now.
    pub pending: bool,
    /// The kernel could take IRQ0 except for short, measured spans.
    pub receptive: bool,
}

pub struct Edges {
    pending: bool,
    /// What the previous sample booked as missing and how: undone for each
    /// edge that arrives after all at the samples right after it.
    opened: (u64, bool),
    /// Periods whose edge never became an IRQ0.
    pub lost: u64,
    /// The part of `lost` that fell while the kernel was receptive.
    pub host: u64,
    /// Counted IRQ0s beyond the periods that passed: spurious or doubled.
    pub extra: u64,
}

impl Edges {
    /// The first sample. Its period may be due but not yet presented, so one
    /// IRQ0 without a new period is expected at the next sample only.
    pub const fn new(pending: bool) -> Self {
        Self {
            pending,
            opened: (1, false),
            lost: 0,
            host: 0,
            extra: 0,
        }
    }

    pub fn account(&mut self, sample: Sample) {
        // Periods not yet counted and not pending either.
        let gained = sample.periods + u64::from(self.pending);
        let used = sample.counted + u64::from(sample.pending);
        self.pending = sample.pending;
        if gained >= used {
            let missing = gained - used;
            self.lost += missing;
            if sample.receptive {
                self.host += missing;
            }
            self.opened = (missing, sample.receptive);
            return;
        }
        // More IRQ0s than periods: only edges the latest booking called
        // missing because the emulator had not presented them yet may arrive,
        // at this sample or the ones right after it.
        let back = used - gained;
        let late = back.min(self.opened.0);
        self.lost = self.lost.saturating_sub(late);
        if self.opened.1 {
            self.host -= late;
        }
        self.extra += back - late;
        self.opened.0 -= late;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(first_pending: bool, samples: &[(u64, u64, bool, bool)]) -> (u64, u64, u64) {
        let mut edges = Edges::new(first_pending);
        for &(periods, counted, pending, receptive) in samples {
            edges.account(Sample {
                periods,
                counted,
                pending,
                receptive,
            });
        }
        (edges.lost, edges.host, edges.extra)
    }

    #[test]
    fn every_period_counted_or_pending_loses_nothing() {
        assert_eq!(run(false, &[(1, 1, false, true); 5000]), (0, 0, 0));
        // An edge that is pending at one sample is counted at the next.
        assert_eq!(
            run(
                false,
                &[(1, 0, true, true), (0, 1, false, true), (1, 1, false, true)]
            ),
            (0, 0, 0)
        );
        // The first sample itself had one pending.
        assert_eq!(
            run(true, &[(0, 1, false, true), (1, 1, false, true)]),
            (0, 0, 0)
        );
    }

    #[test]
    fn merged_edges_are_lost_and_charged_by_receptiveness() {
        assert_eq!(run(false, &[(3, 1, false, true)]), (2, 2, 0));
        assert_eq!(run(false, &[(3, 1, false, false)]), (2, 0, 0));
        assert_eq!(
            run(
                false,
                &[
                    (4, 1, false, true),
                    (1, 1, false, true),
                    (3, 1, false, false)
                ]
            ),
            (5, 3, 0)
        );
    }

    #[test]
    fn an_edge_presented_late_is_not_a_loss() {
        // The period passed before the emulator raised its edge: missing at
        // one sample, counted at the next.
        assert_eq!(
            run(false, &[(1, 0, false, true), (0, 1, false, true)]),
            (0, 0, 0)
        );
        // If the next period merges with it instead, the loss stays.
        assert_eq!(
            run(false, &[(1, 0, false, true), (1, 1, false, true)]),
            (1, 1, 0)
        );
        // The first sample may sit on such a period: one IRQ0 without a period, once.
        assert_eq!(
            run(false, &[(0, 1, false, true), (1, 1, false, true)]),
            (0, 0, 0)
        );
        assert_eq!(
            run(false, &[(1, 1, false, true), (0, 1, false, true)]),
            (0, 0, 1)
        );
        // Two periods passed unpresented and arrive one by one.
        assert_eq!(
            run(
                false,
                &[
                    (2, 0, false, true),
                    (0, 1, false, true),
                    (0, 1, false, true)
                ]
            ),
            (0, 0, 0)
        );
        // A sample that balances closes the booking: a later arrival is extra,
        // and the caller takes it back from the host's share.
        assert_eq!(
            run(
                false,
                &[
                    (2, 0, false, true),
                    (1, 1, false, true),
                    (0, 1, false, true)
                ]
            ),
            (2, 2, 1)
        );
    }

    #[test]
    fn a_counted_tick_without_a_period_is_never_hidden_by_losses() {
        assert_eq!(
            run(false, &[(1, 1, false, true), (1, 2, false, true)]),
            (0, 0, 1)
        );
        // A net rule would cancel this doubled tick against the earlier loss.
        assert_eq!(
            run(
                false,
                &[
                    (3, 1, false, true),
                    (1, 1, false, true),
                    (1, 2, false, true)
                ]
            ),
            (2, 2, 1)
        );
        // Only the immediately preceding sample's missing edge may arrive late.
        assert_eq!(
            run(false, &[(2, 0, false, true), (0, 3, false, true)]),
            (0, 0, 1)
        );
    }
}
