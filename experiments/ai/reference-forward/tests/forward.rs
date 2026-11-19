use baryon_ai_reference_forward::{decode_f32, forward, Dense, Error, MAX_MACS};

#[test]
fn two_layer_forward_uses_bias_relu_and_row_major_weights() {
    let first = Dense::new(3, 2, &[1., 2., -1., 1., 0., 3.], &[0.5, 0., 4.]).unwrap();
    let second = Dense::new(2, 3, &[2., 5., -1., -2., 3., 4.], &[1., -1.]).unwrap();
    let mut output = [99.; 2];
    forward(&first, &second, &[2., -1.], &mut [0.; 5], &mut output).unwrap();
    assert_eq!(output, [1., 2.]);
}

#[test]
fn shape_and_work_are_bounded_before_computation() {
    for (r, c, w, b) in [
        (0, 1, &[][..], &[][..]),
        (1, 2, &[1.][..], &[0.][..]),
        (usize::MAX, 2, &[], &[]),
    ] {
        assert!(matches!(Dense::new(r, c, w, b), Err(Error::Shape)));
    }
    assert!(matches!(
        Dense::new(MAX_MACS + 1, 1, &[], &[]),
        Err(Error::WorkLimit)
    ));
    let a = Dense::new(1, 1, &[1.], &[0.]).unwrap();
    let mut output = [77.];
    for (input, scratch) in [(&[][..], &mut [0.; 2][..]), (&[1.][..], &mut [0.; 1][..])] {
        assert_eq!(
            forward(&a, &a, input, scratch, &mut output),
            Err(Error::Shape)
        );
        assert_eq!(output, [77.]);
    }
    let weights = vec![0.; 1024 * 1024];
    let bias = [0.; 1024];
    let large = Dense::new(1024, 1024, &weights, &bias).unwrap();
    let mut output = [77.; 1024];
    assert_eq!(
        forward(&large, &large, &[0.; 1024], &mut [0.; 2048], &mut output),
        Err(Error::WorkLimit)
    );
    assert_eq!(output, [77.; 1024]);
}

#[test]
fn nonfinite_inputs_and_late_overflow_never_publish_partial_output() {
    for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(matches!(
            Dense::new(1, 1, &[value], &[0.]),
            Err(Error::NonFinite)
        ));
        assert!(matches!(
            Dense::new(1, 1, &[1.], &[value]),
            Err(Error::NonFinite)
        ));
        let a = Dense::new(1, 1, &[1.], &[0.]).unwrap();
        let mut output = [77.];
        assert_eq!(
            forward(&a, &a, &[value], &mut [0.; 2], &mut output),
            Err(Error::NonFinite)
        );
        assert_eq!(output, [77.]);
    }
    let a = Dense::new(1, 1, &[1.], &[0.]).unwrap();
    let b = Dense::new(2, 1, &[1., f32::MAX], &[0., 0.]).unwrap();
    let mut output = [77.; 2];
    assert_eq!(
        forward(&a, &b, &[2.], &mut [0.; 3], &mut output),
        Err(Error::NonFinite)
    );
    assert_eq!(output, [77.; 2]);
}

#[test]
fn tensor_decode_is_exact_little_endian_and_transactional() {
    let mut output = [99.; 2];
    decode_f32(&[0, 0, 128, 63, 0, 0, 0, 192], &mut output).unwrap();
    assert_eq!(output, [1., -2.]);
    for bytes in [&[0, 0, 128, 63, 0, 0, 128, 127][..], &[0; 7], &[0; 9]] {
        let before = output;
        assert!(decode_f32(bytes, &mut output).is_err());
        assert_eq!(output, before);
    }
}

