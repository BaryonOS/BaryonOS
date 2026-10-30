# Verification

Basis `ca45a7dabe6a040921de03bdc2fad3e5045a75ba`, Rust 1.90.0, crate-local
lockfile, own `target/`. Logs and exit codes live next to this file;
`SHA256SUMS` binds sources, lockfile and logs (`sha256sum -c SHA256SUMS`
from `drivers/virtio-snd`).

```sh
cargo +1.90.0 fmt --check
cargo +1.90.0 clippy --offline --locked --all-targets -- -D warnings
cargo +1.90.0 test --offline --locked
cargo +1.90.0 build --offline --locked --release --target x86_64-unknown-none
```

Evidence boundary: host unit tests and no_std crosscompile. No QEMU guest,
audio output, transport, physical hardware or human audit. The 6 prototype
tests are carried over (annotated in `src/tests.rs`) and extended with
feature, range/overflow, multi-record and reader-bounds negatives.

## TASK-H03-TRANSPORT-MODEL-1 (basis `823cba3eb38786d39726ecfb98d6ce7f258754f7`)

`src/transport_model.rs` + `src/transport_model_tests.rs`: pure, bounded host
model of `TRANSPORT.md` §4–§6, linked only under `#[cfg(test)]` (no
production export; the `x86_64-unknown-none` rlib does not contain it). No
deps, Cargo or lock change, no unsafe, no I/O, no VM.

Invariants (each with a red mutant, `model-mutants.py` → `model-mutants.log`,
7/7 caught, source sha256 restored):

| Inv | Rule | Mutant |
| --- | --- | --- |
| I1 | slot reuse only after fence = status 0 **and** BME off read back; frozen (no submit/control/selective completion) while `Fencing` | M1 |
| I2 | stale lease (epoch/PID/cap generation) refused without state change; completion or used-index change after both ACKs ⇒ `Quarantined` | M2 |
| I3 | fence deadline miss, unarmed internal fence, contradicting read-back, deadline/epoch overflow ⇒ `Quarantined`, pool stays pinned | M3 |
| I4 | no submit/start after revoke; reinit needs a fresh lease (higher cap generation, current epoch) | M4 |
| I5 | rx queue never gets buffers | M5 |
| I6 | double completion never frees twice (⇒ fence); double RELEASE refused (F19) | M6, M7 |

<!-- coderabbit-ai-generated:v1 -->

<!-- coderabbit-ai-generated-section:v1:ee1ce81fcd66e7e008e2285c4761d040c9401eb591b2115a84dbdd72d0bc830a -->

Gates rerun on the review fixes based on `9768d8c502fc9c7f51c7a0a5cf2bdd5b9d171efe` (logs/exit `model-{fmt,clippy,tests,build,mutants}`): fmt --check,
clippy `--all-targets -D warnings`, `test --offline --locked` (33 = 11 codec +
22 model, including `i4_invalid_revoke_deadline_quarantines_without_releasing_slots`),
none release build, `python3 verification/model-mutants.py`; all exit 0. The
mutant runner first requires passing baseline model tests and accepts only
confirmed test failures as caught. Its error handling is covered by
`python3 -m unittest verification/test_model_mutants.py` (6 tests, exit 0).

<!-- /coderabbit-ai-generated:v1 -->

Evidence boundary: status 0 and BME-off read-back are **model assumptions**
about a conforming device, not proven DMA quiescence or a device fence. No
production lease/fence/IRQ/MMIO/DMA/root contract is adopted; B1–B9 review
stays separate. Not H03 acceptance, no points.
