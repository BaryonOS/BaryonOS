# Codex: short development loop that preserves evidence

AGENTS.md takes precedence. This is an execution guide, not a second roadmap,
security proof or guarantee about speed. Current project status and criteria
remain in docs/PROJECT-STATE.md and docs/progress.json.

## 1. Resuming without starting over

First check the actual checkout; the last branch or chat history discussed
need not contain the current code:

```sh
git status --short
git branch --show-current
git rev-parse HEAD
git log -5 --oneline
```

Read the newest status block, the relevant contract specification and nested
AGENTS.md files. Review only the diff since the previous verified checkpoint.
A missing checkpoint or unexpected change calls for targeted investigation,
not blind trust in an old test claim. Uncommitted local work by Claude does
not automatically appear in another cloud task.

Choose the next feasible task from the existing project status/roadmap.
Briefly record: **goal; dependency; write files; acceptance command;
evidence boundary**. First check whether the implementation or PR already exists.

## 2. Targeted feedback first, integration afterwards

Token Guard preserves command exit codes and complete local logs while bounding
the displayed context. Use it for suitable foreground text checks; interactive,
protocol, background and TTY-sensitive commands keep their native execution path.
Run the applicable examples below through
`python3 -B tools/tokenguard.py run -- <command>`. For Git diffs and other useful
data, select `run --output bounded --`; inspect any explicitly omitted ranges.
The shared plugin and client setup are described in `docs/TOKEN-GUARD.md`.

These are examples from the existing CI, not a new definition of all required
tests. Choose the suite that the change actually affects. For a change in a
shared crate, the relevant consumers must also be tested.

```sh
# Example: a change in volume-core.
cargo +1.90.0 test --locked -p volume-core
cargo +1.90.0 clippy --locked -p volume-core --all-targets -- -D warnings

# SDK: separate workspace.
cargo +1.90.0 test --manifest-path sdk/Cargo.toml --locked

# Host harness for DocumentSession.
python3 -m unittest discover -s tests -p 'test_document_session_*.py'

# Product prototype; NAS has a separate suite.
node --test --test-isolation=none ui/prototype/tests/*.test.mjs
node --test --test-isolation=none services/nas/tests/*.test.mjs
```

Check formatting early, before a costly integration run. For integration, use
the current commands from `.github/workflows/` and the relevant native gate;
do not copy an outdated test list from a chat. Keep negative capability,
recovery and fault-injection tests intact. A host test does not prove native
behaviour. Never skip a check because a cache was restored.

## 3. Working in parallel without losing locks or integration

Use the agent limits from AGENTS.md; more agents is not a goal in itself.
Split off only genuinely independent tasks, with one owner per interface and
one integrator. Share the contract and the relevant code, not all historical
documentation every time. A reviewer can follow along read-only.

Every writer gets its own worktree/branch. Native fixtures use their existing
isolated `NOS_BUILD_DIR`, `NOS_TARGET_DIR` and `CARGO_TARGET_DIR`; do not mix
profiles or images. Do not let two Cargo processes block the same target
directory. Prevent agent parallelism and unbounded build parallelism from
exhausting CPU and memory at the same time. Scale up only when measured lead
time drops and integration work stays manageable.

## 4. What the CI cache does and does not reuse

The main workflow keeps `host-quality`, `qemu-smoke`, their order, all
existing commands, timeouts and read permissions. Other acceptance workflows
are unchanged.

- Both jobs may reuse Cargo registry indexes and crate archives, separated by
  runner OS, architecture and the contents of all Cargo.lock files.
- Only `host-quality` reuses the explicitly listed **host-debug** target
  directories. Its key additionally covers the toolchain, manifests, Cargo
  config and build scripts. There are no broad restore prefixes to other keys.
- No credentials, full home directory, native release targets, QEMU images,
  keys, reports or acceptance evidence in these caches.

`actions/cache` is pinned to the reviewed v6.1.0 commit
`55cc8345863c7cc4c66a329aec7e433d2d1c52a9`. The upstream `v6.1.0` tag
points directly to this complete commit, whose signature GitHub reports as
valid. `action.yml` uses Node 24; `package.json` declares version 6.1.0,
Node `>=24`, and the MIT license. The minimum runner version therefore remains
2.327.1. This is a CI dependency, not an OS runtime library. Maintain this pin
through the existing review and dependency process; this metadata and tag
verification is not an audit of all bundled action code.

According to the upstream changelog, version 6.1.0 handles read-only cache
access and demotes a redundant cache-save warning to debug level. This does
not expand token permissions: the cache remains best-effort, and an
unavailable save does not constitute a successful build or test step.

The cache is best-effort. A miss runs the normal build. Cache keys are
immutable: a hit does not mean the newest project code has already been
compiled. Cargo and all test commands are invoked again. The workflow
summary shows cache-hit diagnostics, not a passed test result. The mandatory
clean native test directories are not adjusted to work around the cache.

## 5. Measure before claiming speed

Use existing job and step timings in GitHub Actions. Preferably compare
several runs with the same source, suite, runner class and tool versions:
first without a matching cache, then with cache. Record cache hit/miss, wait
time, build/test duration, exit status and work that had to be redone. Do not
compare a smaller test suite with a full native gate as if only execution got
faster.

A local Linux measurement can, for example, be taken like this, without a
pipeline that hides a failure:

```sh
/usr/bin/time -p cargo +1.90.0 test --locked -p volume-core
```

Use the same session and valid environment for related follow-up work.
A GitHub change does **not** automatically configure the published Codex
cloud environment, model choice, budget or personal config.toml.
Avoid reinstallation, routine `cargo clean` and unnecessary full audits.
A cache that turns out to cost more than rebuilding can be removed again.

## 6. Handoff and recovery

Update the existing PROJECT-STATE.md and progress assessment according to
AGENTS.md. Use one compact `NEXT AGENT HANDOFF` there, for example:

```text
Source: <branch + full HEAD, base and relevant PR>
Goal/result: <one verifiable implementation step>
Open work: <files, owner, changes not yet integrated>
Evidence: <exact command, exit status, log path; or NOT EXECUTED>
Blocker: <cause and required authority/dependency>
Next step: <one directly executable command or implementation task>
```

No extra roadmap, fictitious agent work or artificial score increase. This
workflow optimisation does not itself accept a new OS criterion. For a
caching problem: first repeat the full affected check without cache; if
necessary, raise only the cache version prefix in a reviewed change. Preserve
test boundaries and security. Rolling back is possible with a normal revert
of the workflow change, without resetting existing branches.

## Sources

GitHub. (n.d.). *Dependency caching reference*.
https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching

GitHub. (2026). *actions/cache*, version 6.1.0.
https://github.com/actions/cache/tree/55cc8345863c7cc4c66a329aec7e433d2d1c52a9

GitHub. (June 26, 2026). *actions/cache v6.1.0 release notes*.
https://github.com/actions/cache/releases/tag/v6.1.0

OpenAI. (n.d.). *Using Codex Cloud*.
https://help.openai.com/en/articles/20001545-using-codex-cloud

The cache source, tag, and release metadata were verified on November 10,
2026; the remaining sources were accessed on October 29, 2026. Repository
commands are aligned with base commit
`f63a0c1b56fdcd28e43579aa6e9c5a56e1e2cbdb`; consult the current source files
afterward.