#[test]
fn small_shapes_match_an_independent_f64_oracle() {
    for columns in 1..=5 {
        for hidden in 1..=5 {
            for rows in 1..=3 {
                let values = |n: usize| {
                    (0..n)
                        .map(|i| (i as f32 % 7. - 3.) / 4.)
                        .collect::<Vec<_>>()
                };
                let input = values(columns);
                let w1 = values(hidden * columns);
                let b1 = values(hidden);
                let w2 = values(rows * hidden);
                let b2 = values(rows);
                let first = Dense::new(hidden, columns, &w1, &b1).unwrap();
                let second = Dense::new(rows, hidden, &w2, &b2).unwrap();
                let mut output = vec![99.; rows];
                forward(
                    &first,
                    &second,
                    &input,
                    &mut vec![0.; hidden + rows],
                    &mut output,
                )
                .unwrap();
                let reference_hidden = (0..hidden)
                    .map(|r| {
                        (f64::from(b1[r])
                            + (0..columns)
                                .map(|c| f64::from(w1[r * columns + c]) * f64::from(input[c]))
                                .sum::<f64>())
                        .max(0.)
                    })
                    .collect::<Vec<_>>();
                for (r, actual) in output.iter().enumerate() {
                    let expected = f64::from(b2[r])
                        + (0..hidden)
                            .map(|c| f64::from(w2[r * hidden + c]) * reference_hidden[c])
                            .sum::<f64>();
                    assert!((f64::from(*actual) - expected).abs() <= 1e-5 * expected.abs().max(1.));
                }
            }
        }
    }
}

#[test]
fn rms_norm_matches_f64_and_preserves_output_on_errors() {
    use baryon_ai_reference_forward::rms_norm;
    let input = [1., -2., 3., 0.];
    let weights = [1., 2., 0.5, -1.];
    let mut output = [77.; 4];
    rms_norm(&input, &weights, 0.01, &mut [0.; 4], &mut output).unwrap();
    let denominator = (14.0_f64 / 4. + 0.01).sqrt();
    for (i, actual) in output.iter().enumerate() {
        assert!(
            (f64::from(*actual) - f64::from(input[i]) / denominator * f64::from(weights[i])).abs()
                < 1e-6
        );
    }
    for epsilon in [0., -1., f32::NAN, f32::INFINITY] {
        let before = output;
        assert!(rms_norm(&input, &weights, epsilon, &mut [0.; 4], &mut output).is_err());
        assert_eq!(output, before);
    }
    let mut output = [77.];
    assert_eq!(
        rms_norm(&[f32::MAX], &[1.], 0.01, &mut [0.], &mut output),
        Err(Error::NonFinite)
    );
    assert_eq!(output, [77.]);
}

#[test]
fn swiglu_matches_f64_for_positive_negative_and_extreme_gates() {
    use baryon_ai_reference_forward::swi_glu;
    let gate = Dense::new(2, 1, &[1., -1.], &[0., 0.]).unwrap();
    let up = Dense::new(2, 1, &[0., 0.], &[2., 3.]).unwrap();
    let down = Dense::new(1, 2, &[1., 2.], &[0.5]).unwrap();
    for x in [-1000., -2., 0., 2., 1000.] {
        let mut output = [77.];
        swi_glu(&gate, &up, &down, &[x], &mut [0.; 5], &mut output).unwrap();
        let silu = |x: f64| x / (1. + (-x).exp());
        let expected = silu(f64::from(x)) * 2. + silu(-f64::from(x)) * 6. + 0.5;
        assert!((f64::from(output[0]) - expected).abs() < 1e-4);
    }
    let gate = Dense::new(1, 1, &[1.], &[0.]).unwrap();
    let up = Dense::new(1, 1, &[0.], &[f32::MAX]).unwrap();
    let down = Dense::new(1, 1, &[1.], &[0.]).unwrap();
    let mut output = [77.];
    assert_eq!(
        swi_glu(&gate, &up, &down, &[2.], &mut [0.; 3], &mut output),
        Err(Error::NonFinite)
    );
    assert_eq!(output, [77.]);
}
