use baryon_ai_reference_forward::{
    kv_cache_bytes, Bf16, Budget, Cancel, DecoderLayer, Error, KvCache, LayerConfig, NoCancel,
};

const CONFIG: LayerConfig = LayerConfig {
    hidden: 4,
    heads: 2,
    kv_heads: 1,
    head_dim: 2,
    intermediate: 3,
    rms_epsilon: 1e-6,
    rope_theta: 10_000.,
};

/// Deterministic weights that are exact in bfloat16 (multiples of 1/16 in [-1, 1]).
fn weights(seed: u32, n: usize) -> Vec<f32> {
    (0..n as u32)
        .map(|i| ((i * 7 + seed * 13) % 33) as f32 / 16.0 - 1.0)
        .collect()
}

fn encode(values: &[f32]) -> Vec<u8> {
    values
        .iter()
        .flat_map(|v| ((v.to_bits() >> 16) as u16).to_le_bytes())
        .collect()
}

struct Cancelled;

impl Cancel for Cancelled {
    fn cancelled(&self) -> bool {
        true
    }
}

/// Weights as (f32 values, bf16 bytes, rows, columns) in layer order.
struct Tensors(Vec<(Vec<f32>, Vec<u8>, usize, usize)>);

impl Tensors {
    fn new() -> Self {
        let c = CONFIG;
        let (h, kv, i) = (c.hidden, c.kv_heads * c.head_dim, c.intermediate);
        // input norm, q, q bias, k, k bias, v, v bias, o, post norm, gate, up, down
        let shapes = [
            (1, h),
            (h, h),
            (1, h),
            (kv, h),
            (1, kv),
            (kv, h),
            (1, kv),
            (h, h),
            (1, h),
            (i, h),
            (i, h),
            (h, i),
        ];
        Self(
            shapes
                .iter()
                .enumerate()
                .map(|(seed, &(r, c))| {
                    let w = weights(seed as u32 + 1, r * c);
                    let b = encode(&w);
                    (w, b, r, c)
                })
                .collect(),
        )
    }

    fn layer(&self) -> DecoderLayer<'_> {
        let t: Vec<Bf16<'_>> = self
            .0
            .iter()
            .map(|(_, b, r, c)| Bf16::new(*r, *c, b).unwrap())
            .collect();
        DecoderLayer {
            config: CONFIG,
            input_norm: t[0],
            q: t[1],
            q_bias: t[2],
            k: t[3],
            k_bias: t[4],
            v: t[5],
            v_bias: t[6],
            o: t[7],
            post_norm: t[8],
            gate: t[9],
            up: t[10],
            down: t[11],
        }
    }

    fn w(&self, index: usize) -> Vec<f64> {
        self.0[index].0.iter().map(|&x| x as f64).collect()
    }
}

fn matvec(w: &[f64], rows: usize, x: &[f64], bias: Option<&[f64]>) -> Vec<f64> {
    (0..rows)
        .map(|r| {
            let dot: f64 = (0..x.len()).map(|c| w[r * x.len() + c] * x[c]).sum();
            dot + bias.map_or(0.0, |b| b[r])
        })
        .collect()
}

fn norm(x: &[f64], w: &[f64]) -> Vec<f64> {
    let rms = (x.iter().map(|v| v * v).sum::<f64>() / x.len() as f64 + 1e-6).sqrt();
    x.iter().zip(w).map(|(v, g)| v / rms * g).collect()
}

fn rope(x: &[f64], position: usize, d: usize) -> Vec<f64> {
    let mut out = x.to_vec();
    for (index, v) in x.chunks(d).enumerate() {
        for i in 0..d / 2 {
            let angle = position as f64 / 10_000f64.powf(2.0 * i as f64 / d as f64);
            let (a, b) = (v[i], v[i + d / 2]);
            out[index * d + i] = a * angle.cos() - b * angle.sin();
            out[index * d + i + d / 2] = b * angle.cos() + a * angle.sin();
        }
    }
    out
}

