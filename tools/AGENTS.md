# Tool rules
Read the root instructions. Tools are Python 3 standard library (PyYAML only for `agent_gate.py`), a few bash scripts and the Rust `update-tool`. Run every tool as `python3 -B` and reuse the existing entry points (`preflight.py`, `build.py`, `run.py`, `run_*.py`, `test_*.py`, `capture_*.py`, `agent_gate.py`) instead of adding scripts; `--help` documents their flags.
`agent_gate.py`, `agent_pipeline.py`, `common.py` and `progress.py` are protected (root Protected files).
Tests: `python3 -B -m unittest discover -s tools/tests -p 'test_*.py'` and `python3 -m compileall -q tools`.
Harnesses fail on a timeout, a missing marker or an unexpected exit status, and never write expected success logs (tests/AGENTS.md). QEMU runs use only disposable images under `$NOS_BUILD_DIR/run/`; parallel native runs need separate `NOS_BUILD_DIR`, `NOS_TARGET_DIR` and `CARGO_TARGET_DIR`. Capture tools write only to a new child directory of `docs/verification/`.
`gate.sh` runs bare `cargo check --lib`, `cargo test` and `qemu-smoke-test.sh`, which is the K06 regression runner, not a quick smoke.
