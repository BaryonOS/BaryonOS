# Semantic Window bounds at 100% and 200%

Source: PR #68 head `9d538c20cd5924fd9d72b0dd9963de2b9b906e48`, plus the two source changes identified by Git blob and SHA-256 in `validation.json`.

The root Window rectangle already uses physical viewport dimensions. Scaling it again produced 1606 x 1206 for an 803 x 603 viewport at 200%. Child rectangles are logical and still require scaling. The fix preserves exact physical root bounds, including odd viewport dimensions.

The existing raster/semantic regression now checks both root and editor bounds at 100% and 200%. With only the new assertions and the original renderer it failed (exit 101); with the renderer fix it passed (exit 0). All 20 UI host tests, Clippy with warnings denied, and rustfmt checks passed on the recorded Rust 1.90.0 source snapshot. Exact commands, source blobs, durations and log hashes are in `validation.json`.

This is host regression evidence. No new guest execution, screen reader integration, physical hardware test, human accessibility acceptance or product criterion is claimed. GitHub checks on the resulting commit remain a separate merge gate. The existing roadmap remains 34/100.
