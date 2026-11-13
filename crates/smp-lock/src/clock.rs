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

/// Whether a lost edge of this sample is the host's: exactly one IRQ0 woke
/// the CPU, it handled no other interrupt since the previous sample, and each
/// span in which it could not take IRQ0 stayed under `limit`. Then the CPU sat
/// in `sti; hlt` for the rest, and two edges can only have merged because the
/// emulator presented or delivered them late.
pub fn receptive(counted: u64, others: u64, spans: [u64; 2], limit: u64) -> bool {
    counted == 1 && others == 0 && spans.iter().all(|span| *span < limit)
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

/// Outcome of the idle clock check over its segments (ADR-082 D3).
#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    /// The judged median segment, in µs.
    Pass(u64),
    /// The judged median is more than 1 % from the PIT's own period.
    Drift(u64),
    /// Most segments owe more than a tenth of their length to the host credit:
    /// the witness is not trusted that far, whatever the median says.
    HostBound(u64),
}

/// ADR-082 D3: each segment as (`own`, `host`) µs, where `own` keeps every
/// period except those the witness charged to the host and `host` is that
/// credit. The median of `own` must be within 1 % of `expected` on both sides,
/// so kernel-charged loss and doubled ticks still fail, and at most a minority
/// of segments may owe more than 10 % of their length to the host credit. A
/// stall in one or two segments cannot move the median either way, as with
/// the raw median; it must not fail through the bound either.
pub fn judge(segments: &mut [(u64, u64)], expected: u64) -> Verdict {
    let over = segments
        .iter()
        .filter(|&&(own, host)| host * 10 > own + host)
        .count();
    segments.sort_unstable();
    let (own, _) = segments[segments.len() / 2];
    if over * 2 > segments.len() {
        Verdict::HostBound(own)
    } else if own.abs_diff(expected) * 100 > expected {
        Verdict::Drift(own)
    } else {
        Verdict::Pass(own)
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
    fn only_a_halted_cpu_with_short_spans_is_receptive() {
        assert!(receptive(1, 0, [10, 20], 250));
        // No IRQ0 or several since the previous sample.
        assert!(!receptive(0, 0, [10, 20], 250));
        assert!(!receptive(2, 0, [10, 20], 250));
        // Another handler ran while the CPU should have been halted.
        assert!(!receptive(1, 1, [10, 20], 250));
        // IRQ0 could not be taken before the halt or after the wake.
        assert!(!receptive(1, 0, [250, 20], 250));
        assert!(!receptive(1, 0, [10, 250], 250));
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
    const NOMINAL: u64 = 999_847;

    #[test]
    fn the_median_without_host_credit_is_judged_on_both_sides() {
        // Host-lost periods alone, every segment: the kernel's clock is exact.
        let mut segments = [(NOMINAL, 20_000); 5];
        assert_eq!(judge(&mut segments, NOMINAL), Verdict::Pass(NOMINAL));
        // Loss charged to the kernel stays in `own`: +1.2 % fails.
        let mut segments = [(NOMINAL + 12_000, 0); 5];
        assert_eq!(
            judge(&mut segments, NOMINAL),
            Verdict::Drift(NOMINAL + 12_000)
        );
        // Doubled or synthesised ticks make segments short: -1.2 % fails.
        let mut segments = [(NOMINAL - 12_000, 0); 5];
        assert!(matches!(judge(&mut segments, NOMINAL), Verdict::Drift(_)));
        // Exactly 1 % passes, as in the raw rule.
        let mut segments = [(NOMINAL + NOMINAL / 100, 0); 5];
        assert!(matches!(judge(&mut segments, NOMINAL), Verdict::Pass(_)));
    }

    #[test]
    fn two_bad_segments_do_not_move_the_median() {
        // A long host stall and a kernel stall, each in one segment; the
        // stalled segment's own value lands on the median (review K1).
        let mut segments = [
            (NOMINAL - 1, 0),
            (NOMINAL + 500_000, 0),
            (NOMINAL, 900_000),
            (NOMINAL - 1, 0),
            (NOMINAL + 1, 10),
        ];
        assert_eq!(judge(&mut segments, NOMINAL), Verdict::Pass(NOMINAL));
        // Two host stalls are still a minority.
        let mut segments = [
            (NOMINAL, 900_000),
            (NOMINAL, 150_000),
            (NOMINAL, 0),
            (NOMINAL, 0),
            (NOMINAL, 0),
        ];
        assert_eq!(judge(&mut segments, NOMINAL), Verdict::Pass(NOMINAL));
    }

    #[test]
    fn host_credit_above_a_tenth_of_most_segments_fails() {
        // 111 112 of 1 110 959 µs credited: just above 10 %.
        let mut segments = [(NOMINAL, 111_112); 5];
        assert_eq!(judge(&mut segments, NOMINAL), Verdict::HostBound(NOMINAL));
        let mut segments = [(NOMINAL, 111_094); 5];
        assert_eq!(judge(&mut segments, NOMINAL), Verdict::Pass(NOMINAL));
        // Three of five over the bound is a majority.
        let mut segments = [
            (NOMINAL, 111_112),
            (NOMINAL, 0),
            (NOMINAL, 111_112),
            (NOMINAL, 0),
            (NOMINAL, 900_000),
        ];
        assert_eq!(judge(&mut segments, NOMINAL), Verdict::HostBound(NOMINAL));
    }
}
