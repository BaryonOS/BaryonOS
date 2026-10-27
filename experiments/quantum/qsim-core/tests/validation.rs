use qsim_core::{execute, Gate, Uninterrupted, ValidatedJob, MAX_GATES};

#[test]
fn q1_10_invalid_dimensions_and_targets_rejected_without_execution() {
    for qubits in [0, 13, u64::MAX] {
        assert_eq!(
            ValidatedJob::new(qubits, vec![], 1, 0).unwrap_err().field,
            "qubits"
        );
    }
    for target in [2, u64::MAX] {
        assert_eq!(
            ValidatedJob::new(2, vec![Gate::X { target }], 1, 0)
                .unwrap_err()
                .field,
            "target"
        );
    }
    for gate in [
        Gate::Cx {
            control: 0,
            target: 0,
        },
        Gate::Cx {
            control: 2,
            target: 0,
        },
        Gate::Cx {
            control: u64::MAX,
            target: 0,
        },
    ] {
        assert_eq!(
            ValidatedJob::new(2, vec![gate], 1, 0).unwrap_err().field,
            "control"
        );
    }
}

#[test]
fn q1_11_limits_and_integer_overflow() {
    for shots in [0, 65_537, u64::MAX] {
        assert_eq!(
            ValidatedJob::new(2, vec![], shots, 0).unwrap_err().field,
            "shots"
        );
    }
    assert!(
        ValidatedJob::new(1, vec![Gate::H { target: 0 }; MAX_GATES + 1], 1, 0)
            .unwrap_err()
            .resource_limit
    );
    // Owned capacity is bounded as well as initialized length.
    let mut overallocated = Vec::with_capacity(MAX_GATES + 1);
    overallocated.push(Gate::H { target: 0 });
    assert!(
        ValidatedJob::new(1, overallocated, 1, 0)
            .unwrap_err()
            .resource_limit
    );
}

#[test]
fn q1_17_maximum_job_and_measured_buffer_capacities() {
    let job = ValidatedJob::new(12, vec![Gate::H { target: 0 }; 4096], 65_536, 0).unwrap();
    assert_eq!(job.work_budget(), 17_633_280);
    let result = execute(&job, &mut Uninterrupted).unwrap();
    assert_eq!(result.shots_completed, 65_536);
    assert_eq!(result.counts.iter().sum::<u32>(), 65_536);
    assert!(
        result.buffer_bytes <= 512 * 1024,
        "actual owned capacities: {}",
        result.buffer_bytes
    );
    println!(
        "maximum-job owned buffer capacities: {} bytes",
        result.buffer_bytes
    );
}
