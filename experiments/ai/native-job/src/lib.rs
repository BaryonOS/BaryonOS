//! Deterministic decoder fixture shared by the native SSE2 job and the host.
//! Both run the same code; the native report must match the host bit for bit.
#![no_std]
#![forbid(unsafe_code)]

use baryon_ai_reference_forward::{
    Bf16, Budget, DecoderLayer, Error, KvCache, LayerConfig, Model, NoCancel,
};

pub const CONFIG: LayerConfig = LayerConfig {
    hidden: 8,
    heads: 2,
    kv_heads: 1,
    head_dim: 4,
    intermediate: 12,
    rms_epsilon: 1e-6,
    rope_theta: 10_000.,
};
pub const VOCAB: usize = 16;
pub const LAYERS: usize = 2;
pub const TOKENS: [u32; 4] = [3, 1, 4, 1];
pub const STEPS: usize = TOKENS.len();
/// f32 output words: logits after every prompt token.
pub const OUTPUT: usize = STEPS * VOCAB;

const H: usize = CONFIG.hidden;
const KV: usize = CONFIG.kv_heads * CONFIG.head_dim;
const I: usize = CONFIG.intermediate;
/// (rows, columns) of one layer: input norm, q, q bias, k, k bias, v, v bias,
/// o, post norm, gate, up, down.
const SHAPES: [(usize, usize); 12] = [
    (1, H),
    (H, H),
    (1, H),
    (KV, H),
    (1, KV),
    (KV, H),
    (1, KV),
    (H, H),
    (1, H),
    (I, H),
    (I, H),
    (H, I),
];
const LAYER_ELEMENTS: usize = {
    let mut total = 0;
    let mut i = 0;
    while i < SHAPES.len() {
        total += SHAPES[i].0 * SHAPES[i].1;
        i += 1;
    }
    total
};
const ELEMENTS: usize = LAYERS * LAYER_ELEMENTS + VOCAB * H + H;
const SCRATCH: usize = 8 * H + 3 * KV + 2 * I + STEPS;

/// BF16 weights, exact multiples of 1/16 in [-1, 1].
pub struct Weights([u8; 2 * ELEMENTS]);

impl Weights {
    pub fn new() -> Self {
        let mut bytes = [0u8; 2 * ELEMENTS];
        for (i, pair) in bytes.as_chunks_mut::<2>().0.iter_mut().enumerate() {
            let value = ((i * 7 + 13) % 33) as f32 / 16.0 - 1.0;
            pair.copy_from_slice(&((value.to_bits() >> 16) as u16).to_le_bytes());
        }
        Self(bytes)
    }
}

impl Default for Weights {
    fn default() -> Self {
        Self::new()
    }
}

fn tensor<'a>(rest: &mut &'a [u8], rows: usize, columns: usize) -> Result<Bf16<'a>, Error> {
    let (head, tail) = rest.split_at(2 * rows * columns);
    *rest = tail;
    Bf16::new(rows, columns, head)
}

/// Run the prompt through the model and return the logits after each token
/// and the work charged. Every call starts from empty caches.
pub fn run(weights: &Weights, logits: &mut [f32; OUTPUT]) -> Result<u64, Error> {
    let mut rest: &[u8] = &weights.0;
    let mut layer = || -> Result<DecoderLayer<'_>, Error> {
        let mut t = [None; 12];
        for (slot, &(r, c)) in t.iter_mut().zip(SHAPES.iter()) {
            *slot = Some(tensor(&mut rest, r, c)?);
        }
        let t = t.map(|x| x.ok_or(Error::Shape));
        Ok(DecoderLayer {
            config: CONFIG,
            input_norm: t[0]?,
            q: t[1]?,
            q_bias: t[2]?,
            k: t[3]?,
            k_bias: t[4]?,
            v: t[5]?,
            v_bias: t[6]?,
            o: t[7]?,
            post_norm: t[8]?,
            gate: t[9]?,
            up: t[10]?,
            down: t[11]?,
        })
    };
    let layers = [layer()?, layer()?];
    let embedding = tensor(&mut rest, VOCAB, H)?;
    let norm = tensor(&mut rest, 1, H)?;
    let model = Model {
        embedding,
        layers: &layers,
        norm,
    };

    let (mut k0, mut v0, mut k1, mut v1) = (
        [0.; STEPS * KV],
        [0.; STEPS * KV],
        [0.; STEPS * KV],
        [0.; STEPS * KV],
    );
    let mut caches = [
        KvCache::new(CONFIG.kv_heads, CONFIG.head_dim, &mut k0, &mut v0)?,
        KvCache::new(CONFIG.kv_heads, CONFIG.head_dim, &mut k1, &mut v1)?,
    ];
    let (mut hidden, mut normed, mut scratch) = ([0f32; H], [0f32; H], [0f32; SCRATCH]);
    let mut budget = Budget::new(u64::MAX, &NoCancel);
    for (position, (&id, out)) in TOKENS
        .iter()
        .zip(logits.as_chunks_mut::<VOCAB>().0)
        .enumerate()
    {
        let len = CONFIG.scratch_len(position + 1).ok_or(Error::Shape)?;
        model.step(
            id,
            position as u32,
            &mut caches,
            &mut hidden,
            &mut scratch[..len],
            &mut budget,
        )?;
        model.logits(&hidden, CONFIG.rms_epsilon, &mut normed, out, &mut budget)?;
    }
    Ok(u64::MAX - budget.remaining())
}

/// Greedy token after each prompt position.
pub fn argmax(logits: &[f32; OUTPUT]) -> [u8; STEPS] {
    core::array::from_fn(|step| {
        let row = &logits[step * VOCAB..(step + 1) * VOCAB];
        let mut best = 0;
        for (i, v) in row.iter().enumerate() {
            if *v > row[best] {
                best = i;
            }
        }
        best as u8
    })
}

/// FNV-1a over the little-endian bit patterns of `logits`.
pub fn digest(logits: &[f32; OUTPUT]) -> u64 {
    logits
        .iter()
        .flat_map(|v| v.to_bits().to_le_bytes())
        .fold(0xcbf2_9ce4_8422_2325, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x100_0000_01b3)
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_is_deterministic_finite_and_nontrivial() {
        let w = Weights::new();
        let (mut a, mut b) = ([0.; OUTPUT], [0.; OUTPUT]);
        let macs = run(&w, &mut a).unwrap();
        assert_eq!(run(&w, &mut b).unwrap(), macs);
        assert_eq!(digest(&a), digest(&b));
        assert!(a.iter().all(|v| v.is_finite()));
        // Rows differ by position: the cache and RoPE are actually used.
        assert_ne!(a[..VOCAB], a[VOCAB..2 * VOCAB]);
        assert!(macs > 0);
    }
}
