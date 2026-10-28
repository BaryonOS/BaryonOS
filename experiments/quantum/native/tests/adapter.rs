use qsim_native::{execute, Abort, Control, Error, ExecuteError, Gate, MAX_GATES, MAX_SHOTS};

struct CheckedControl {
    calls: usize,
    abort_at: Option<(usize, Abort)>,
    previous: u64,
    max_gap: u64,
}
impl CheckedControl {
    fn new() -> Self {
        Self {
            calls: 0,
            abort_at: None,
            previous: 0,
            max_gap: 0,
        }
    }
}
impl Control for CheckedControl {
    fn checkpoint(&mut self, work: u64) -> Result<(), Abort> {
        assert!(work >= self.previous);
        self.max_gap = self.max_gap.max(work - self.previous);
        self.previous = work;
        self.calls += 1;
        if let Some((call, reason)) = self.abort_at {
            if self.calls >= call {
                return Err(reason);
            }
        }
        Ok(())
    }
}

#[test]
fn unchanged_bell_fixture_has_exact_distribution_and_zero_tail() {
    let gates = [
        Gate::H { target: 0 },
        Gate::Cx {
            control: 0,
            target: 1,
        },
    ];
    let mut control = CheckedControl::new();
    let result = execute(2, &gates, 4096, 0, &mut control).unwrap();
    assert_eq!(result.bins, 4);
    assert_eq!(result.shots_completed, 4096);
    assert_eq!(&result.counts[..4], &[2082, 0, 0, 2014]);
    assert!(result.counts[4..].iter().all(|count| *count == 0));
    assert!(result.buffer_bytes < 4096);
    assert!(control.max_gap <= 64 * 3);
}

#[test]
fn six_qubit_boundary_runs_the_existing_core() {
    let gates = core::array::from_fn::<_, 6, _>(|target| Gate::X {
        target: target as u64,
    });
    let result = execute(6, &gates, MAX_SHOTS, u64::MAX, &mut CheckedControl::new()).unwrap();
    assert_eq!(result.bins, 64);
    assert_eq!(result.counts[63], MAX_SHOTS as u32);
    assert!(result.counts[..63].iter().all(|count| *count == 0));
    assert!(result.buffer_bytes < 4096);
}

#[test]
fn native_limits_and_gate_errors_precede_execution() {
    let mut control = CheckedControl::new();
    for qubits in [0, 7, u64::MAX] {
        assert_eq!(
            execute(qubits, &[], 1, 0, &mut control),
            Err(Error::QubitLimit)
        );
    }
    assert_eq!(
        execute(
            1,
            &[Gate::X { target: 0 }; MAX_GATES + 1],
            1,
            0,
            &mut control
        ),
        Err(Error::GateLimit)
    );
    for shots in [0, MAX_SHOTS + 1, u64::MAX] {
        assert_eq!(
            execute(1, &[], shots, 0, &mut control),
            Err(Error::ShotLimit)
        );
    }
    for gate in [
        Gate::X { target: u64::MAX },
        Gate::Cx {
            control: 0,
            target: 0,
        },
        Gate::Cx {
            control: 6,
            target: 0,
        },
    ] {
        assert!(matches!(
            execute(6, &[gate], 1, 0, &mut control),
            Err(Error::Invalid(_))
        ));
    }
    assert_eq!(control.calls, 0);
}

#[test]
fn cancel_and_deadline_do_not_return_a_partial_histogram() {
    for reason in [Abort::Cancelled, Abort::DeadlineExceeded] {
        for stop in [1, 8, 80] {
            let mut control = CheckedControl::new();
            control.abort_at = Some((stop, reason));
            assert_eq!(
                execute(6, &[Gate::H { target: 0 }; 64], 4096, 0, &mut control),
                Err(Error::Execution(ExecuteError::Aborted(reason)))
            );
            assert_eq!(control.calls, stop);
        }
    }
}

#[test]
fn maximum_native_request_is_bounded_and_repeatable() {
    let gates = [Gate::H { target: 0 }; MAX_GATES];
    let one = execute(6, &gates, MAX_SHOTS, 0, &mut CheckedControl::new()).unwrap();
    let two = execute(6, &gates, MAX_SHOTS, 0, &mut CheckedControl::new()).unwrap();
    assert_eq!(one, two);
    assert_eq!(one.counts[0], MAX_SHOTS as u32);
    assert_eq!(one.counts.iter().sum::<u32>(), MAX_SHOTS as u32);
    assert!(one.buffer_bytes < 4096);
}

#[test]
fn bump_allocation_rejects_overflow_misalignment_and_exhaustion() {
    use qsim_native::{heap_budget::allocation_range as range, HEAP_BYTES};
    assert_eq!(range(1, 16, 8, HEAP_BYTES), Some((8, 24)));
    assert_eq!(range(0, HEAP_BYTES, 8, HEAP_BYTES), Some((0, HEAP_BYTES)));
    assert_eq!(range(HEAP_BYTES, 1, 1, HEAP_BYTES), None);
    assert_eq!(range(0, usize::MAX, 8, HEAP_BYTES), None);
    assert_eq!(range(usize::MAX, 1, 8, HEAP_BYTES), None);
    assert_eq!(range(0, 1, 0, HEAP_BYTES), None);
    assert_eq!(range(0, 1, 3, HEAP_BYTES), None);
    assert_eq!(range(0, 1, HEAP_BYTES * 2, HEAP_BYTES), None);
    assert_eq!(range(0, 1, 8, HEAP_BYTES - 1), None);
    let mut used = 0;
    while let Some((start, end)) = range(used, 7, 16, HEAP_BYTES) {
        assert_eq!(start % 16, 0);
        assert!(start >= used && end <= HEAP_BYTES);
        used = end;
    }
    assert_eq!(range(used, 7, 16, HEAP_BYTES), None);
}
