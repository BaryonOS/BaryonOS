//! Host-only Candle oracle for L01. Reads a verified local copy of the pinned
//! Qwen2.5-0.5B-Instruct files and prints JSON fixtures: token ids, greedy
//! continuations and top-k final logits. No network access; nothing here runs
//! on the project's OS.

use std::{env, fs, path::Path};

use candle_core::{DType, Device, Tensor};
use candle_nn::VarBuilder;
use candle_transformers::models::qwen2::{Config, ModelForCausalLM};
use serde_json::{json, Value};
use tokenizers::Tokenizer;

const PROMPTS: &[&str] = &[
    "The capital of the Netherlands is",
    "De hoofdstad van Nederland is",
    "Summarize: Local AI runs on the device and never sends text to the cloud.",
    "Vat samen: lokale AI draait op het apparaat en stuurt geen tekst naar de cloud.",
    "café naïve façade — 東京 123,456.78 €",
];
const GENERATE: usize = 8;
const TOP_K: usize = 10;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = env::args().nth(1).ok_or("usage: candle-oracle <model-dir>")?;
    let dir = Path::new(&dir);
    let tokenizer = Tokenizer::from_file(dir.join("tokenizer.json")).map_err(|e| e.to_string())?;
    let config: Config = serde_json::from_slice(&fs::read(dir.join("config.json"))?)?;
    let device = Device::Cpu;
    // F32 matches the reference engine: BF16 weights widened losslessly.
    let vb = unsafe {
        // SAFETY: the file is a local, hash-verified copy that is not modified
        // while mapped.
        VarBuilder::from_mmaped_safetensors(&[dir.join("model.safetensors")], DType::F32, &device)?
    };
    let mut model = ModelForCausalLM::new(&config, vb)?;
    let mut cases = Vec::new();
    for prompt in PROMPTS {
        let ids = tokenizer
            .encode(*prompt, false)
            .map_err(|e| e.to_string())?
            .get_ids()
            .to_vec();
        model.clear_kv_cache();
        let input = Tensor::new(ids.as_slice(), &device)?.unsqueeze(0)?;
        let logits = model.forward(&input, 0)?.squeeze(0)?.squeeze(0)?;
        let values: Vec<f32> = logits.to_vec1()?;
        let mut order: Vec<usize> = (0..values.len()).collect();
        order.sort_by(|&a, &b| values[b].total_cmp(&values[a]));
        let top: Vec<Value> = order[..TOP_K]
            .iter()
            .map(|&i| json!([i, values[i]]))
            .collect();
        let mut generated = vec![order[0] as u32];
        for step in 1..GENERATE {
            let input = Tensor::new(&[generated[step - 1]], &device)?.unsqueeze(0)?;
            let logits = model.forward(&input, ids.len() + step - 1)?.squeeze(0)?.squeeze(0)?;
            generated.push(logits.argmax(0)?.to_scalar::<u32>()?);
        }
        let text = tokenizer.decode(&generated, false).map_err(|e| e.to_string())?;
        cases.push(json!({
            "prompt": prompt,
            "ids": ids,
            "top_logits": top,
            "greedy_ids": generated,
            "greedy_text": text,
        }));
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "oracle": "candle 0.11.1, tokenizers 0.23.2, CPU, F32",
            "model": "Qwen/Qwen2.5-0.5B-Instruct@7ae557604adf67be50417f59c2c2f167def9a775",
            "cases": cases,
        }))?
    );
    Ok(())
}
