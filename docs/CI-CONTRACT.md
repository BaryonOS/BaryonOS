# CI contract: impact catalogue and selector

Status: phase 2b of issue #494 (current). **Phase 2b does not change which CI
runs.** The advisory `ci-impact-shadow` job in `ci.yml` runs the selector on
every event and preserves the plan and a catalogue coverage report; no job,
required check or merge-hold reads either. Existing triggers and required
gates behave as before, except that two positive path filters were widened
(see "Shadow-mode wiring"). Phase 2a (PR #499) added the data model and the
selector described below.

Files:

- `.github/ci-impact.toml`: the catalogue (data only).
- `tools/ci_impact.py`: the selector, one self-contained standard-library file.
- `tools/tests/test_ci_impact.py`: unit tests with temporary git repositories.

## Invariants

- A plan never narrows on doubt. Every runtime error, missing history, invalid
  trusted catalogue, central input or unknown path yields `mode = "full"` with
  a `fallback` code, and `plan` exits 0. Only usage errors exit 2.
- Full mode selects only automatic `core` and `rwa` executions whose workflow
  has the event. It never selects `manual`, `operational`, `advisory` or
  `publication` executions.
- Policy comes from the trusted base: the catalogue is read from the base
  commit, never from the change under test.
- Selection is separate from evidence collection (`tools/ci_evidence.py`).
  Selection decides before anything runs and widens on error; evidence
  collection runs afterwards and fails the job on error.

## Classes and obligations

| Class | Meaning | Obligation |
|---|---|---|
| `core` | Always runs on its events (host-quality, qemu-smoke, merge-hold, CodeQL `analyze`). | Must be automatic and unconditional. Always selected. |
| `rwa` | Required when affected. | Needs a workflow `obligation` citation and inputs (`cargo`, `inputs` or `input_sets`). Selected when an input changes; always selected in full mode. |
| `advisory` | Informative automatic job. | Never required; never selected. |
| `manual` | Runs only on dispatch or an opt-in label. | Never automatic; never selected. |
| `operational` | Repository maintenance (schedules, queue cleanup). | Never selected. |
| `publication` | Publishes or uploads (coverage upload, history audit). | Never selected; not acceptance evidence. |

A `core` or `rwa` job must not need, directly or through other jobs, a job
of a class that is never selected: GitHub would skip the selected gate when
its prerequisite does not run. `validate` rejects such a catalogue; no real
job does this today.

`experimental-m3-m4-smp` in `native-smp.yml` is automatic and blocking despite
its name, so it is registered as `rwa`. `windows-qemu-test-version.yml` is
registered as `advisory`; the project owner may promote it.
`native-network-slirp` in `net-broker-quality.yml` depends on the runner's real
DNS and internet access and is non-required by design, so it is registered as
`advisory` too.

## Catalogue schema (version 1)

Top level: `schema_version = 1`, integer `revision` (bump on every change),
`[policy]`, `[input_sets]`, `[[crate_escape]]` and `[[workflow]]`. Unknown keys
are rejected.

`[policy]`

- `mode`: `shadow`, `enforce`, `select` or `full`. `full` is the rollback
  switch: every plan becomes full with fallback `policy-rollback`.
- `always_full`: globs whose change forces a full plan (all of `.github/**`,
  every `Cargo.lock`, `rust-toolchain.toml`, root `Cargo.toml`,
  `sdk/Cargo.toml`, `.gitattributes`, the build helpers, the selector and the
  catalogue). `validate` requires the tool configuration that cargo, rustfmt
  and clippy read from any ancestor directory, in root and `**/` form:
  `rustfmt.toml`, `.rustfmt.toml`, `clippy.toml`, `.clippy.toml`,
  `rust-toolchain*` and `.cargo/**`.
- `inert`: metadata globs; equal to the workflows' metadata `paths-ignore`
  and referenced from triggers as `"@metadata"`.
- `core_only`: globs read only by the always-running core gates.
- `[policy.events]`: one entry per event. `merge_group`, `push` and
  `schedule` must be `full`; `validate` rejects anything else, and the
  selector also plans these three events as full whatever the catalogue
  says. `workflow_dispatch` must be `full` or `manual`. `pull_request` may
  be `select`, `full` or `manual`.

`[input_sets]`: named glob lists. `native` covers the integrated QEMU suites.

`[[crate_escape]]`: `crate` (directory), `when` (`always`: whenever the crate is
in a closure; `root`: only when it is the gate's own root, for tests and
binaries), `inputs` (globs) and `evidence` (`file:line`).

`[[workflow]]`: `path`, `owner` (required, a non-empty string; `validate`
rejects a workflow without it; the current convention is `owner = "ci-owner"`),
`granularity`
(`variant`, `job` or `workflow`), `events` (exact `on` keys), `obligation`
(a list of `{ cite = "file:line", anchor = "<text on that line>" }`; `cite`
may be a `file:first-last` range) and `trigger.<event>` for `pull_request`,
`push` and `merge_group` with verbatim `types`, `branches`, `paths` or
`paths_ignore`.

`[[workflow.job]]`: `id`, verbatim `name`, `class`, `activation` (`automatic`,
`dispatch`, `schedule`, `label:<name>` or `push-branch:<branch>`), verbatim
`condition` (the job `if`), `needs`, `timeout_minutes`, `max_parallel`,
`cargo` (root manifests), `input_sets`, `inputs`, `evidence`
(preserve-evidence `name` values) and `ruleset_required`.

`[[workflow.job.variant]]` (or an inline `variant = [...]` array): `id`, rendered
`check_name`, verbatim `matrix`, optional `cargo`, `input_sets`, `inputs` and
`evidence`. A runtime matrix is one variant with `id = "*"` and
`dynamic = true`. Its `matrix` is the verbatim table with expression axes, or,
when the workflow gives the whole matrix as one expression (for example
`matrix: ${{ fromJSON(needs.prepare.outputs.matrix) }}`), that expression
string; `coverage` compares either form with the workflow. A static variant's
`matrix` must be a table.

Workflow paths are `.github/workflows/<id>.yml` (or `.yaml`) with an id of
`[A-Za-z0-9_][A-Za-z0-9_-]*`; job ids match `[A-Za-z_][A-Za-z0-9_-]*` and
variant ids `[A-Za-z0-9_.*-]+`. The step summary escapes every table cell
(`&`, `|`, `<`, `>`, `[`, `]` and backticks become HTML entities), so even a
hostile head catalogue listed in a bootstrap plan cannot inject Markdown or
HTML.

Execution ids are `<workflow stem>#<job id>[/<variant id>]`, for example
`component-leaves#component-quality/image-viewer`. Execution ids and rendered
check names are unique across the catalogue.

Globs use the GitHub path-filter dialect restricted to `*` (within one path
segment) and `**` (across segments); `validate` rejects `?`, `[`, `]`, `+`, a
leading `/` or `!`, backslashes and `.` or `..` segments. As in GitHub, a `**/`
at the start of a segment matches zero or more whole directories:
`**/rustfmt.toml` matches `rustfmt.toml` and `a/b/rustfmt.toml`, and
`a/**/b.rs` matches `a/b.rs`. A single `*` never crosses `/`. The root forms
that `validate` also requires are therefore redundant but harmless.

## Input derivation

An execution's inputs are its `inputs`, its input sets and the Cargo closure
of its `cargo` roots. The closure is derived, not listed:

- path dependencies in `[dependencies]`, `[dev-dependencies]`,
  `[build-dependencies]` and their `target.*` forms, including optional ones,
  each contributing `<crate dir>/**`;
- manifest target paths outside the crate (`lib`, `package.build`, and for
  roots also `bin`, `test`, `example` and `bench`);
- registered crate escapes for `#[path]`, `include_str!` and `include_bytes!`;
- every ancestor `Cargo.toml` of a root directory up to the repository root,
  because cargo reads it as a potential workspace root.

A declared path dependency contributes `<dir>/**` even where its manifest is
absent. A root manifest absent at the merge base, base and head gives
`cargo-manifest-missing`; a git read error (not absence) gives
`git-read-failed`.

The closure is the union over the merge base, the base tip and the head, so an
added or removed dependency is followed either way. `workspace = true`
dependencies and `[patch]` or `[replace]` tables are not resolved and force a
full plan (`cargo-workspace-inheritance`, `cargo-patch-unresolved`). An
explicit `[package] workspace = "<path>"` in any closure manifest forces a full
plan too (`cargo-explicit-workspace`), because that workspace root can lie
outside the ancestor walk; no tracked manifest sets it today. Cargo's legacy
`[project]` alias of `[package]` forces a full plan as well
(`cargo-legacy-project-table`). Ancestor
manifests are read at every revision too, because a workspace root's
`[patch]` or `[replace]` applies to all its members.

## Plan algorithm and precedence

1. Event: `workflow_dispatch` with a base and a `manual` policy gives
   `mode = "manual"` (nothing selected).
   `merge_group`, `push` and `schedule` are full by policy (`merge-group`,
   `push-main`, `schedule`); the diff is still recorded. A push whose `before`
   is empty, all zeros or not an ancestor gives `push-unknown-before`.
   A `workflow_dispatch` or `schedule` run without a base has no trusted
   catalogue: the plan is full with `no-trusted-base`, `selector.trusted` is
   false, and the head catalogue only lists the executions.
2. Trusted policy: the base commit must contain both the catalogue and
   `tools/ci_impact.py`; otherwise the plan is full with
   `bootstrap-no-base-catalogue` (the introducing pull request) and lists the
   executions from the head catalogue for information. An invalid base
   catalogue gives `invalid-trusted-catalogue`.
3. History: missing commits are fetched with `--depth=1`; the merge base is
   searched with bounded `--deepen` steps. Failures give `fetch-failed` or
   `missing-history`. More than one best merge base (a criss-cross history,
   `git merge-base --all`) gives `ambiguous-merge-base`. In a shallow
   repository a single visible merge base may hide a newer one behind the
   shallow boundary, so it is accepted only after `git fetch --unshallow`
   succeeds and the repository is complete; the merge base is then
   recomputed. Otherwise, or with `--no-fetch`, the plan is full with
   `shallow-history`. For a pull request, a tested merge commit must have the
   base and head as parents (`merge-ref-mismatch`).
4. Diff: `git diff --name-status -z --find-renames <merge base> <head>`.
   Renames and copies contribute the old and the new path, deletions the old
   path. An unknown status or more than 5000 paths gives full.
5. Classification of each changed path, in order: `always_full` (full,
   `always-full`), any execution input (selects it), `core_only` or `inert`
   (nothing), otherwise `unknown-path` with reason `unknown-path:<path>`.
6. Closure: core executions are always selected; `needs` pull in upstream
   jobs; `job` granularity selects all variants of a job; `workflow`
   granularity selects every automatic execution of the workflow.
7. Filter visibility: each selected execution reports `blocked_by_filter`
   when its own workflow path filter would keep it from starting.
   `filter_eval_uncertain` is set for zero or more than 300 changed files;
   `blocked_by_filter` is then `null` (unknown) for every filtered workflow,
   never a definite `true` or `false`.

## Plan schema

`baryonos.ci-impact.plan`, version 1, canonical JSON with sorted keys and
ASCII escapes (control characters in paths never reach logs raw):

- `catalogue`: `path`, `source_sha`, `blob`, `revision`;
- `selector`: `blob` of the executing file, `base_blob`, `trusted` (equal);
- `event`, `policy_mode`, `mode` (`select`, `full` or `manual`);
- `fallback`: `null` or `{code, detail, reasons}`;
- `shas`: `base`, `merge_base`, `head`, `tested`;
- `changes`: `count`, `digest` (`sha256` over the sorted, NUL-joined paths),
  `paths` (at most 1000);
- `selected`: `id`, `workflow`, `job`, `variant`, `check_name`, `class`,
  `reasons` (at most three `[glob, path]` pairs), `reason_count`,
  `blocked_by_filter` (`true`, `false` or `null` for unknown);
- `excluded`: `id`, `class`, `reason` (class name, `not-automatic`,
  `event-not-applicable`, `no-input-match` or `manual-event`);
- `plan_id`: `sha256` over the schema version, catalogue blob and revision,
  selector blob, event, mode, fallback code, SHAs, changes digest and selected
  ids. The same inputs always give the same id.

In full mode `selected` is informational; `mode = "full"` itself means every
automatic core and rwa gate runs.

When the planner itself fails (`selector-error:<Type>`) or an output cannot
be written (`output-error:<Type>`), the command emits an emergency plan with
every field above: `mode = "full"`, `selector.trusted = false`, the SHAs from
the command line, no listed executions and `filter_eval_uncertain = true`.

## Commands

```sh
python3 -B tools/ci_impact.py validate .github/ci-impact.toml   # exit 0 valid, 1 invalid
python3 -B tools/ci_impact.py plan --event pull_request \
  --base-sha "$BASE_SHA" --head-sha "$HEAD_SHA" --out plan.json
python3 -B tools/ci_impact.py coverage   # JSON drift report, always exit 0
```

Optional `plan` flags: `--tested-sha "$MERGE_SHA"` (the commit under test,
default the head), `--summary "$GITHUB_STEP_SUMMARY"` (append a Markdown
summary), `--repo <dir>`, `--catalogue <path>` and `--no-fetch`. `--out`
defaults to standard output. The summary is written first and the plan file
is published last, atomically (temporary file, then rename). If any output
fails, the published plan is the full `output-error:<Type>` emergency plan,
never the computed one. If the emergency plan cannot be published either, the
file at `--out` is removed, or else overwritten in place with the emergency
plan (`O_NOFOLLOW`: a symlink at `--out` is never written through); if neither
works, `ci-impact: stale-plan-not-removed: <path>` goes to
standard error. The emergency plan goes to standard output and the exit status
stays 0. A consumer must still check that the plan's `shas.tested` matches its
own run before it uses a plan file.

`coverage` needs PyYAML and otherwise reports `coverage-unavailable`. It
compares the catalogue with `.github/workflows/*.y*ml` (workflows, events,
trigger filters, jobs, names, conditions, needs, timeouts, max-parallel,
rendered matrix variants and check names, evidence names; for a dynamic
matrix, the unexpanded evidence name templates). It also checks:

- that no workflow mapping repeats a key (`workflow-duplicate-key`); PyYAML
  would otherwise keep only the last value silently. The comparison then
  continues on that last-wins parse;
- that each job's `activation` agrees with the workflow's `on:` keys; a
  `schedule` or `dispatch` job in a workflow that also has `pull_request`,
  `push` or `merge_group` needs a condition such as
  `github.event_name == 'schedule'` (`activation-mismatch`);
- obligation citations: the anchor must occur on the cited line or range;
  within five lines it reports `citation-moved` with the new line, otherwise
  `citation-anchor-missing` or `citation-missing`;
- that Cargo roots exist, and scans the git-tracked `.rs` files of closure
  crates for unregistered source escapes (untracked build output such as
  `target/` is ignored);
- that no tracked symlink (mode 120000) lies in an execution's inputs
  (`symlink-input`), since its target can change without a diff inside them;
- every `git ls-files` path is classified.

It is a report, not an enforcement. From phase 2b the `ci-impact-shadow` job
runs it on every event and preserves `coverage.json`; catalogue drift does not
fail any gate, including `host-quality`, before phase 3.

## Registering changes

- **New workflow:** add a `[[workflow]]` with its events and triggers, and one
  `[[workflow.job]]` per job with a class and activation. An `rwa` job needs an
  obligation citation and inputs.
- **New job or matrix variant:** copy `name`, `if`, `needs`,
  `timeout-minutes`, `max-parallel`, the matrix entry, the rendered check name
  and the preserve-evidence names verbatim.
- **New crate escape:** when a closure crate compiles a file outside its
  directory through `#[path]`, `include_str!` or `include_bytes!`, add a
  `[[crate_escape]]` with the target glob and the `file:line` evidence. A
  non-literal argument widens to the directory of its literal prefix plus
  `/**`.
- **New tracked path outside every input:** add it to an input, `core_only` or
  `inert`; otherwise every plan that touches it is full.

Then run `validate` and `coverage`; both must be clean, and bump `revision`.

## Shadow-mode wiring (phase 2b)

One job records the plan: `ci-impact-shadow` in `ci.yml`. A path-filtered
workflow cannot record its own plan, because it never starts on filtered
changes; `ci.yml` has no path filters. `merge-hold.yml` cannot host the job,
because its contract pins a single job.

**Job.** `ubuntu-24.04`, `timeout-minutes: 10`, `permissions: contents:
read`, no `needs`, no `if`, no consumer, not required. `actions/checkout`
uses the same pinned SHA as `host-quality` with `persist-credentials: false`
and `fetch-depth: 0`. Full history means the merge base never depends on a
shallow clone, so `missing-history` and `fetch-failed` indicate a real
problem rather than a clone depth. The pinned Python, with the host
dependencies (`host-dependencies: 'true'`, which provides PyYAML for
`coverage`), comes from `./.github/actions/setup-python`. The steps are:

1. `./.github/actions/select-ci` writes `build/ci-impact/plan.json`.
2. A report-only step runs `coverage` from the checkout and writes
   `build/ci-impact/coverage.json`, plus its status and at most 50 problems
   in the step summary. It never asserts the status.
3. `./.github/actions/preserve-evidence` preserves both files as
   `ci-impact-plan` (compact upload `ci-impact-plan-reports`, three-day
   retention, docs/CI-EVIDENCE.md), with `if: ${{ always() }}`,
   `if-no-files-found: error` and only the plan as a required path.

**Inputs.**

| Event | `tested-sha` | `head-sha` | `base-sha` |
|---|---|---|---|
| `pull_request` | `github.sha` (merge ref) | `pull_request.head.sha` | empty; the action uses the merge ref's first parent |
| `merge_group` | `github.sha` | `merge_group.head_sha` | `merge_group.base_sha` |
| `push` | `github.sha` | `github.sha` | `github.event.before` |
| `workflow_dispatch` | `github.sha` | `github.sha` | empty |

On `pull_request` the base is the first parent of GitHub's merge commit, not
the payload's `pull_request.base.sha`, which can name an older base tip and
would then give `merge-ref-mismatch`. `github.event.before` is passed only on
`push`; on a `pull_request` `synchronize` it names the previous pull request
head, not the base.

**Action `.github/actions/select-ci`.** A composite action with inputs
`event-name`, `tested-sha`, `head-sha` and `base-sha`, and outputs `plan-id`,
`mode` and `plan-path` (`build/ci-impact/plan.json`).

- Every `${{ }}` value reaches the shell only through `env:`; no `run:`
  block contains an expression.
- A base is used only when it matches `^[0-9a-f]{40}$`. When the base commit
  contains `tools/ci_impact.py`, the action writes that blob with
  `git cat-file blob` to `$RUNNER_TEMP` and runs it with `python3 -B -I`.
  The trusted selector then reads the catalogue from the base commit itself.
- Without a base copy (no base, an all-zero `before`, or a base before the
  selector existed), the checkout copy runs. The selector reports
  `no-trusted-base`, `push-unknown-before` or `bootstrap-no-base-catalogue`,
  and the action forces its `mode` output to `full` whatever that copy
  writes.
- Arguments use the `--flag=value` form, so a value that begins with `-`
  cannot be parsed as an option.
- The step fails only on a selector usage error (exit 2), an unreadable
  selector or a plan without a valid `plan_id` and `mode`. Selector
  uncertainty is a full plan with exit 0.
- The step summary carries the selector's Markdown summary plus the selector
  source (`base` or `checkout`).

**Widened filters.** `storage-resilience.yml` and `volume-acceptance.yml`
use the shared `setup-python`, `setup-qemu` and `preserve-evidence` actions.
Their positive `pull_request` paths now include `.github/actions/**`; the
literal `.github/actions/preserve-evidence/**` entry stays, because the
historical evidence verifier records it. `storage-resilience.yml` also
includes `sdk/**`, since the kernel build compiles files from
`sdk/examples`. Widening only starts these gates more often.

**Host-quality checks.** `tests/test_ci_workflows.py` (run by
`host-quality`) checks the job and action contract: independence, read-only
permissions, the checkout pin, the action inputs and outputs, expressions only
through `env:`, the report-only coverage step and the preservation call. It
also runs the action's script against a temporary repository: a trusted base
copy gives `select` while the checkout copy is broken; no base gives `full`; a
checkout copy that claims `select` without a base is forced to `full`; a
malformed base gives `full`. It does not run `coverage`, so catalogue drift
never fails `host-quality` in phase 2b; it appears in the shadow job's
coverage report.

## Known limits before enforcement

Phase 2a and 2b plans are shadow information. The following gaps are known
and must be closed, or shown not to apply, before phase 3 (enforce) or phase
4 (select) relies on a plan:

- **Only the selector and catalogue are base-trusted.** On `pull_request`,
  the `ci-impact-shadow` job and `.github/actions/select-ci` come from the
  pull request side; only `tools/ci_impact.py` and `.github/ci-impact.toml`
  are read from the base commit. The coverage step also runs the checkout
  copy. A pull request can therefore change how the plan is produced,
  reported and preserved. This is acceptable while nothing consumes the
  plan. Phase 3 must recompute or verify the plan from base-side code (for
  example in merge-hold) and must not trust the pull-request-side action
  outputs or the preserved artifact.
- **The plan command line spans one base/head generation.** The action from
  the head calls the selector from the base, so a pull request that adds a
  `plan` flag to both makes the base selector fail with a usage error (exit
  2). A new flag lands in the selector first and is used by the action only
  after that selector is on `main`.
- **Shadow plans are not acceptance evidence.** A preserved plan records what
  the selector would have chosen; it proves no gate result.

- **Event applicability ignores the target branch.** `applicable` checks the
  event name and, for `push`, a `branches` list containing `main`. It does
  not compare `pull_request` or `merge_group` branch filters with the pull
  request's base branch, nor a pushed ref with the filter. A stacked pull
  request onto a non-`main` branch can therefore be planned with a gate such
  as `codeql.yml` (`branches: [main]`) that GitHub will not start.
- **The escape scanner sees only literal compile-time paths.** `scan_escapes`
  finds `#[path = "..."]` and `include*!` arguments. It does not model
  runtime paths built from `env!("CARGO_MANIFEST_DIR")` with `join("..")`,
  nor `concat!(env!("CARGO_MANIFEST_DIR"), "/../x")`. Examples:
  `services/nas/native-volume/tests/volume.rs:206` runs a script outside the
  crate, and `tools/update-tool/tests/roundtrip.rs:305` reads
  `../update-keys`. Such inputs must be registered as `[[crate_escape]]`
  entries by hand until the scanner, or an equivalent check, covers them.
- **A root-scoped escape of a library crate is not attached to dependents.**
  `cargo_closure` adds a `[[crate_escape]]` with `when = "root"` only to a gate
  whose own root is that crate, but `coverage` accepts any registration whose
  target glob matches, whatever its `when`. When a library crate's source
  leaves the crate and the escape is registered only with `when = "root"`, a
  gate that uses the crate as a dependency does not receive the target as an
  input; if another gate covers the file, a change to it alone omits that
  gate. Library escapes must use `when = "always"`, and `coverage` must report
  a scope mismatch, before a plan is enforced.
- **Job-level `if` is not evaluated.** The planner selects a job and its
  `needs` prerequisites without evaluating their job-level `if` for the event.
  When a prerequisite's condition is false, GitHub skips it and then skips the
  dependent selected job unless that job's own `if` overrides the skip, while
  the plan still lists the job as selected. The planner must evaluate the
  condition for the event, or reject such a selection, before merge-hold
  enforcement reads the plan.

Both of the last two limits must be closed before merge-hold enforcement
(phase 3).

## Phases

- **2a (PR #499, merged):** catalogue, selector, tests and this contract. No
  CI wiring.
- **2b (current):** the `ci-impact-shadow` job and the `select-ci` action run
  the trusted base copy of the selector on every event and preserve the plan;
  a report-only step preserves the catalogue coverage report. Shadow plans
  and coverage are recorded, never enforced.
- **3 (enforce):** per-workflow `select` and `final` jobs and merge-hold verify
  that every selected gate passed, bound to the `plan_id`.
- **4 (select):** workflows run only the selected executions.
- **Rollback:** setting `policy.mode = "full"` in the catalogue on the base
  branch makes every plan full immediately.
