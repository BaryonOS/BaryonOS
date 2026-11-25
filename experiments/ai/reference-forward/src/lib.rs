#![no_std]
#![forbid(unsafe_code)]
//! Host numerical reference only; no model loader, permissions or native inference.

mod decoder;
pub use decoder::{Bf16, DecoderLayer, LayerConfig};

pub const MAX_MACS: usize = 1_048_576;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Shape,
    WorkLimit,
    NonFinite,
    /// The fixed context or KV ceiling is exhausted; nothing was truncated.
    ContextLimit,
    /// The caller's cancellation signal was observed; no output was published.
    Cancelled,
}

/// Cooperative cancellation, polled between rows, heads and layers.
pub trait Cancel {
    fn cancelled(&self) -> bool;
}

/// Never cancels; for tests and callers without a cancellation source.
pub struct NoCancel;

impl Cancel for NoCancel {
    fn cancelled(&self) -> bool {
        false
    }
}

/// Per-request multiply/accumulate budget. Work is charged before it is done,
/// so an exhausted budget stops the request without partial output.
pub struct Budget<'c> {
    remaining: u64,
    cancel: &'c dyn Cancel,
}

impl<'c> Budget<'c> {
    pub fn new(macs: u64, cancel: &'c dyn Cancel) -> Self {
        Self {
            remaining: macs,
            cancel,
        }
    }

    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Check cancellation, then reserve `macs` of work.
    pub fn charge(&mut self, macs: u64) -> Result<(), Error> {
        if self.cancel.cancelled() {
            return Err(Error::Cancelled);
        }
        self.remaining = self.remaining.checked_sub(macs).ok_or(Error::WorkLimit)?;
        Ok(())
    }
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

/// Proposed L01 KV ceiling (LOCAL-AI-L01-PROPOSAL.md); not a measured requirement.
pub const MAX_KV_BYTES: usize = 128 * 1024 * 1024;

/// `2 × layers × kv_heads × head_dim × tokens × bytes_per_element`, checked before
/// any allocation. Overflow is `Shape`; exceeding `MAX_KV_BYTES` is `ContextLimit`.
pub fn kv_cache_bytes(
    layers: usize,
    kv_heads: usize,
    head_dim: usize,
    tokens: usize,
    bytes_per_element: usize,
) -> Result<usize, Error> {
    let factors = [layers, kv_heads, head_dim, tokens, bytes_per_element];
    if factors.contains(&0) {
        return Err(Error::Shape);
    }
    let bytes = factors
        .iter()
        .try_fold(2usize, |acc, &n| acc.checked_mul(n))
        .ok_or(Error::Shape)?;
    if bytes > MAX_KV_BYTES {
        return Err(Error::ContextLimit);
    }
    Ok(bytes)
}

/// Rotary position embedding for one head in the rotate-half layout used by
/// Llama/Qwen2-style checkpoints: element `i` pairs with `i + head_dim / 2`, with
/// inverse frequency `theta^(-2i / head_dim)`. Output is committed only on success.
pub fn rope(input: &[f32], position: u32, theta: f32, output: &mut [f32]) -> Result<(), Error> {
    let dim = input.len();
    if dim == 0 || !dim.is_multiple_of(2) || output.len() != dim || dim > MAX_MACS {
        return Err(Error::Shape);
    }
    if !theta.is_finite() || theta <= 1.0 || input.iter().any(|x| !x.is_finite()) {
        return Err(Error::NonFinite);
    }
    let half = dim / 2;
    // Validate every rotated value before publishing any of them.
    let rotated = |i: usize| -> Result<(f32, f32), Error> {
        let inv_freq = 1.0 / libm::powf(theta, (2 * i) as f32 / dim as f32);
        let angle = position as f32 * inv_freq;
        let (sin, cos) = (libm::sinf(angle), libm::cosf(angle));
        let (a, b) = (input[i], input[i + half]);
        let pair = (a * cos - b * sin, b * cos + a * sin);
        if pair.0.is_finite() && pair.1.is_finite() {
            Ok(pair)
        } else {
            Err(Error::NonFinite)
        }
    };
    for i in 0..half {
        rotated(i)?;
    }
    for i in 0..half {
        (output[i], output[i + half]) = rotated(i)?;
    }
    Ok(())
}

/// Fixed-capacity per-layer KV cache over caller-owned buffers. No allocation;
/// a full cache refuses further tokens instead of evicting or truncating.
pub struct KvCache<'a> {
    keys: &'a mut [f32],
    values: &'a mut [f32],
    kv_heads: usize,
    head_dim: usize,
    len: usize,
}

