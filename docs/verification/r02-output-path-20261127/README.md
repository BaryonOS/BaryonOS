# R02 diagnostic output path regression

The diagnostic runner now resolves a newly created `R02_OUTPUT_DIR` before
changing worktrees. The isolated test uses a relative output path and a home
path containing spaces, and an output path ending in a newline, with stub builds; it performs no native or QEMU run.

Run `python3 -B docs/verification/r02-output-path-20261127/test_output_path.py`.
The test failed on source `3009755b` with missing relative log directories
and passed after the path-resolution change. The earlier raw diagnostic
reports and hashes remain unchanged.
