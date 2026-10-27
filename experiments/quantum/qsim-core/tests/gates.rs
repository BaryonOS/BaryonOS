use qsim_core::{execute, Gate, Uninterrupted, ValidatedJob};

#[test]
fn q1_01_identity_and_q1_02_little_endian_bit_order() {
    let identity = ValidatedJob::new(3, vec![], 17, 0).unwrap();
    assert_eq!(
        execute(&identity, &mut Uninterrupted).unwrap().counts,
        [17, 0, 0, 0, 0, 0, 0, 0]
    );
    let x = ValidatedJob::new(2, vec![Gate::X { target: 1 }], 17, 0).unwrap();
    assert_eq!(
        execute(&x, &mut Uninterrupted).unwrap().counts,
        [0, 0, 17, 0]
    );
}

#[test]
fn control_direction_and_zero_probability_bins() {
    for (control, target) in [(0, 1), (1, 0)] {
        let job = ValidatedJob::new(
            2,
            vec![Gate::X { target: control }, Gate::Cx { control, target }],
            1024,
            u64::MAX,
        )
        .unwrap();
        assert_eq!(
            execute(&job, &mut Uninterrupted).unwrap().counts,
            [0, 0, 0, 1024]
        );
    }
}
