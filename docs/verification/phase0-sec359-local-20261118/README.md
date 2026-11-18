# PR359 local containment reconciliation — 18 November 2026

Source `fd0f76bcfc48cda4f0663091179d45b0f880ac65` reconciles the existing PR359 with main `de9b99047138c4fd0015ffc3f874b273b3537484`. All current-main product, UI, design and criterion identities are preserved. The direct issue-selection entry now refuses before filesystem or external-command access, completing the operational hold across all nine class ingresses and the CLI. No flag/configuration override is introduced.

[Checks and exact commands](checks.json) bind the five focused and 404 complete tooltests, strict preflight, build and real M1 smoke to exact source/test hashes. The pre-fix negative control is preserved with exit1. Successful checks exit0; the native guest exits33, with all required markers and four loaded-artifact/firmware hashes rechecked. [Native result](native-result.json), [serial output](serial.txt) and [build manifest](build-manifest.json) retain the normal-profile source-v3 binding. One pre-existing dead-code warning in kernel serial formatting remains in the build log.

Compressed public logs replace only the absolute task-worktree path; raw originals and their SHA256 remain recorded locally. Host-only mock fixtures printed by the tools suite are not native evidence. Independent AI review passed the exact containment source and complete PR diff; it is not a human audit. Current-head hosted checks and resulting-main acceptance remain pending.

This is containment only. Full PR355/357 resource/client/gate isolation, three Daybreak findings and all five issues remain open. Score41/100 is unchanged. The strict governance transition is recorded separately in the owner clarification; no governance work has begun.
