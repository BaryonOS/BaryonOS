use qsim_core::{execute, Abort, Control, ExecuteError, Gate, ValidatedJob};

struct StopAt {
    calls: usize,
    stop_at: usize,
    reason: Abort,
    previous_work: u64,
    max_gap: u64,
}
impl Control for StopAt {
    fn checkpoint(&mut self, work: u64) -> Result<(), Abort> {
        assert!(work >= self.previous_work);
        self.max_gap = self.max_gap.max(work - self.previous_work);
        self.previous_work = work;
        self.calls += 1;
        if self.calls >= self.stop_at {
            Err(self.reason)
        } else {
            Ok(())
        }
    }
}

#[test]
fn q1_13_cancel_before_allocation_during_gate_and_during_sampling() {
    let job = ValidatedJob::new(12, vec![Gate::H { target: 0 }; 10], 65_536, 0).unwrap();
    // 1: before allocation; 25: within gate loops; 240: within sampling.
    for stop_at in [1, 25, 240] {
        let mut control = StopAt {
            calls: 0,
            stop_at,
            reason: Abort::Cancelled,
            previous_work: 0,
            max_gap: 0,
        };
        assert_eq!(
            execute(&job, &mut control),
            Err(ExecuteError::Aborted(Abort::Cancelled))
        );
        assert_eq!(control.calls, stop_at);
        assert!(control.max_gap <= 64 * 13);
    }
}

#[test]
fn q1_14_deadline_is_deterministic_and_never_returns_partial_result() {
    let job = ValidatedJob::new(12, vec![Gate::H { target: 0 }], 65_536, 0).unwrap();
    let mut control = StopAt {
        calls: 0,
        stop_at: 26,
        reason: Abort::DeadlineExceeded,
        previous_work: 0,
        max_gap: 0,
    };
    assert_eq!(
        execute(&job, &mut control),
        Err(ExecuteError::Aborted(Abort::DeadlineExceeded))
    );
}

#[test]
fn gate_and_sampling_checkpoint_intervals_are_bounded() {
    let job = ValidatedJob::new(12, vec![Gate::H { target: 0 }], 128, 0).unwrap();
    let mut control = StopAt {
        calls: 0,
        stop_at: usize::MAX,
        reason: Abort::Cancelled,
        previous_work: 0,
        max_gap: 0,
    };
    execute(&job, &mut control).unwrap();
    assert!(control.max_gap <= 832);
}

#[test]
fn basis_loops_and_sampling_have_separate_checkpoint_bounds() {
    struct CheckIntervals {
        previous_work: u64,
        sampling_start: u64,
        basis_max: u64,
        sampling_max: u64,
    }
    impl Control for CheckIntervals {
        fn checkpoint(&mut self, work: u64) -> Result<(), Abort> {
            let gap = work.checked_sub(self.previous_work).unwrap();
            if self.previous_work < self.sampling_start {
                self.basis_max = self.basis_max.max(gap);
                assert!(gap <= 256, "basis loop checkpoint gap: {gap}");
            } else {
                self.sampling_max = self.sampling_max.max(gap);
                assert!(gap <= 64 * 13, "sampling checkpoint gap: {gap}");
            }
            self.previous_work = work;
            Ok(())
        }
    }
    let job = ValidatedJob::new(12, vec![Gate::H { target: 0 }; 2], 129, 0).unwrap();
    let mut control = CheckIntervals {
        previous_work: 0,
        // Initialization + gates + probability/CDF passes + counter allocation.
        sampling_start: (2 + 4) * 4096,
        basis_max: 0,
        sampling_max: 0,
    };
    execute(&job, &mut control).unwrap();
    assert_eq!(control.basis_max, 256);
    assert_eq!(control.sampling_max, 64 * 13);
    assert_eq!(control.previous_work, control.sampling_start + 129 * 13);
}
