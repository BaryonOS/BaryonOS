# Change report — template

**Result:** state what was actually achieved, not merely what was intended.

**Context:** branch/commit, target architecture and relevant assumptions.

**Changes:** concrete files, behaviour and rationale; no invented commits.

**Sources:** title + revision + W-ID, or book page + B-ID; additional primary source where needed.

| Check | Exact command/environment | Actual result | Evidence |
|---|---|---|---|
| Baseline | Enter the command executed | Not run / passed / failed | Log or reason |
| New tests | Enter the command executed | Not run / passed / failed | Log or reason |
| Emulator/hardware | Only if actually used | Not run / passed / failed | Log or reason |

**Risks and untested areas:** identify remaining gaps without false certainty.

**Rollback:** describe how to reverse this change without erasing other work.

**Baryon OS progress (only where applicable):** previous score, demonstrated gate,
new or unchanged score, and the existing assessment rubric used.
