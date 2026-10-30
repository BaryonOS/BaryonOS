/*
 * Copyright (c) 2023.
 *
 * This software is free software;
 *
 * You can redistribute it or modify it under terms of the MIT, Apache License or Zlib license
 */

pub fn upsample_horizontal(
    input: &[i16], _ref: &[i16], _in_near: &[i16], _scratch: &mut [i16], output: &mut [i16]
) {
    assert_eq!(
        input.len() * 2,
        output.len(),
        "Input length is not half the size of the output length"
    );
    assert!(
        output.len() > 4 && input.len() > 2,
        "Too Short of a vector, cannot upsample"
    );

    output[0] = input[0];
    output[1] = (input[0] * 3 + input[1] + 2) >> 2;

    // This code is written for speed and not readability
    //
    // The readable code is
    //
    //      for i in 1..input.len() - 1{
    //         let sample = 3 * input[i] + 2;
    //         out[i * 2] = (sample + input[i - 1]) >> 2;
    //         out[i * 2 + 1] = (sample + input[i + 1]) >> 2;
    //     }
    //
    // The output of a pixel is determined by it's surrounding neighbours but we attach more weight to it's nearest
    // neighbour (input[i]) than to the next nearest neighbour.

    for (output_window, input_window) in output[2..].chunks_exact_mut(2).zip(input.windows(3)) {
        let sample = 3 * input_window[1] + 2;

        output_window[0] = (sample + input_window[0]) >> 2;
        output_window[1] = (sample + input_window[2]) >> 2;
    }
    // Get lengths
    let out_len = output.len() - 2;
    let input_len = input.len() - 2;

    // slice the output vector
    let f_out = &mut output[out_len..];
    let i_last = &input[input_len..];

    // write out manually..
    f_out[0] = (3 * i_last[1] + i_last[0] + 2) >> 2;
    f_out[1] = i_last[1];
}
pub fn upsample_vertical(
    input: &[i16], in_near: &[i16], in_far: &[i16], _scratch_space: &mut [i16], output: &mut [i16]
) {
    assert_eq!(input.len() * 2, output.len());
    assert_eq!(in_near.len(), input.len());
    assert_eq!(in_far.len(), input.len());

    let middle = output.len() / 2;

    let (out_top, out_bottom) = output.split_at_mut(middle);

    // for the first row, closest row is in_near
    for ((near, far), x) in input.iter().zip(in_near.iter()).zip(out_top) {
        *x = (((3 * near) + 2) + far) >> 2;
    }
    // for the second row, the closest row to input is in_far
    for ((near, far), x) in input.iter().zip(in_far.iter()).zip(out_bottom) {
        *x = (((3 * near) + 2) + far) >> 2;
    }
}

pub fn upsample_hv(
    input: &[i16],
    in_near: &[i16],
    in_far: &[i16],
    scratch_space: &mut [i16],
    output: &mut [i16],
) {
    assert!(!input.is_empty());
    assert_eq!(input.len(), in_near.len());
    assert_eq!(input.len(), in_far.len());
    assert_eq!(input.len() * 4, output.len());
    assert!(input.len() * 2 <= scratch_space.len());

    // Nieuw OS local correction, not an upstream-accepted change.
    // Apply the standard separable 3:1 triangle weights without rounding the
    // vertical intermediate. The combined weights are 9:3:3:1; a single final
    // shift avoids the old two-stage upward rounding bias. Alternate half-tie
    // bias 8/7 as in libjpeg-turbo 3.1.4.1 jdsample.c:h2v2_fancy_upsample
    // (commit 9217719d3a58633923b096af4c1d50d304768a64). This is an original Rust
    // expression of that mathematical interpolation, not copied C source.
    let width = input.len();
    let (top, bottom) = output.split_at_mut(width * 2);
    for (far, row) in [(in_near, top), (in_far, bottom)] {
        for (x, pair) in row.chunks_exact_mut(2).enumerate() {
            let previous = x.saturating_sub(1);
            let next = (x + 1).min(width - 1);
            let middle = 3 * i32::from(input[x]) + i32::from(far[x]);
            let left = 3 * i32::from(input[previous]) + i32::from(far[previous]);
            let right = 3 * i32::from(input[next]) + i32::from(far[next]);
            pair[0] = ((3 * middle + left + 8) >> 4) as i16;
            pair[1] = ((3 * middle + right + 7) >> 4) as i16;
        }
    }
}

pub fn upsample_generic(
    input: &[i16], _in_near: &[i16], _in_far: &[i16], _scratch_space: &mut [i16],
    output: &mut [i16]
) {
    // use nearest sample
    let difference = output.len() / input.len();
    if difference > 0 {
        // nearest neighbour
        for (input, chunk_output) in input.iter().zip(output.chunks_exact_mut(difference)) {
            for x in chunk_output.iter_mut() { *x = *input; }
        }
    }
}
