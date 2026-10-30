// Exercise the actual patched production function, not a duplicated routine.
#[rustfmt::skip]
#[allow(dead_code, clippy::all)]
#[path = "../vendor/zune-jpeg-0.5.16-rc3/src/upsampler/scalar.rs"]
mod production;

#[test]
fn constants_and_maximum_samples_stay_constant_at_odd_and_tiny_widths() {
    for width in [1, 2, 3, 8, 9, 17] {
        for sample in [0, 1, 127, 128, 254, 255, i16::MAX, i16::MIN] {
            let input = vec![sample; width];
            let mut scratch = vec![0x1234; width * 2 + 17];
            let mut output = vec![0x2345; width * 4 + 17];
            production::upsample_hv(
                &input,
                &input,
                &input,
                &mut scratch,
                &mut output[..width * 4],
            );
            assert!(output[..width * 4].iter().all(|x| *x == sample));
            assert!(output[width * 4..].iter().all(|x| *x == 0x2345));
            assert!(scratch[width * 2..].iter().all(|x| *x == 0x1234));
        }
    }
}

#[test]
fn alternating_tie_bias_avoids_double_rounding_even_for_one_column() {
    let mut output = [0i16; 4];
    production::upsample_hv(&[0], &[2], &[2], &mut [0; 2], &mut output);
    assert_eq!(output, [1, 0, 1, 0]);
    production::upsample_hv(&[100], &[0], &[200], &mut [0; 2], &mut output);
    assert_eq!(output, [75, 75, 125, 125]);
}

#[test]
fn wrong_geometry_fails_before_any_destination_write() {
    use std::panic::{catch_unwind, AssertUnwindSafe};
    for (near, far, output_len, scratch_len) in
        [(1, 2, 8, 4), (2, 1, 8, 4), (2, 2, 7, 4), (2, 2, 8, 3)]
    {
        let mut output = [0x2345i16; 8];
        let mut scratch = [0x1234i16; 4];
        let result = catch_unwind(AssertUnwindSafe(|| {
            production::upsample_hv(
                &[0; 2],
                &vec![0; near],
                &vec![0; far],
                &mut scratch[..scratch_len],
                &mut output[..output_len],
            )
        }));
        assert!(result.is_err());
        assert_eq!(output, [0x2345; 8]);
        assert_eq!(scratch, [0x1234; 4]);
    }
}
