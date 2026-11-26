//! Opt-in id-for-id check of the no_std tokenizer against the pinned
//! tokenizers 0.23.2 (Oniguruma) reference. Set `NOS_QWEN_DIR` to the
//! directory holding the pinned `tokenizer.json`; without it the test returns
//! early and proves nothing. `NOS_TOKENIZER_SWEEP` may name an extra fixture
//! from `candle-oracle --bin tokenize <dir> --cjk` (not committed). Host only.

mod common;

use artifact_store_core::{
    ArtifactDescriptor, ArtifactRef, ArtifactStore, Error as StoreError, StoreCapacity,
};
use baryon_ai_reference_forward::{
    pre_tokenize, Budget, NoCancel, TokenError, Tokenizer, ADDED_TOKENS,
};
use nieuw_os_model_store_contract::{open, Cancellation, ReadBudget};
use serde_json::{json, Value};

const TOKENIZER_LEN: u64 = 7_031_645;
const TOKENIZER_SHA256: &str = "c0382117ea329cdf097041132f6d735924b697924d6f6fc3945713e96ce87539";
const SPLIT: &str = r"(?i:'s|'t|'re|'ve|'m|'ll|'d)|[^\r\n\p{L}\p{N}]?\p{L}+|\p{N}| ?[^\s\p{L}\p{N}]+[\r\n]*|\s*[\r\n]+|\s+(?!\S)|\s+";

struct Never;
impl Cancellation for Never {
    fn is_cancelled(&self) -> bool {
        false
    }
}

// Same in-memory store as qwen_oracle.rs; deduplicate once #525 has landed.
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

/// Read `tokenizer.json` and verify it against the pinned length and SHA-256.
fn pinned_tokenizer(dir: &str) -> Value {
    let reference = ArtifactRef {
        artifact_id: [2; 32],
        immutable_generation: 1,
    };
    let mut sha256 = [0; 32];
    for (i, byte) in sha256.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&TOKENIZER_SHA256[2 * i..2 * i + 2], 16).unwrap();
    }
    let mut store = Memory(
        std::fs::read(format!("{dir}/tokenizer.json")).unwrap(),
        reference,
    );
    let descriptor = ArtifactDescriptor {
        reference,
        length: TOKENIZER_LEN,
        sha256,
    };
    let mut verifier = open(&mut store, descriptor).unwrap();
    let mut read = ReadBudget::new(u64::MAX);
    while !verifier.verify_next(&mut store, &mut read, &Never).unwrap() {}
    verifier.finish().unwrap();
    serde_json::from_slice(&store.0).unwrap()
}

/// GPT-2 byte-to-character map used by the ByteLevel pre-tokenizer.
fn byte_chars() -> [char; 256] {
    let printable = |b: u32| (0x21..=0x7E).contains(&b) || (0xA1..=0xAC).contains(&b) || b >= 0xAE;
    let mut map = ['\0'; 256];
    let mut next = 256;
    for b in 0..256u32 {
        map[b as usize] = if printable(b) {
            char::from_u32(b).unwrap()
        } else {
            next += 1;
            char::from_u32(next - 1).unwrap()
        };
    }
    map
}

