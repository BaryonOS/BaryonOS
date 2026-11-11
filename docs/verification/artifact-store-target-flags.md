# Artifact-store freestanding compiler flags — issue #292

Source: `f11e20d3d9981f683802f493325e8a5cf2b2b1b0` on `main`; fix branch `coderabbit/issue-292-preserve-target-flags`.
Issue: https://github.com/simonvanieperen/BaryonOS/issues/292

## Observation and change

Rust 1.90.0 verbose builds were executed before changing the workflow and after
replacing environment `RUSTFLAGS` with an additive `--config` target-rustflags
array. The original invocation retained `linker=rust-lld`, `no-redzone=yes`
and the scalar target features, but omitted `code-model=small`,
`relocation-model=static` and every configured linker argument. Both builds
returned exit status 0: compiling a library alone does not detect this loss.
The corrected invocation includes every configured argument below. The shared
`.cargo/config.toml` is unchanged.

### Before — observed `rustc -C` arguments

```text
opt-level=3
panic=abort
embed-bitcode=no
codegen-units=1
metadata=8a1ed5a498dbe07d
extra-filename=-7632c29ddfb3813b
linker=rust-lld
strip=debuginfo
no-redzone=yes
target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2
```

### After — observed `rustc -C` arguments

```text
opt-level=3
panic=abort
embed-bitcode=no
codegen-units=1
metadata=8a1ed5a498dbe07d
extra-filename=-0c5183bc5f537b8d
linker=rust-lld
strip=debuginfo
code-model=small
relocation-model=static
no-redzone=yes
link-arg=-Tkernel/linker.ld
link-arg=--no-pie
link-arg=--build-id=none
link-arg=-zmax-page-size=0x1000
target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2
```

## Validation

All commands below returned exit status 0 with
`CARGO_TARGET_DIR=$PWD/target/artifact-store-quality` and Rust 1.90.0:

```sh
cargo +1.90.0 test --locked -p artifact-store-core
cargo +1.90.0 clippy --locked -p artifact-store-core --all-targets -- -D warnings
cargo +1.90.0 fmt --manifest-path crates/artifact-store-core/Cargo.toml -- --check
cargo +1.90.0 build -vv --locked --release --lib --target x86_64-unknown-none -p artifact-store-core --config 'target.x86_64-unknown-none.rustflags=["-C", "target-feature=-sse,-sse2,-sse3,-ssse3,-sse4.1,-sse4.2,-avx,-avx2"]'
python3 -m unittest discover -s tests -p test_ci_workflows.py
```

The contract suite passed 9 tests; the workflow suite passed 12 tests.
Restoring the original workflow temporarily caused the new
`test_artifact_build_preserves_target_rustflags` check to fail as expected;
the corrected workflow was restored afterwards. The check rejects environment
flag overrides and requires the additive scalar-feature configuration.

Before-fix reproduction used the same verbose build without `--config` and
with the original workflow's `RUSTFLAGS`. A token-level comparison of the
observed compiler arguments against `.cargo/config.toml` found six missing
settings before the fix and none afterwards.

This is host-test and cross-compilation evidence, not kernel linking, QEMU,
physical hardware, or formal safety evidence. The GitHub-hosted job itself
was not run locally; its Rust quality commands were executed in this sandbox.
Graphify was unavailable; repository files were inspected directly.
Progress remains 39/100 because this CI correction accepts no OS criterion.

Cargo configuration reference (Rust Project):
https://doc.rust-lang.org/cargo/reference/config.html#buildrustflags
The observed invocations above provide version-specific local verification.
