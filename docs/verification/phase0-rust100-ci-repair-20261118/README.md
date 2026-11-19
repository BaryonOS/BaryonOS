# Rust 1.100 hosted-CI compiler repair evidence

The source under test is commit `71c9d0592d7243fc8db72d452a616a66748afe38`,
with source fingerprint v3
`acc0da9822c62c8251eff8e935fd34c12d34295b316e1d0ab94f21578beb6391`.
This is PR #354 on its temporary S09 preintegration base. This receipt does
not establish acceptance of the eventual combined main.

The published `bfbb4a6d` candidate failed actual hosted Clippy checks in
network parsing and window fixtures. Nine files now use equivalent validated
array chunks or iterators. Existing size guards, assertions, protocol ordering,
CI flags and dependency pins are retained. Additional compiler findings from
the complete local window host command list were repaired before publication.

`checks.json` records all commands, raw exits, artifact hashes and scope:

- The existing `ci.yml#host-quality` local gate passed all 18 steps, exit 0,
  in 310.917 seconds. Its full transcript records the commands and their exits.
- All 13 unique native-window workflow host commands passed, exit 0 each,
  in 631.045 seconds, including the full window-session, materiaal and
  calculator tests, original Clippy flags and formatting checks.
- All nine locked dependency preparations and both native window scale probe
  builds passed, exit 0 each, in 44.926 seconds. The two probe manifests bind
  the produced loader/kernel hashes to the source fingerprint above.
- The network crate's declared Rust 1.90 freestanding target check passed.

Validation used the repository's canonical Token Guard and the Rust 1.100.0
toolchain. The full host-quality gate ran in a clean detached worktree at the
same source commit, with `CARGO_BUILD_JOBS=2`, isolated `CARGO_TARGET_DIR`,
`NOS_TARGET_DIR` and `NOS_BUILD_DIR`. Workflow-pinned actionlint 1.7.12,
cargo-llvm-cov 0.9.1, coverage 7.11.3, c8 10.1.3 and PyYAML 6.0.3 were
provisioned into dedicated local prefixes. The actionlint archive checksum
matches the workflow. Python dependencies were installed with the repository's
hash-checked requirements. Probe builds used `CARGO_NET_OFFLINE=true`,
`NOS_WINDOW_SCALE=1` or `2`, and separate build and native-target directories.
The earlier stable-manifest provenance is retained in the previous archive.

Full transcripts are compressed losslessly with deterministic gzip timestamps.
`checks.json` includes raw and compressed SHA-256 digests; `SHA256SUMS` binds
all receipt files. Earlier failed and interrupted diagnostics are retained:
the short initial outer bound exited 124, and one readiness interruption has
no verified aggregate exit. Neither is counted as a passing check.

Current-head hosted CI, the long native window runtime matrix, Windows bundle
execution, final combined-main gates, physical hardware and human audit remain
NOT VERIFIED by this receipt. Local host checks and probe compilation do not
substitute for them. The earlier 15-stage native volume and DocumentSession
proof in `../phase0-rust100-cloud-20261118/` remains immutable historical
evidence for its earlier source fingerprint.
