//! Opt-in check of the full reference graph against the Candle oracle fixture
//! (`experiments/ai/candle-oracle`). Set `NOS_QWEN_DIR` to a directory holding
//! the pinned `model.safetensors`; run with `--release`. Without the variable the
//! test returns early and proves nothing.

use artifact_store_core::{
    ArtifactDescriptor, ArtifactRef, ArtifactStore, Error as StoreError, StoreCapacity,
    READ_CHUNK_MAX,
};
use baryon_ai_reference_forward::{
    Bf16, Budget, DecoderLayer, KvCache, LayerConfig, Model, NoCancel,
};
use nieuw_os_model_format::{parse, Dtype, Tensor};
use nieuw_os_model_store_contract::{open_pinned_model, Cancellation, ReadBudget};

const LAYERS: usize = 24;
const CONFIG: LayerConfig = LayerConfig {
    hidden: 896,
    heads: 14,
    kv_heads: 2,
    head_dim: 64,
    intermediate: 4864,
    rms_epsilon: 1e-6,
    rope_theta: 1_000_000.,
};
const VOCAB: usize = 151_936;

struct Never;
impl Cancellation for Never {
    fn is_cancelled(&self) -> bool {
        false
    }
}

struct Memory(Vec<u8>, ArtifactRef);
impl ArtifactStore for Memory {
    type Handle = ArtifactRef;
    fn open(
        &mut self,
        d: &ArtifactDescriptor,
    ) -> Result<(Self::Handle, ArtifactDescriptor), StoreError> {
        Ok((
            self.1,
            ArtifactDescriptor {
                length: self.0.len() as u64,
                ..*d
            },
        ))
    }
    fn read_at(
        &mut self,
        _: &ArtifactRef,
        offset: u64,
        out: &mut [u8],
    ) -> Result<usize, StoreError> {
        let start = offset as usize;
        let n = out.len().min(self.0.len() - start);
        out[..n].copy_from_slice(&self.0[start..start + n]);
        Ok(n)
    }
    fn capacity(&mut self) -> Result<StoreCapacity, StoreError> {
        Err(StoreError::Io)
    }
}