/// Check every assumption the no_std table relies on, then serialize it.
fn qwen_blob(t: &Value) -> Vec<u8> {
    let byte_level = json!({"type": "ByteLevel", "add_prefix_space": false, "trim_offsets": false, "use_regex": false});
    assert_eq!(t["normalizer"], json!({"type": "NFC"}));
    assert_eq!(
        t["pre_tokenizer"],
        json!({"type": "Sequence", "pretokenizers": [
            {"type": "Split", "pattern": {"Regex": SPLIT}, "behavior": "Isolated", "invert": false},
            byte_level,
        ]})
    );
    assert_eq!(t["post_processor"], byte_level);
    assert_eq!(t["decoder"], byte_level);
    let model = &t["model"];
    assert_eq!(model["type"], "BPE");
    for (key, value) in [
        ("dropout", Value::Null),
        ("unk_token", Value::Null),
        ("continuing_subword_prefix", json!("")),
        ("end_of_word_suffix", json!("")),
        ("fuse_unk", json!(false)),
        ("byte_fallback", json!(false)),
    ] {
        assert_eq!(model[key], value, "{key}");
    }
    assert!(model.get("ignore_merges").is_none_or(|v| v == false));
    let vocab = model["vocab"].as_object().unwrap();
    let id = |s: &str| vocab[s].as_u64().unwrap() as u32;
    let chars = byte_chars();
    let mut byte_ids = [0; 256];
    for (b, c) in chars.iter().enumerate() {
        byte_ids[b] = id(&c.to_string());
    }
    let lines = model["merges"].as_array().unwrap();
    let mut merges = Vec::with_capacity(lines.len());
    for (rank, line) in lines.iter().enumerate() {
        let (left, right) = line.as_str().unwrap().split_once(' ').unwrap();
        assert_eq!(id(&format!("{left}{right}")), 256 + rank as u32, "{line}");
        merges.push((id(left), id(right)));
    }
    assert_eq!(vocab.len(), 256 + merges.len());
    let added = t["added_tokens"].as_array().unwrap();
    assert_eq!(added.len(), ADDED_TOKENS.len());
    for (i, (token, expected)) in added.iter().zip(ADDED_TOKENS).enumerate() {
        assert_eq!(token["content"], expected);
        assert_eq!(token["id"], (vocab.len() + i) as u64);
        for flag in ["single_word", "lstrip", "rstrip", "normalized"] {
            assert_eq!(token[flag], false, "{expected} {flag}");
        }
    }
    common::blob(&byte_ids, &merges)
}

fn check(t: &Tokenizer<'_>, text: &str, ids: &[u32]) {
    let mut out = vec![0; text.len()];
    let mut budget = Budget::new(u64::MAX, &NoCancel);
    let n = t.encode(text, &mut out, &mut budget).unwrap();
    assert_eq!(&out[..n], ids, "{text:?}");
    let mut bytes = vec![0; text.len()];
    let n = t.decode(ids, &mut bytes).unwrap();
    assert_eq!(&bytes[..n], text.as_bytes());
}

fn ids(v: &Value) -> Vec<u32> {
    v.as_array()
        .unwrap()
        .iter()
        .map(|i| i.as_u64().unwrap() as u32)
        .collect()
}

#[test]
fn pinned_tokenizer_matches_reference() {
    let Ok(dir) = std::env::var("NOS_QWEN_DIR") else {
        eprintln!("NOS_QWEN_DIR unset: tokenizer comparison skipped");
        return;
    };
    let data = qwen_blob(&pinned_tokenizer(&dir));
    let t = Tokenizer::new(&data).unwrap();
    assert_eq!(t.vocab(), 151_643);

    let oracle: Value = serde_json::from_str(include_str!("fixtures/qwen-oracle.json")).unwrap();
    for case in oracle["cases"].as_array().unwrap() {
        check(&t, case["prompt"].as_str().unwrap(), &ids(&case["ids"]));
        let greedy = ids(&case["greedy_ids"]);
        let mut bytes = [0; 256];
        let n = t.decode(&greedy, &mut bytes).unwrap();
        assert_eq!(
            &bytes[..n],
            case["greedy_text"].as_str().unwrap().as_bytes()
        );
    }

    let mut corpora = vec![include_str!("fixtures/qwen-tokenizer.json").to_string()];
    if let Ok(path) = std::env::var("NOS_TOKENIZER_SWEEP") {
        corpora.push(std::fs::read_to_string(path).unwrap());
    }
    let mut cases = 0;
    for corpus in corpora {
        let corpus: Value = serde_json::from_str(&corpus).unwrap();
        for case in corpus["cases"].as_array().unwrap() {
            let text = case["text"].as_str().unwrap();
            let pieces: Vec<&str> = pre_tokenize(text).unwrap().collect();
            assert_eq!(json!(pieces), case["pieces"], "{text:?}");
            check(&t, text, &ids(&case["ids"]));
            cases += 1;
        }
    }
    eprintln!("{cases} reference cases matched");

    // Control literals are refused in text but decode from their ids.
    let mut out = [0; 32];
    let mut budget = Budget::new(u64::MAX, &NoCancel);
    assert_eq!(
        t.encode("<|im_start|>user", &mut out, &mut budget),
        Err(TokenError::Unsupported)
    );
    let mut bytes = [0; 32];
    let n = t.decode(&[151_644, 151_645], &mut bytes).unwrap();
    assert_eq!(&bytes[..n], b"<|im_start|><|im_end|>");
}
