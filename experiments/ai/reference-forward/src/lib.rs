#![no_std]
#![forbid(unsafe_code)]
//! Host numerical reference only; no model loader, permissions or native inference.

pub const MAX_MACS: usize = 1_048_576;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Shape,
    WorkLimit,
    NonFinite,
}

/// Borrowed row-major weights and one bias per output row.
pub struct Dense<'a> {
    rows: usize,
    columns: usize,
    weights: &'a [f32],
    bias: &'a [f32],
}

impl<'a> Dense<'a> {
    pub fn new(
        rows: usize,
        columns: usize,
        weights: &'a [f32],
        bias: &'a [f32],
    ) -> Result<Self, Error> {
        let work = rows.checked_mul(columns).ok_or(Error::Shape)?;
        if rows == 0 || columns == 0 {
            return Err(Error::Shape);
        }
        if work > MAX_MACS {
            return Err(Error::WorkLimit);
        }
        if weights.len() != work || bias.len() != rows {
            return Err(Error::Shape);
        }
        if weights.iter().chain(bias).any(|value| !value.is_finite()) {
            return Err(Error::NonFinite);
        }
        Ok(Self {
            rows,
            columns,
            weights,
            bias,
        })
    }

    fn compute(&self, input: &[f32], scratch: &mut [f32]) -> Result<(), Error> {
        if input.iter().any(|value| !value.is_finite()) {
            return Err(Error::NonFinite);
        }
        for (row, result) in scratch.iter_mut().enumerate() {
            let mut sum = self.bias[row];
            for (&weight, &value) in self.weights[row * self.columns..(row + 1) * self.columns]
                .iter()
                .zip(input)
            {
                sum += weight * value;
                if !sum.is_finite() {
                    return Err(Error::NonFinite);
                }
            }
            *result = sum;
        }
        Ok(())
    }
}

/// Compute second(ReLU(first(input))). Output is unchanged on every error.
/// Scratch is caller-owned and may contain intermediate/partial values on error.
/// This reference uses separate f32 multiply/add, without a fused operation.
pub fn forward(
    first: &Dense<'_>,
    second: &Dense<'_>,
    input: &[f32],
    scratch: &mut [f32],
    output: &mut [f32],
) -> Result<(), Error> {
    if first.rows != second.columns
        || input.len() != first.columns
        || output.len() != second.rows
        || scratch.len() != first.rows.checked_add(second.rows).ok_or(Error::Shape)?
    {
        return Err(Error::Shape);
    }
    let work = first
        .weights
        .len()
        .checked_add(second.weights.len())
        .ok_or(Error::WorkLimit)?;
    if work > MAX_MACS {
        return Err(Error::WorkLimit);
    }
    let (hidden, final_values) = scratch.split_at_mut(first.rows);
    first.compute(input, hidden)?;
    for value in hidden.iter_mut() {
        *value = value.max(0.0);
    }
    second.compute(hidden, final_values)?;
    output.copy_from_slice(final_values);
    Ok(())
}

/// Decode exact little-endian IEEE binary32 data. No authenticity is implied.
/// Nonfinite values, extra/truncated bytes and oversized input leave output intact.
pub fn decode_f32(bytes: &[u8], output: &mut [f32]) -> Result<(), Error> {
    let expected = output.len().checked_mul(4).ok_or(Error::Shape)?;
    if output.len() > MAX_MACS {
        return Err(Error::WorkLimit);
    }
    if bytes.len() != expected {
        return Err(Error::Shape);
    }
    let (chunks, _) = bytes.as_chunks::<4>();
    if chunks
        .iter()
        .any(|chunk| !f32::from_le_bytes(*chunk).is_finite())
    {
        return Err(Error::NonFinite);
    }
    for (target, chunk) in output.iter_mut().zip(chunks) {
        *target = f32::from_le_bytes(*chunk);
    }
    Ok(())
}

/// RMS normalization with an explicit positive epsilon and finite intermediates.
/// Scratch may change on failure; output is committed only on complete success.
pub fn rms_norm(
    input: &[f32],
    weights: &[f32],
    epsilon: f32,
    scratch: &mut [f32],
    output: &mut [f32],
) -> Result<(), Error> {
    if input.is_empty()
        || input.len() != weights.len()
        || input.len() != scratch.len()
        || input.len() != output.len()
    {
        return Err(Error::Shape);
    }
    if input.len() > MAX_MACS {
        return Err(Error::WorkLimit);
    }
    if !epsilon.is_finite() || epsilon <= 0.0 || input.iter().chain(weights).any(|x| !x.is_finite())
    {
        return Err(Error::NonFinite);
    }
    let mut squares = 0.0;
    for x in input {
        squares += x * x;
    }
    let mean = squares / input.len() as f32 + epsilon;
    if !mean.is_finite() {
        return Err(Error::NonFinite);
    }
    let denominator = libm::sqrtf(mean);
    for ((result, x), weight) in scratch.iter_mut().zip(input).zip(weights) {
        *result = (x / denominator) * weight;
        if !result.is_finite() {
            return Err(Error::NonFinite);
        }
    }
    output.copy_from_slice(scratch);
    Ok(())
}

/// Compute down(SiLU(gate(input)) * up(input)), with one aggregate work bound.
/// Uses the pinned soft-float libm exponential, never an ad-hoc approximation.
pub fn swi_glu(
    gate: &Dense<'_>,
    up: &Dense<'_>,
    down: &Dense<'_>,
    input: &[f32],
    scratch: &mut [f32],
    output: &mut [f32],
) -> Result<(), Error> {
    let size = gate
        .rows
        .checked_mul(2)
        .and_then(|n| n.checked_add(down.rows))
        .ok_or(Error::Shape)?;
    if gate.columns != up.columns
        || gate.rows != up.rows
        || down.columns != gate.rows
        || input.len() != gate.columns
        || scratch.len() != size
        || output.len() != down.rows
    {
        return Err(Error::Shape);
    }
    if gate
        .weights
        .len()
        .checked_add(up.weights.len())
        .and_then(|n| n.checked_add(down.weights.len()))
        .ok_or(Error::WorkLimit)?
        > MAX_MACS
    {
        return Err(Error::WorkLimit);
    }
    let (gates, remaining) = scratch.split_at_mut(gate.rows);
    let (ups, final_values) = remaining.split_at_mut(up.rows);
    gate.compute(input, gates)?;
    up.compute(input, ups)?;
    for (g, u) in gates.iter_mut().zip(ups) {
        let silu = if *g >= 0.0 {
            *g / (1.0 + libm::expf(-*g))
        } else {
            let e = libm::expf(*g);
            (*g * e) / (1.0 + e)
        };
        *g = silu * *u;
        if !g.is_finite() {
            return Err(Error::NonFinite);
        }
    }
    down.compute(gates, final_values)?;
    output.copy_from_slice(final_values);
    Ok(())
}