fn add(a: &[f64], b: &[f64]) -> Vec<f64> {
    a.iter().zip(b).map(|(x, y)| x + y).collect()
}

/// Independent f64 reference for a sequence of hidden states through one layer.
fn reference(t: &Tensors, inputs: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let c = CONFIG;
    let (h, d, kv) = (c.hidden, c.head_dim, c.kv_heads * c.head_dim);
    let (mut keys, mut values, mut outputs) = (vec![], vec![], vec![]);
    for (position, x) in inputs.iter().enumerate() {
        let n = norm(x, &t.w(0));
        let q = rope(&matvec(&t.w(1), h, &n, Some(&t.w(2))), position, d);
        keys.push(rope(&matvec(&t.w(3), kv, &n, Some(&t.w(4))), position, d));
        values.push(matvec(&t.w(5), kv, &n, Some(&t.w(6))));
        let mut attn = vec![0.0; h];
        for head in 0..c.heads {
            let kvh = head / (c.heads / c.kv_heads);
            let scores: Vec<f64> = keys
                .iter()
                .map(|k| {
                    (0..d)
                        .map(|i| q[head * d + i] * k[kvh * d + i])
                        .sum::<f64>()
                        / (d as f64).sqrt()
                })
                .collect();
            let max = scores.iter().cloned().fold(f64::MIN, f64::max);
            let exps: Vec<f64> = scores.iter().map(|s| (s - max).exp()).collect();
            let total: f64 = exps.iter().sum();
            for (e, v) in exps.iter().zip(&values) {
                for i in 0..d {
                    attn[head * d + i] += e / total * v[kvh * d + i];
                }
            }
        }
        let residual = add(&matvec(&t.w(7), h, &attn, None), x);
        let n2 = norm(&residual, &t.w(8));
        let gate = matvec(&t.w(9), c.intermediate, &n2, None);
        let up = matvec(&t.w(10), c.intermediate, &n2, None);
        let act: Vec<f64> = gate
            .iter()
            .zip(&up)
            .map(|(g, u)| g / (1.0 + (-g).exp()) * u)
            .collect();
        outputs.push(add(&matvec(&t.w(11), h, &act, None), &residual));
    }
    outputs
}

fn inputs() -> Vec<Vec<f64>> {
    vec![
        vec![0.5, -1.0, 0.25, 2.0],
        vec![-0.75, 0.5, 1.5, -0.25],
        vec![1.0, 1.0, -1.0, 0.125],
    ]
}

#[test]
fn layer_matches_independent_f64_reference_over_three_tokens() {
    let t = Tensors::new();
    let layer = t.layer();
    let expected = reference(&t, &inputs());
    let (mut k, mut v) = ([0.; 6], [0.; 6]);
    let mut cache = KvCache::new(1, 2, &mut k, &mut v).unwrap();
    let mut budget = Budget::new(u64::MAX, &NoCancel);
    for (position, (x, want)) in inputs().iter().zip(&expected).enumerate() {
        let mut hidden: Vec<f32> = x.iter().map(|&v| v as f32).collect();
        let mut scratch = vec![0.; CONFIG.scratch_len(position + 1).unwrap()];
        layer
            .forward(
                &mut hidden,
                position as u32,
                &mut cache,
                &mut scratch,
                &mut budget,
            )
            .unwrap();
        for (got, want) in hidden.iter().zip(want) {
            assert!(
                (*got as f64 - want).abs() <= 1e-4 * want.abs().max(1.0),
                "position {position}: {got} vs {want}"
            );
        }
    }
    assert_eq!(cache.len(), 3);
}

