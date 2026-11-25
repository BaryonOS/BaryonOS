//! One Qwen2-style decoder layer over borrowed BF16 weights.

use crate::{attention, rope, Budget, Error, KvCache};

/// Widen one little-endian bfloat16 to f32. Lossless: every bf16 value is an f32.
fn bf16(bytes: &[u8], index: usize) -> f32 {
    f32::from_bits(u32::from(u16::from_le_bytes([bytes[2 * index], bytes[2 * index + 1]])) << 16)
}

/// Borrowed row-major little-endian bfloat16 tensor, read in place. Weights stay
/// BF16 in memory and each element is widened to f32 when used: no quantization
/// and no second copy of the model.
#[derive(Clone, Copy)]
pub struct Bf16<'a> {
    rows: usize,
    columns: usize,
    bytes: &'a [u8],
}

impl<'a> Bf16<'a> {
    /// Validate the shape and that every value is finite.
    pub fn new(rows: usize, columns: usize, bytes: &'a [u8]) -> Result<Self, Error> {
        let len = rows
            .checked_mul(columns)
            .and_then(|n| n.checked_mul(2))
            .ok_or(Error::Shape)?;
        if rows == 0 || columns == 0 || bytes.len() != len {
            return Err(Error::Shape);
        }
        if (0..rows * columns).any(|i| !bf16(bytes, i).is_finite()) {
            return Err(Error::NonFinite);
        }
        Ok(Self {
            rows,
            columns,
            bytes,
        })
    }

    fn len(&self) -> usize {
        self.rows * self.columns
    }

    /// `output = self × input (+ bias)`, charging the budget and polling
    /// cancellation once per row.
    fn matvec(
        &self,
        input: &[f32],
        bias: Option<&Bf16<'_>>,
        output: &mut [f32],
        budget: &mut Budget<'_>,
    ) -> Result<(), Error> {
        if input.len() != self.columns
            || output.len() != self.rows
            || bias.is_some_and(|b| b.len() != self.rows)
        {
            return Err(Error::Shape);
        }
        for (row, result) in output.iter_mut().enumerate() {
            budget.charge(self.columns as u64)?;
            let mut sum = bias.map_or(0.0, |b| bf16(b.bytes, row));
            let base = row * self.columns;
            for (column, &value) in input.iter().enumerate() {
                sum += bf16(self.bytes, base + column) * value;
            }
            if !sum.is_finite() {
                return Err(Error::NonFinite);
            }
            *result = sum;
        }
        Ok(())
    }
}

/// Shape of one Qwen2-style decoder layer.
#[derive(Clone, Copy, Debug)]
pub struct LayerConfig {
    pub hidden: usize,
    pub heads: usize,
    pub kv_heads: usize,
    pub head_dim: usize,
    pub intermediate: usize,
    pub rms_epsilon: f32,
    pub rope_theta: f32,
}

impl LayerConfig {
    fn validate(&self) -> Result<(), Error> {
        if self.heads.checked_mul(self.head_dim) != Some(self.hidden)
            || self.hidden == 0
            || self.kv_heads == 0
            || !self.heads.is_multiple_of(self.kv_heads)
            || self.intermediate == 0
        {
            return Err(Error::Shape);
        }
        if !self.rms_epsilon.is_finite() || self.rms_epsilon <= 0.0 {
            return Err(Error::NonFinite);
        }
        Ok(())
    }

    fn kv_width(&self) -> usize {
        self.kv_heads * self.head_dim
    }

    /// Scratch values `DecoderLayer::forward` needs when the cache will hold
    /// `tokens` entries after this step.
    pub fn scratch_len(&self, tokens: usize) -> Option<usize> {
        self.hidden
            .checked_mul(8)?
            .checked_add(self.kv_heads.checked_mul(self.head_dim)?.checked_mul(3)?)?
            .checked_add(self.intermediate.checked_mul(2)?)?
            .checked_add(tokens)
    }
}

