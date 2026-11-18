# Legacy issue-controller containment — 18 November 2026

The project owner requires issue-driven execution to remain disabled until the full security boundary is verified. Integrated main67d5859d still contains the legacy controller; paused launchers alone do not enforce that requirement at its callable entry points.

This minimal containment adds an unconditional source-level hold before CLI controller construction and every issue transaction/scheduler ingress. It stops selection, L0, execution, gates, publication and automatic cleanup before their side effects. Dry-run and parallel-merge flags provide no override. Existing low-level parsing/test helpers and the manually selected trusted CI gate remain unchanged. Removal requires a reviewed source replacement with the complete verified security boundary; no runtime enable option is introduced.

## Verification

Mocked red regressions fail10 subcases without starting any model or external operation. Four focused tests then pass, including the real isolated CLI with an empty PATH and credential-free environment. All388 complete tools tests pass on committed sourceef75a64d. Strict preflight, standard build and actual M1 smoke pass on that source; QEMUraw33 and M1:PASS. Independent AI review verifies all eight transaction ingresses and eight actual credential-free CLI flag combinations without a blocker. Exact source/log hashes and raw exit codes are in checks.json. Complete local logs remain ignored.

The change enforces an operational hold only. Full355 resource/scratch/process-lifecycle, selected authenticated model-client/offline gate compatibility and final security verification remain blocking. Recovery357 is preserved and unaccepted. No security finding, product criterion or issue is closed; score41/100 is unchanged. Current-head hosted CI/native suites and final metadata review remain required before manual squash.
