# Candle host oracle (L01)

Host-only reference that runs the pinned Qwen2.5-0.5B-Instruct model with
Candle and prints JSON fixtures: token ids, top-10 final logits and an 8-token
greedy continuation for five English, Dutch and mixed-script prompts. The no_std
engine in `../reference-forward` is checked against
`../reference-forward/tests/fixtures/qwen-oracle.json`. This tool never runs on
the project's OS and is not part of any image; it reads only a local directory.

```sh
cargo +1.100.0 build --release --locked
target/release/baryon-ai-candle-oracle <dir with config.json, tokenizer.json, model.safetensors>
```

The model directory must hold the files listed in the reference-forward README,
checked against their SHA-256; `model.safetensors` is 988,097,824 bytes with
SHA-256 `fdf756fa7fcbe7404d5c60e26bff1a0c8b8aa1f72ced49e7dd0210fe288fb7fe`.
Observed on the development host (32 CPUs): 4.6 s wall, 2.97 GB peak RSS for the
whole fixture set (F32 weights).

## Dependency review (25 November 2026; serde_json closure rechecked 27 November 2026)

| Crate | Version | Released | Licence | Features |
|---|---|---|---|---|
| candle-core, candle-nn, candle-transformers | =0.11.1 | 2026-10-31 | MIT OR Apache-2.0 | none (CPU only; no CUDA/Metal/MKL) |
| tokenizers | =0.23.2 | 2026-09-03 | Apache-2.0 | `onig` only (no progress bar, no HTTP/hub) |
| serde_json | =1.0.154 | 2026-10-11 | MIT OR Apache-2.0 | default |

The lockfile resolves 171 packages, all under permissive licences (MIT,
Apache-2.0, Unlicense, Zlib, Unicode-3.0, LLVM exception). `onig_sys` compiles the
vendored Oniguruma C library; the other build scripts emit configuration only.
The single `unsafe` block memory-maps the hash-verified local model file. No
network access is performed by the tool; the model files are fetched separately.

serde_json 1.0.145 → 1.0.154 (27 November 2026) changes one thing in the closure:
it adds `zmij` 1.0.23 (MIT, <https://github.com/dtolnay/zmij>, published by the
serde_json maintainer, crates.io checksum
`29666d0abbfad1e3dc4dcf6144730dd3a3ab225bbbdac83319345b1b44ccfc1b`), which replaces
`ryu` as serde_json's float-to-string routine. `ryu` 1.0.23 stays in this lockfile
through `compact_str` (tokenizers) and leaves the `../reference-forward` lockfile,
where serde_json is a dev-dependency only. The serde_json checksum is
`e7e9cc8b1b85264074fbcc02a88680c4096b1e47df8f739dceb03bf482f04bd6`; both versions
and checksums equal the ones already recorded in
`experiments/quantum/THIRD-PARTY.md`. The build scripts of serde_json and zmij
were read: they emit `cfg` flags only (zmij runs `rustc --version`). zmij contains
`unsafe` code and is not audited here. No RustSec/OSV advisory was listed for
either version on 27 November 2026 (OSV query; `cargo audit` was not run).