/// Borrowed BF16 weights of one decoder layer. Q/K/V carry biases; O and the
/// MLP projections do not, as in Qwen2.
pub struct DecoderLayer<'a> {
    pub config: LayerConfig,
    pub input_norm: Bf16<'a>,
    pub q: Bf16<'a>,
    pub q_bias: Bf16<'a>,
    pub k: Bf16<'a>,
    pub k_bias: Bf16<'a>,
    pub v: Bf16<'a>,
    pub v_bias: Bf16<'a>,
    pub o: Bf16<'a>,
    pub post_norm: Bf16<'a>,
    pub gate: Bf16<'a>,
    pub up: Bf16<'a>,
    pub down: Bf16<'a>,
}

fn rms_norm(
    input: &[f32],
    weight: &Bf16<'_>,
    epsilon: f32,
    output: &mut [f32],
    budget: &mut Budget<'_>,
) -> Result<(), Error> {
    if weight.len() != input.len() || output.len() != input.len() {
        return Err(Error::Shape);
    }
    budget.charge(2 * input.len() as u64)?;
    let mut squares = 0.0;
    for x in input {
        squares += x * x;
    }
    let mean = squares / input.len() as f32 + epsilon;
    if !mean.is_finite() {
        return Err(Error::NonFinite);
    }
    let denominator = libm::sqrtf(mean);
    for (i, (result, x)) in output.iter_mut().zip(input).enumerate() {
        *result = (x / denominator) * bf16(weight.bytes, i);
        if !result.is_finite() {
            return Err(Error::NonFinite);
        }
    }
    Ok(())
}

fn take<'s>(rest: &mut &'s mut [f32], n: usize) -> &'s mut [f32] {
    let (head, tail) = core::mem::take(rest).split_at_mut(n);
    *rest = tail;
    head
}

impl DecoderLayer<'_> {
    /// Process one token at `position`: pre-norm attention with RoPE and this
    /// layer's KV cache, residual, pre-norm SwiGLU MLP, residual. `hidden` is
    /// updated and the token's K/V kept only on success; on any error the cache
    /// is rolled back and `hidden` is unchanged. A full cache is refused with
    /// `ContextLimit` before any work.
    pub fn forward(
        &self,
        hidden: &mut [f32],
        position: u32,
        cache: &mut KvCache<'_>,
        scratch: &mut [f32],
        budget: &mut Budget<'_>,
    ) -> Result<(), Error> {
        let before = cache.len();
        let result = self.step(hidden, position, cache, scratch, budget);
        if result.is_err() {
            cache.truncate(before);
        }
        result
    }

    fn step(
        &self,
        hidden: &mut [f32],
        position: u32,
        cache: &mut KvCache<'_>,
        scratch: &mut [f32],
        budget: &mut Budget<'_>,
    ) -> Result<(), Error> {
        let c = self.config;
        c.validate()?;
        let (h, kv, d) = (c.hidden, c.kv_width(), c.head_dim);
        if hidden.len() != h || cache.kv_heads != c.kv_heads || cache.head_dim != d {
            return Err(Error::Shape);
        }
        if hidden.iter().any(|x| !x.is_finite()) {
            return Err(Error::NonFinite);
        }
        if cache.len() == cache.capacity() {
            return Err(Error::ContextLimit);
        }
        let tokens = cache.len() + 1;
        if Some(scratch.len()) != c.scratch_len(tokens) {
            return Err(Error::Shape);
        }
        let mut rest = scratch;
        let normed = take(&mut rest, h);
        let q = take(&mut rest, h);
        let q_rot = take(&mut rest, h);
        let k = take(&mut rest, kv);
        let k_rot = take(&mut rest, kv);
        let v = take(&mut rest, kv);
        let attn_scratch = take(&mut rest, tokens + h);
        let attn = take(&mut rest, h);
        let residual = take(&mut rest, h);
        let normed2 = take(&mut rest, h);
        let gate = take(&mut rest, c.intermediate);
        let up = take(&mut rest, c.intermediate);
        let down = take(&mut rest, h);

        rms_norm(hidden, &self.input_norm, c.rms_epsilon, normed, budget)?;
        self.q.matvec(normed, Some(&self.q_bias), q, budget)?;
        self.k.matvec(normed, Some(&self.k_bias), k, budget)?;
        self.v.matvec(normed, Some(&self.v_bias), v, budget)?;
        for (src, dst) in q.chunks_exact(d).zip(q_rot.chunks_exact_mut(d)) {
            rope(src, position, c.rope_theta, dst)?;
        }
        for (src, dst) in k.chunks_exact(d).zip(k_rot.chunks_exact_mut(d)) {
            rope(src, position, c.rope_theta, dst)?;
        }
        cache.append(k_rot, v)?;
        attention(q_rot, c.heads, cache, attn_scratch, attn, budget)?;
        self.o.matvec(attn, None, residual, budget)?;
        for (r, x) in residual.iter_mut().zip(hidden.iter()) {
            *r += x;
        }
        rms_norm(residual, &self.post_norm, c.rms_epsilon, normed2, budget)?;
        self.gate.matvec(normed2, None, gate, budget)?;
        self.up.matvec(normed2, None, up, budget)?;
        for (g, u) in gate.iter_mut().zip(up.iter()) {
            let silu = if *g >= 0.0 {
                *g / (1.0 + libm::expf(-*g))
            } else {
                let e = libm::expf(*g);
                (*g * e) / (1.0 + e)
            };
            *g = silu * u;
        }
        self.down.matvec(gate, None, down, budget)?;
        for (out, r) in down.iter_mut().zip(residual.iter()) {
            *out += r;
            if !out.is_finite() {
                return Err(Error::NonFinite);
            }
        }
        hidden.copy_from_slice(down);
        Ok(())
    }
}