impl<'a> KvCache<'a> {
    pub fn new(
        kv_heads: usize,
        head_dim: usize,
        keys: &'a mut [f32],
        values: &'a mut [f32],
    ) -> Result<Self, Error> {
        let row = kv_heads.checked_mul(head_dim).ok_or(Error::Shape)?;
        if row == 0
            || keys.len() != values.len()
            || keys.is_empty()
            || !keys.len().is_multiple_of(row)
        {
            return Err(Error::Shape);
        }
        Ok(Self {
            keys,
            values,
            kv_heads,
            head_dim,
            len: 0,
        })
    }

    fn row(&self) -> usize {
        self.kv_heads * self.head_dim
    }

    pub fn capacity(&self) -> usize {
        self.keys.len() / self.row()
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Restore an earlier length, used to roll back a failed layer step.
    fn truncate(&mut self, len: usize) {
        self.len = self.len.min(len);
    }

    /// Append one token's keys and values (`kv_heads × head_dim` each).
    pub fn append(&mut self, key: &[f32], value: &[f32]) -> Result<(), Error> {
        let row = self.row();
        if key.len() != row || value.len() != row {
            return Err(Error::Shape);
        }
        if key.iter().chain(value).any(|x| !x.is_finite()) {
            return Err(Error::NonFinite);
        }
        if self.len == self.capacity() {
            return Err(Error::ContextLimit);
        }
        let range = self.len * row..(self.len + 1) * row;
        self.keys[range.clone()].copy_from_slice(key);
        self.values[range].copy_from_slice(value);
        self.len += 1;
        Ok(())
    }
}

/// Causal grouped-query attention of one new query over every cached token
/// (the caller appends the current token first). `query` holds `heads × head_dim`
/// values and `heads` must be a multiple of the cache's KV heads. Scratch holds
/// `cache.len() + heads × head_dim` values; output is committed only on success.
/// The whole step (two passes per head and token) is charged to `budget` first.
pub fn attention(
    query: &[f32],
    heads: usize,
    cache: &KvCache<'_>,
    scratch: &mut [f32],
    output: &mut [f32],
    budget: &mut Budget<'_>,
) -> Result<(), Error> {
    let dim = cache.head_dim;
    let width = heads.checked_mul(dim).ok_or(Error::Shape)?;
    let tokens = cache.len;
    if heads == 0
        || !heads.is_multiple_of(cache.kv_heads)
        || query.len() != width
        || output.len() != width
        || scratch.len() != tokens.checked_add(width).ok_or(Error::Shape)?
    {
        return Err(Error::Shape);
    }
    if tokens == 0 {
        return Err(Error::Shape);
    }
    if query.iter().any(|x| !x.is_finite()) {
        return Err(Error::NonFinite);
    }
    // Scores and weighted sum: two multiply/accumulate passes per head and token.
    let work = (width as u64)
        .checked_mul(tokens as u64)
        .and_then(|n| n.checked_mul(2))
        .ok_or(Error::WorkLimit)?;
    budget.charge(work)?;
    let group = heads / cache.kv_heads;
    let row = cache.row();
    let scale = 1.0 / libm::sqrtf(dim as f32);
    let (scores, accum) = scratch.split_at_mut(tokens);
    for head in 0..heads {
        let q = &query[head * dim..(head + 1) * dim];
        let offset = (head / group) * dim;
        let mut max = f32::NEG_INFINITY;
        for (t, score) in scores.iter_mut().enumerate() {
            let k = &cache.keys[t * row + offset..t * row + offset + dim];
            let mut dot = 0.0;
            for (a, b) in q.iter().zip(k) {
                dot += a * b;
            }
            *score = dot * scale;
            if !score.is_finite() {
                return Err(Error::NonFinite);
            }
            max = max.max(*score);
        }
        let mut total = 0.0;
        for score in scores.iter_mut() {
            *score = libm::expf(*score - max);
            total += *score;
        }
        let out = &mut accum[head * dim..(head + 1) * dim];
        out.fill(0.0);
        for (t, weight) in scores.iter().enumerate() {
            let v = &cache.values[t * row + offset..t * row + offset + dim];
            for (o, x) in out.iter_mut().zip(v) {
                *o += (weight / total) * x;
            }
        }
        if out.iter().any(|x| !x.is_finite()) {
            return Err(Error::NonFinite);
        }
    }
    output.copy_from_slice(accum);
    Ok(())
}