#[test]
fn failures_roll_back_cache_and_leave_hidden_unchanged() {
    let t = Tensors::new();
    let layer = t.layer();
    let (mut k, mut v) = ([0.; 2], [0.; 2]);
    let mut cache = KvCache::new(1, 2, &mut k, &mut v).unwrap();
    let start = [0.5f32, -1.0, 0.25, 2.0];
    let mut hidden = start;
    let mut scratch = vec![0.; CONFIG.scratch_len(1).unwrap()];

    // Enough budget for norm + Q + K + V (8 + 16 + 8 + 8), so the K/V append
    // happens and attention then fails: the append must be rolled back.
    let mut budget = Budget::new(40, &NoCancel);
    assert_eq!(
        layer.forward(&mut hidden, 0, &mut cache, &mut scratch, &mut budget),
        Err(Error::WorkLimit)
    );
    assert_eq!((hidden, cache.len()), (start, 0));

    let mut cancelled = Budget::new(u64::MAX, &Cancelled);
    assert_eq!(
        layer.forward(&mut hidden, 0, &mut cache, &mut scratch, &mut cancelled),
        Err(Error::Cancelled)
    );
    assert_eq!((hidden, cache.len()), (start, 0));

    // A full cache is refused, never truncated or evicted.
    let mut budget = Budget::new(u64::MAX, &NoCancel);
    layer
        .forward(&mut hidden, 0, &mut cache, &mut scratch, &mut budget)
        .unwrap();
    let after = hidden;
    let mut scratch = vec![0.; CONFIG.scratch_len(2).unwrap()];
    assert_eq!(
        layer.forward(&mut hidden, 1, &mut cache, &mut scratch, &mut budget),
        Err(Error::ContextLimit)
    );
    assert_eq!((hidden, cache.len()), (after, 1));

    // Wrong scratch size and non-finite input are refused without side effects.
    let (mut k, mut v) = ([0.; 4], [0.; 4]);
    let mut cache = KvCache::new(1, 2, &mut k, &mut v).unwrap();
    assert_eq!(
        layer.forward(&mut hidden, 0, &mut cache, &mut [0.; 3], &mut budget),
        Err(Error::Shape)
    );
    let mut bad = [f32::NAN, 0., 0., 0.];
    let mut scratch = vec![0.; CONFIG.scratch_len(1).unwrap()];
    assert_eq!(
        layer.forward(&mut bad, 0, &mut cache, &mut scratch, &mut budget),
        Err(Error::NonFinite)
    );
    assert!(cache.is_empty());
}

#[test]
fn bf16_tensors_reject_bad_shapes_and_nonfinite_values() {
    assert!(matches!(Bf16::new(2, 2, &[0; 6]), Err(Error::Shape)));
    assert!(matches!(Bf16::new(0, 2, &[]), Err(Error::Shape)));
    assert!(matches!(Bf16::new(usize::MAX, 2, &[]), Err(Error::Shape)));
    for value in [f32::NAN, f32::INFINITY] {
        assert!(matches!(
            Bf16::new(1, 1, &encode(&[value])),
            Err(Error::NonFinite)
        ));
    }
}

#[test]
fn pinned_qwen_shape_fits_the_approved_budgets() {
    // Qwen2.5-0.5B-Instruct config at revision 7ae5576 (config.json SHA-256
    // 18e18afc…ff45): hidden 896, 14 heads, 2 KV heads, intermediate 4864, 24 layers.
    let qwen = LayerConfig {
        hidden: 896,
        heads: 14,
        kv_heads: 2,
        head_dim: 64,
        intermediate: 4864,
        rms_epsilon: 1e-6,
        rope_theta: 1_000_000.,
    };
    // f32 KV for all 24 layers at the 1,024-token context: 24 MiB of 128 MiB.
    assert_eq!(kv_cache_bytes(24, 2, 64, 1024, 4), Ok(25_165_824));
    // Per-layer scratch at that context, in f32.
    assert_eq!(qwen.scratch_len(1024).map(|n| n * 4), Some(73_216));
}

#[test]
fn scratch_len_refuses_kv_width_overflow() {
    let bad = LayerConfig {
        kv_heads: usize::MAX / 2 + 1,
        ..CONFIG
    };
    assert_eq!(bad.scratch_len(1), None);
}
