# Userspace probe rules
Read the root and kernel instructions. Probes are `no_std` ring-3 programs, each with its own linker script, compiled by `tools/build_users.py` (direct `rustc`, not Cargo) and embedded by `tools/build.py` for a kernel feature. Never build them standalone.
A probe proves behaviour only through its native gate (`tools/run_*.py` with the matching `--kernel-feature`). Change its markers or exit codes only together with that gate.
Host tests: `python3 -m compileall -q userspace/tests` and `python3 -B userspace/tests/test_<probe>.py`; `test_smp_fpu_probe.py` runs in `native-smp.yml`.
