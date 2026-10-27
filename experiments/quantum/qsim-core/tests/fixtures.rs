use qsim_core::{execute, Gate, Uninterrupted, ValidatedJob};

#[test]
fn q1_07_bell8_q1_08_bell4096_and_q1_09_repeatability() {
    for (shots, expected) in [(8, [5, 0, 0, 3]), (4096, [2082, 0, 0, 2014])] {
        let job = ValidatedJob::new(
            2,
            vec![
                Gate::H { target: 0 },
                Gate::Cx {
                    control: 0,
                    target: 1,
                },
            ],
            shots,
            0,
        )
        .unwrap();
        let first = execute(&job, &mut Uninterrupted).unwrap();
        assert_eq!(first.counts, expected);
        assert_eq!(first, execute(&job, &mut Uninterrupted).unwrap());
    }
}