/// A full Qwen2-style causal model: token embedding, decoder layers, final
/// RMSNorm and an output head tied to the embedding.
pub struct Model<'a, 'l> {
    pub embedding: Bf16<'a>,
    pub layers: &'l [DecoderLayer<'a>],
    pub norm: Bf16<'a>,
}

impl Model<'_, '_> {
    /// Run token `id` at `position` through every layer, leaving the final
    /// hidden state (before the final norm) in `hidden`. Each layer has its own
    /// cache in `caches`. On any error every cache is rolled back to its length
    /// before the call and `hidden` holds unspecified intermediate values.
    pub fn step(
        &self,
        id: u32,
        position: u32,
        caches: &mut [KvCache<'_>],
        hidden: &mut [f32],
        scratch: &mut [f32],
        budget: &mut Budget<'_>,
    ) -> Result<(), Error> {
        let id = id as usize;
        if caches.len() != self.layers.len()
            || self.layers.is_empty()
            || id >= self.embedding.rows
            || hidden.len() != self.embedding.columns
            || caches.iter().any(|c| c.len() != caches[0].len())
        {
            return Err(Error::Shape);
        }
        let before = caches[0].len();
        let base = id * self.embedding.columns;
        for (i, value) in hidden.iter_mut().enumerate() {
            *value = bf16(self.embedding.bytes, base + i);
        }
        for (layer, cache) in self.layers.iter().zip(caches.iter_mut()) {
            if let Err(error) = layer.forward(hidden, position, cache, scratch, budget) {
                for cache in caches.iter_mut() {
                    cache.truncate(before);
                }
                return Err(error);
            }
        }
        Ok(())
    }

    /// Final norm and tied output head: `logits = embedding × norm(hidden)`.
    pub fn logits(
        &self,
        hidden: &[f32],
        epsilon: f32,
        normed: &mut [f32],
        logits: &mut [f32],
        budget: &mut Budget<'_>,
    ) -> Result<(), Error> {
        rms_norm(hidden, &self.norm, epsilon, normed, budget)?;
        self.embedding.matvec(normed, None, logits, budget)
    }
}