#[test]
fn full_graph_matches_candle_oracle() {
    let Ok(dir) = std::env::var("NOS_QWEN_DIR") else {
        eprintln!("NOS_QWEN_DIR unset: oracle comparison skipped");
        return;
    };
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/qwen-oracle.json")).unwrap();
    let reference = ArtifactRef {
        artifact_id: [1; 32],
        immutable_generation: 1,
    };
    let mut store = Memory(
        std::fs::read(format!("{dir}/model.safetensors")).unwrap(),
        reference,
    );

    // Verify the full artifact against the pinned length and SHA-256 first.
    let mut verifier = open_pinned_model(&mut store, reference).unwrap();
    let mut read = ReadBudget::new(u64::MAX);
    while !verifier.verify_next(&mut store, &mut read, &Never).unwrap() {}
    let mut artifact = verifier.finish().unwrap();

    let mut header = vec![0; 64 * 1024];
    let mut table = vec![Tensor::EMPTY; 512];
    let model = parse(
        &mut artifact,
        &mut store,
        &mut read,
        &Never,
        &mut header,
        &mut table,
    )
    .unwrap();
    let start = model.data_start() as usize;
    let tensors: Vec<(String, (u64, u64), Vec<u64>)> = model
        .tensors()
        .iter()
        .map(|t| {
            assert_eq!(t.dtype(), Dtype::BF16);
            (t.name().to_string(), t.data_offsets(), t.shape().to_vec())
        })
        .collect();
    // Data reads go through the verified artifact, in bounded chunks.
    let mut data = vec![0u8; store.0.len() - start];
    let mut offset = 0;
    while offset < data.len() {
        let end = (offset + READ_CHUNK_MAX).min(data.len());
        offset += artifact
            .read_at(
                &mut store,
                (start + offset) as u64,
                &mut read,
                &Never,
                &mut data[offset..end],
            )
            .unwrap();
    }
    let tensor = |name: &str| -> Bf16<'_> {
        let (_, (a, b), shape) = tensors.iter().find(|t| t.0 == name).unwrap();
        let (rows, cols) = match shape.as_slice() {
            [r, c] => (*r as usize, *c as usize),
            [n] => (1, *n as usize),
            _ => panic!("rank"),
        };
        Bf16::new(rows, cols, &data[*a as usize..*b as usize]).unwrap()
    };
    let layers: Vec<DecoderLayer<'_>> = (0..LAYERS)
        .map(|i| {
            let p = |s: &str| tensor(&format!("model.layers.{i}.{s}"));
            DecoderLayer {
                config: CONFIG,
                input_norm: p("input_layernorm.weight"),
                q: p("self_attn.q_proj.weight"),
                q_bias: p("self_attn.q_proj.bias"),
                k: p("self_attn.k_proj.weight"),
                k_bias: p("self_attn.k_proj.bias"),
                v: p("self_attn.v_proj.weight"),
                v_bias: p("self_attn.v_proj.bias"),
                o: p("self_attn.o_proj.weight"),
                post_norm: p("post_attention_layernorm.weight"),
                gate: p("mlp.gate_proj.weight"),
                up: p("mlp.up_proj.weight"),
                down: p("mlp.down_proj.weight"),
            }
        })
        .collect();
    let qwen = Model {
        embedding: tensor("model.embed_tokens.weight"),
        layers: &layers,
        norm: tensor("model.norm.weight"),
    };

    let case = &fixture["cases"][0];
    let ids: Vec<u32> = case["ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect();
    let greedy: Vec<u32> = case["greedy_ids"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u32)
        .collect();
    let capacity = ids.len() + 2;
    let kv = CONFIG.kv_heads * CONFIG.head_dim * capacity;
    let mut storage: Vec<(Vec<f32>, Vec<f32>)> =
        (0..LAYERS).map(|_| (vec![0.; kv], vec![0.; kv])).collect();
    let mut caches: Vec<KvCache<'_>> = storage
        .iter_mut()
        .map(|(k, v)| KvCache::new(CONFIG.kv_heads, CONFIG.head_dim, k, v).unwrap())
        .collect();
    let mut budget = Budget::new(u64::MAX, &NoCancel);
    let (mut hidden, mut normed) = (vec![0.; 896], vec![0.; 896]);
    let (mut staging, mut logits) = (vec![0.; VOCAB], vec![0.; VOCAB]);
    let mut run = |id: u32, position: usize, caches: &mut [KvCache<'_>]| {
        let mut scratch = vec![0.; CONFIG.scratch_len(position + 1).unwrap()];
        qwen.step(
            id,
            position as u32,
            caches,
            &mut hidden,
            &mut scratch,
            &mut budget,
        )
        .unwrap();
        qwen.logits(
            &hidden,
            CONFIG.rms_epsilon,
            &mut normed,
            &mut staging,
            &mut logits,
            &mut budget,
        )
        .unwrap();
        logits.clone()
    };
    let mut last = Vec::new();
    for (position, &id) in ids.iter().enumerate() {
        last = run(id, position, &mut caches);
    }
    // Top-10 final logits: the computed top-10 id set equals the fixture's,
    // and each fixture value is within 1e-2.
    let mut order: Vec<usize> = (0..last.len()).collect();
    order.sort_by(|a, b| last[*b].total_cmp(&last[*a]));
    let mut got: Vec<u64> = order[..10].iter().map(|&i| i as u64).collect();
    let mut want: Vec<u64> = case["top_logits"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| e[0].as_u64().unwrap())
        .collect();
    got.sort_unstable();
    want.sort_unstable();
    assert_eq!(got, want, "top-10 token ids");
    for entry in case["top_logits"].as_array().unwrap() {
        let id = entry[0].as_u64().unwrap() as usize;
        let want = entry[1].as_f64().unwrap() as f32;
        assert!(
            (last[id] - want).abs() < 1e-2,
            "logit {id}: {} vs {want}",
            last[id]
        );
    }
    let argmax = |l: &[f32]| {
        l.iter()
            .enumerate()
            .max_by(|a, b| a.1.total_cmp(b.1))
            .unwrap()
            .0 as u32
    };
    // Two greedy steps reproduce the oracle continuation.
    let first = argmax(&last);
    assert_eq!(first, greedy[0]);
    let next = run(first, ids.len(), &mut caches);
    assert_eq!(argmax(&next), greedy[1]);
}
