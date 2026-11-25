# CI contract: impact catalogue and selector

Status: phase 4 of issue #494 (current, ADR-CI-SELECTION-20261124 on top of
ADR-CI-GATE-ENFORCEMENT-20261124). `policy.mode = "select"`: on pull requests,
`component-leaves.yml` runs only the selected leaf variants, and
`app-management-quality.yml` and `net-tls-quality.yml` run only when selected
(see "Selection"). Every other workflow still runs everything its triggers
start. Phase 3 introduced the enforcement below, which stays unchanged: the required
`merge-hold` check recomputes the plan from base-side code and blocks until
every applicable automatic gate has passed on the pull request head (see
"Enforcement"). `host-quality` fails on an invalid catalogue or on drift
between the catalogue, the workflows and the sources. Every workflow still
starts on its own triggers and path filters. Phase 2a (PR #499) added the data
model and the selector; phase 2b (PR #501) recorded shadow plans in CI.

Files:

- `.github/ci-impact.toml`: the catalogue (data only).
- `tools/ci_impact.py`: the selector, one self-contained standard-library file.
- `tools/tests/test_ci_impact.py`: unit tests with temporary git repositories.
- `tools/merge_hold.py` and `tools/tests/test_merge_hold.py`: the enforcing
  required check (phase 3) and its tests with a fake API.

## Invariants

- A plan never narrows on doubt. Every runtime error, missing history, invalid
  trusted catalogue, central input or unknown path yields `mode = "full"` with
  a `fallback` code, and `plan` exits 0. Only usage errors exit 2.
- Full mode selects only automatic `core` and `rwa` executions whose workflow
  has the event. It never selects `manual`, `operational`, `advisory` or
  `publication` executions.
- Policy comes from the trusted base: the catalogue is read from the base
  commit, never from the change under test.
- A pull request cannot shrink its own expected gate set: merge-hold runs the
  base commit's `tools/merge_hold.py` and `tools/ci_impact.py`, never the
  pull request's copies.
- "Not applicable" (filtered out, another target branch, a false job
  condition) is reported and never counts as a pass. A plan proves no gate
  result; only the gate's own successful run does.
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

- `mode`: `shadow`, `enforce`, `select` or `full`.
  - `shadow`: merge-hold reports the applicable gates but does not block on
    them (emergency switch for the enforcement itself).
  - `enforce` (phase 3): merge-hold blocks on the applicable gates; workflows
    run everything their own triggers start.
  - `select` (phase 4): as `enforce`, and workflows with a `select` job skip
    executions the plan does not select.
  - `full`: the selection rollback. Every plan becomes full with fallback
    `policy-rollback`, so every workflow that its own path filter starts runs
    everything (see "Tooling metadata"), while merge-hold still verifies every
    applicable gate.
- `[policy.budgets]` (required): `pull_request_wait_minutes` and
  `merge_group_wait_minutes`, integers from 0 to 45 (below the 50-minute
  merge-hold timeout). merge-hold reads them from the trusted base.
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
- `"@nonnative"` (#494, phase 5 C2): `inert` followed by `core_only`, in that
  order, as one `paths_ignore` macro. `validate` accepts it only on the
  `pull_request` trigger of a workflow whose `push` trigger exists, applies to
  `main` and uses `"@metadata"`: that push run is the post-merge backstop.
  When a workflow uses it, `validate` also rejects a `core_only` pattern that
  names or contains an `always_full` path (each pattern matched against the
  other as a literal path). A partial overlap such as `plugins/**` with
  `**/Cargo.lock` is caught by a host-quality test instead: no tracked path may
  match both lists.
- `[policy.events]`: one entry per event. `merge_group`, `push` and
  `schedule` must be `full`; `validate` rejects anything else, and the
  selector also plans these three events as full whatever the catalogue
  says. `workflow_dispatch` must be `full` or `manual`. `pull_request` may
  be `select`, `full` or `manual`.

`[input_sets]`: named glob lists. `native` covers the integrated QEMU suites.

`[[crate_escape]]`: `crate` (directory), `when` (`always`: whenever the crate is
in a closure; `root`: only when it is the gate's own root, for tests and
binaries), `inputs` (globs) and `evidence` (`file:line`). `root` is allowed
only when the escaping file is compiled for the crate as a root alone: under
`tests/`, `benches/`, `examples/` or `src/bin/`, `src/main.rs`, a declared
`[[bin]]`, `[[test]]`, `[[example]]` or `[[bench]]` path, or a file reached
only from those. Library source needs `always`, even inside `#[cfg(test)]`,
which `coverage` cannot see; it reports `escape-scope-mismatch` otherwise.

`[[workflow]]`: `path`, `owner` (required, a non-empty string; `validate`
rejects a workflow without it; the current convention is `owner = "ci-owner"`),
`granularity`
(`variant`, `job` or `workflow`), `events` (exact `on` keys), `obligation`
(a list of `{ cite = "file:line", anchor = "<text on that line>" }`; `cite`
may be a `file:first-last` range) and `trigger.<event>` for `pull_request`,
`push` and `merge_group` with verbatim `types`, `branches`, `paths` or
`paths_ignore`. `paths_ignore` may name a macro instead: `"@metadata"` or, on
`pull_request` only, `"@nonnative"`; the workflow YAML lists its literal
expansion, which `coverage` compares in order.

`[[workflow.job]]`: `id`, verbatim `name`, `class`, `activation` (`automatic`,
`dispatch`, `schedule`, `workflow-run`, `label:<name>` or
`push-branch:<branch>`), verbatim
`condition` (the job `if`; for a `core` or `rwa` job it must be decidable on
every automatic event of its workflow, see "Job conditions"), `needs`,
`timeout_minutes`, `max_parallel`,
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

### Job conditions

`condition_value` evaluates a small subset of GitHub expressions, not the
full language: `github.event_name` compared with a string literal
(case-insensitive, as GitHub compares strings), `inputs.<name>` as a truth
value (null outside `workflow_dispatch`), `needs.select.outputs.<key>`
compared with a literal (the phase-4 selection output), `!`, `&&`, `||` and
parentheses, optionally inside `${{ }}`. Anything else, including status
functions such as `always()` and `!cancelled()`, `contains()` and other
contexts, is undecidable. A `core` or `rwa` job runs on an event when its own
condition holds and every `needs` prerequisite runs; an undecidable own
condition makes the job undecidable whatever its prerequisites do (a status
function such as `always()` runs a job after a skipped prerequisite). A
runtime-matrix (`dynamic`) check name of a `core` or `rwa` job needs literal
text besides its `${{ }}` parts, so merge-hold's template match cannot match
every job of a run. `validate` rejects a
`core` or `rwa` job whose start is undecidable on any of `pull_request`,
`push` or `merge_group` in its workflow (`condition-undecidable`). The plan
excludes a job whose condition, or a prerequisite's, is false on the event
(`condition-false`), so merge-hold never expects a job GitHub skips by design.

Branch filters (`trigger.<event>.branches`) use the path glob dialect: `*`
within one segment, `**` across segments. Negation (`!`) and `?`, `[`, `]`,
`+` are rejected; `branches-ignore` and tag filters are not modelled.

## Input derivation

An execution's inputs are its `inputs`, its input sets and the Cargo closure
of its `cargo` roots. The closure is derived, not listed:

- path dependencies in `[dependencies]`, `[dev-dependencies]`,
  `[build-dependencies]` and their `target.*` forms, including optional ones,
  each contributing `<crate dir>/**`;
- manifest target paths outside the crate (`lib`, `package.build`, and for
  roots also `bin`, `test`, `example` and `bench`);
- registered crate escapes for `#[path]`, `include_str!`, `include_bytes!` and
  runtime `env!("CARGO_MANIFEST_DIR")` joins;
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

0. Applicability: a workflow applies when it has the event and, if it has a
   `branches` filter for that event, the target branch matches it:
   `--base-ref` for `pull_request` and `merge_group` (without one the filter
   is unknown and the workflow counts as applicable, which only widens a
   listing), `--base-ref` or `main` for `push`.
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
- `event`, `ref` (the `--base-ref`, possibly empty), `policy_mode`, `mode`
  (`select`, `full` or `manual`);
- `budgets`: the trusted base catalogue's `policy.budgets`, or `{}` when the
  catalogue is not the base one;
- `fallback`: `null` or `{code, detail, reasons}`;
- `shas`: `base`, `merge_base`, `head`, `tested`;
- `changes`: `count`, `digest` (`sha256` over the sorted, NUL-joined paths),
  `paths` (at most 1000);
- `selected`: `id`, `workflow`, `job`, `variant`, `check_name`, `class`,
  `reasons` (at most three `[glob, path]` pairs), `reason_count`,
  `blocked_by_filter` (`true`, `false` or `null` for unknown),
  `filter_hidden_inputs` (the sorted non-metadata hit paths of a row whose own
  filter blocks it; every changed non-metadata path when a fallback stopped the
  plan before classification; `[]` otherwise);
- `excluded`: `id`, `class`, `workflow`, `job`, `check_name`, `reason` (class
  name, `not-automatic`, `event-not-applicable`, `condition-false`,
  `no-input-match` or `manual-event`);
- `plan_id`: `sha256` over the schema version, catalogue blob and revision,
  selector blob, event, ref, mode, fallback code, SHAs, changes digest and
  selected ids. The same inputs always give the same id.

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
python3 -B tools/ci_impact.py coverage            # JSON drift report, exit 0
python3 -B tools/ci_impact.py coverage --strict   # exit 1 on drift (host-quality)
```

Optional `plan` flags: `--tested-sha "$MERGE_SHA"` (the commit under test,
default the head), `--base-ref <branch>` (target branch, phase 3; a caller
from another base/head generation may use it only once that selector is on
`main`), `--summary "$GITHUB_STEP_SUMMARY"` (append a Markdown
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
  `github.event_name == 'schedule'`, and a `workflow-run` job needs a
  `workflow_run` trigger (`activation-mismatch`);
- obligation citations: the anchor must occur on the cited line or range;
  within five lines it reports `citation-moved` with the new line, otherwise
  `citation-anchor-missing` or `citation-missing`;
- that Cargo roots exist, and scans the git-tracked `.rs` files of closure
  crates for unregistered source escapes (untracked build output such as
  `target/` is ignored): literal and widened `#[path]`/`include*!` targets,
  `include*!` of an environment variable other than `OUT_DIR` (registered by
  citing its `file:line` in `evidence`), and string literals with a `..`
  segment in the statement of a `env!("CARGO_MANIFEST_DIR")` join, within
  three lines; targets under `build/` or `target/` are output and ignored. It
  reports `escape-unregistered` and, for a `root` registration of library
  source, `escape-scope-mismatch`;
- that no tracked symlink (mode 120000) lies in an execution's inputs
  (`symlink-input`), since its target can change without a diff inside them;
- every `git ls-files` path is classified.

From phase 3, `host-quality` runs `validate` and `coverage --strict` on the
checkout, so an invalid catalogue or any drift that can change a plan fails a
required check (`strict_status`). Obligation citation drift
(`citation-moved`, `citation-anchor-missing`, `citation-missing`) is reported
in `status` and as a warning, but does not fail: citations document why a gate
exists and never change a plan, and their lines move whenever an ADR is added
at the top of docs/DECISIONS.md. The
`ci-impact-shadow` job keeps its report-only copy and preserves
`coverage.json`.

## Registering changes

- **New workflow:** add a `[[workflow]]` with its events and triggers, and one
  `[[workflow.job]]` per job with a class and activation. An `rwa` job needs an
  obligation citation and inputs.
- **New job or matrix variant:** copy `name`, `if`, `needs`,
  `timeout-minutes`, `max-parallel`, the matrix entry, the rendered check name
  and the preserve-evidence names verbatim.
- **New crate escape:** when a closure crate compiles or reads a file outside
  its directory through `#[path]`, `include_str!`, `include_bytes!` or a
  `CARGO_MANIFEST_DIR` join, add a `[[crate_escape]]` with the target glob and
  the `file:line` evidence, with `when = "always"` unless the origin is
  root-only (see the schema). A non-literal argument widens to the directory
  of its literal prefix plus `/**`.
- **New job condition on a `core` or `rwa` job:** keep it within the subset of
  "Job conditions", or `validate` rejects the catalogue.
- **New tracked path outside every input:** add it to an input, `core_only` or
  `inert`; otherwise every plan that touches it is full.

Then run `validate` and `coverage --strict`; both must pass (host-quality runs
them on every pull request), and bump `revision`.

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

1. `./.github/actions/select-ci` (step id `select`) writes
   `build/ci-impact/plan.json`. Its step summary opens with the heading "Plan
   from the enforced catalogue `.github/ci-impact.toml`", and its "Selector
   source" line names the same catalogue.
2. The same action, with `catalogue: .github/ci-impact.candidate.toml` and
   `out: build/ci-impact/plan-candidate.json`, plans the base commit's copy of
   the advisory candidate catalogue (#494, phase 5 C3). Its summary opens with
   "Plan from the advisory candidate catalogue `<catalogue>`", and its "Selector
   source" line states that nothing consumes the plan. The candidate starts
   as a copy of the enforced catalogue and later carries candidate selection
   data, which is activated only on counterexample-free shadow evidence.
   Its header declares the increment state in one line, `# Candidate
   increment: none`. While that line says `none`, a host-quality test requires
   the candidate to be its comment header plus a verbatim copy of
   `.github/ci-impact.toml`, so every catalogue change, revision bumps
   included, is copied in the same pull request. An intentional increment
   replaces `none` with its name and #494 reference.
   No selection or enforcement step reads it: merge-hold, the select jobs and
   the `validate` and `coverage` steps of host-quality ignore it. A host-quality
   unit test does validate the checked-out copy, so an invalid candidate fails
   host-quality, and as a `.github/**` path every edit to it makes the plan
   full. Without a base copy the candidate plan is `full`
   (`bootstrap-no-base-catalogue`).
3. A report-only step runs `coverage` from the checkout and writes
   `build/ci-impact/coverage.json`, plus its status and at most 50 problems
   in the step summary. It never asserts the status. Its condition,
   `${{ !cancelled() && steps.select.outcome == 'success' }}`, depends on the
   enforced plan only. A failed candidate step still fails the job but does not
   skip the report; a failed `select` step skips it as before.
4. `./.github/actions/preserve-evidence` preserves the three files as
   `ci-impact-plan` (compact upload `ci-impact-plan-reports`, three-day
   retention, docs/CI-EVIDENCE.md), with `if: ${{ always() }}`,
   `if-no-files-found: error` and only the enforced plan as a required path.

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
`event-name`, `tested-sha`, `head-sha`, `base-sha`, `base-ref`, and the
optional `catalogue` and `out` (empty by default: the enforced
`.github/ci-impact.toml` and `build/ci-impact/plan.json`), and outputs
`plan-id`, `mode` and `plan-path`. A `catalogue` other than
`.github/<name>.toml` or an `out` other than `build/ci-impact/<name>.json` is a
usage error (exit 2).

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

## Enforcement (phase 3)

**Where.** The required `merge-hold` check (`.github/workflows/merge-hold.yml`)
decides; no new optional summary check is involved. Its checkout has full
history. A load step takes GitHub's merge commit, checks that its parents are
exactly the base tip and the pull request head, and extracts
`tools/merge_hold.py` and `tools/ci_impact.py` from the **base** commit into
`$RUNNER_TEMP`; the decision step runs that copy with `python3 -B -I`. The
selector is the copy next to it, so both come from one generation. The
workflow file itself still comes from the pull request side (see the limits
below). merge-hold.yml may add environment variables, but must keep every one
an older base `merge_hold.py` reads; `tests/test_ci_workflows.py` pins them.

**Plan identity.** merge-hold recomputes the plan with `--event=pull_request`,
the base tip, the head, the merge commit as `--tested-sha` and the pull
request's base branch as `--base-ref`. It fails closed unless the plan
describes exactly those SHAs, event and branch, was computed by the trusted
base selector from the base catalogue, has a usable `mode` and `plan_id`,
lists at least one `core` gate, and did not fall back for a reason that can
leave the listing incomplete (`merge-ref-mismatch`, `invalid-sha`,
`fetch-failed`, `invalid-trusted-catalogue`, `bootstrap-no-base-catalogue`,
`no-trusted-base`, `unknown-event`, `selector-error:*`, `output-error:*`).
Plans published by pull-request-side jobs (the shadow job, phase-4 `select`
jobs) are never trusted; merge-hold's own recomputation is the reference.

**Expected set.** Every selected `core` or `rwa` execution except merge-hold
itself whose `blocked_by_filter` is not `true`. In full mode that is every
automatic `core` and `rwa` gate whose workflow applies. `blocked_by_filter =
null` (filter result unknown, more than 300 changed files) keeps the gate
expected: fail closed. When such a gate has no run, its Pending line states
the cause and the remedy: the filter result is undefined above 300 changed
files (a smaller pull request or a new head clears it), or the changed-file
list is incomplete after a fallback, which the line names (a re-run with a
usable diff or a new head clears it). A closing line states that waiting
does not help (K10). A path filter may hide only metadata: a row with
`blocked_by_filter = true` and a non-empty `filter_hidden_inputs` is a
failure ("its path filter hides registered input(s)"), never "not
applicable" (#494, phase 5 C1). The rule makes `"@nonnative"` self-protecting:
a `core_only` path that is also a registered input of a workflow whose
pull_request filter hides it (for example the token-guard script of
`native-hid-input` and `native-xhci`, which therefore keep `"@metadata"`) is a
hidden input and fails. A row whose hidden hits are all metadata
(`policy.inert`) stays "Not a pass … (path filter)"; that is a documented
limit, for example a compiled vendored `README.md`. Central (`always_full`)
paths are never hits, so a full plan still does not expect a gate whose own
filter excludes the central path. When a fallback stops the plan before
classification (for example `cargo-patch-unresolved`), no hits exist: every
changed non-metadata path then counts as a possibly hidden input, central and
unregistered paths included, so a blocked row fails unless only metadata
changed. A pull request that introduces such a Cargo construct therefore needs
the gate to run or the construct to be modelled; a transient read failure
clears on a re-run. The two allowlist workflows without a push
backstop (`storage-resilience`, `volume-acceptance`) keep their `paths` list,
because the 2026-11-08 verification reads it; central paths outside that list and
new top-level paths therefore still do not start them. Closing that needs a push
backstop or a filter change with its own verification (#494).

**Results.** merge-hold lists the Actions runs for the pull request head
(`/actions/runs?head_sha=<head>&event=pull_request`, every page, by id, with a
check that the count reaches `total_count`). A run counts only with the
pull_request event, that head SHA, the pull request's head branch, this
repository and the pull request's head repository, and the workflow `path`.
The run must also list this pull request's number and base branch among its
pull requests; GitHub fills that list for runs from a same-repository head. A
fork head's runs list no pull request and cannot be bound to a base, so they
never count: a fork pull request fails closed in merge-hold until its commits
are pushed to a branch of this repository. Relaxing this for forks is a
project-owner decision on a trust boundary. Per workflow the
**newest** run decides, by the start of its latest attempt and then by id, so
an older success never masks a newer running or failed run, and a later
re-run of an older run counts. Its jobs come from
`/actions/runs/<id>/jobs?filter=latest` (every page, `run_id` checked), which
reflects the latest attempt after a re-run. The run is read again after its
jobs; if its attempt or status changed in between, it counts as pending. A
job's class comes only from rows of its own workflow. Per expected execution exactly one
job with the rendered check name must have concluded `success`; a runtime
matrix (`id = "*"`) needs at least one matching job, all successful. Then:

- no run yet, or an expected job not started or still queued or in progress:
  **pending**; an expected job that already succeeded counts while the rest
  of its run goes on;
- missing job (also an empty or partly skipped matrix), `failure`,
  `cancelled`, `timed_out`, `skipped`, `action_required`, `neutral`, `stale`,
  or a duplicate name: **failed**;
- a `core`, `rwa` or unregistered job in the newest run of a catalogue
  workflow that the plan did not expect: **pending** while it runs, and
  **failed** if it then fails. A gate GitHub started must finish, so a
  selector miss cannot merge past a running gate.
- a running run of a workflow that has any applicable `core` or `rwa`
  execution: **pending** as a whole, because jobs waiting on `needs` or a
  runtime matrix may not be listed yet; a runtime-matrix row never passes
  while its run is running. A running run of an advisory-only workflow does
  not hold merge-hold.
  Advisory, manual, operational and publication jobs never block, and
  merge-hold's own earlier runs are ignored.

The pull request also fails when its head does not contain the base tip: the
gates then did not test the tree that would merge. GitHub's strict
required-check rule forces such an update anyway.

**Waiting.** With `pull_request_wait_minutes = 0` (the current value, #494
B9) merge-hold makes one full decision as soon as it starts and holds no runner
while gates run. Gates still pending at that decision are listed and fail the
check with "Pending"; a failed gate fails it as before. The visible effect on a
pull request: merge-hold turns red with "Pending" shortly after a push, and
turns green only when the re-check below re-runs it after the gates have
finished. A merge-hold failure whose error lines are all "Pending:" is
therefore expected and needs no action: do not push a fix, wait for the
re-check, and re-run by hand only after every gate has finished. Watchers and
Auto-fix act only on "Failed:" lines. Before B9 the budget was 40 minutes: a merge-hold run then held a
runner while it polled, which over the last 100 `pull_request` merge-hold runs
(5.4 hours, measured on 25 November 2026) came to 1,567 runner-minutes, about
4.8 runners held permanently; 672 of them belonged to runs cancelled by a new
push. A positive budget still works that way: once a minute merge-hold reads
only the head's run listing and makes the next full decision once no gated
workflow run is open or the budget ends; a failed poll counts as open, and
pending gates with nothing left running decide at once. `merge_group` keeps
`merge_group_wait_minutes = 45` and still waits that way.

**Re-check.** `.github/workflows/merge-hold-recheck.yml` runs on
`workflow_run` (`completed`) of every workflow that owns an automatic `core` or
`rwa` job in the catalogue (a gate workflow) and of every other catalogue
workflow with a `pull_request` trigger, merge-hold excluded
(`tests/test_ci_workflows.py` pins the list against the catalogue), for
`pull_request` runs whose head is in this repository. The other workflows
(today the Copilot environment, the Materiaal probe, the security alert summary
and the Windows QEMU bundle) are triggers only: merge-hold counts their runs
too, and a run that changed while merge-hold read it, or a running job that the
base catalogue does not know, leaves merge-hold at "Pending". Its script,
`tools/merge_hold_recheck.py`, re-runs the newest merge-hold run of each listed
pull request whose current head is still the run's head, but only when

- no run of a gate workflow is still open for the head (the triggering run
  counts as finished even if the listing lags). Each open one normally starts
  another re-check when it completes (see Limits for the exceptions). Open runs
  of the other workflows do not hold merge-hold, so the re-check does not wait
  for them; one that ends later starts its own re-check;
- that merge-hold run concluded `failure`, `timed_out` or `cancelled`
  (re-run of the failed jobs; for `cancelled`, of the whole run); and
- its latest attempt started before the triggering run finished. A later
  attempt has normally already seen that result.

A merge-hold run that is still queued or running but whose latest attempt
started before the triggering run finished may have read the gate state just
before that run ended. The re-check then reads the run listing again, up to
eight times 15 seconds apart (two minutes for all listed pull requests
together, far below its 5-minute timeout), and applies the rules above to the
newest merge-hold run in the fresh listing. The pull request head is read again
just before the re-run request; a moved head ends the re-check for that pull
request.

The re-check never sets a pull request check: the re-run merge-hold decides
again on the same head and push-time merge commit, with live run, job, label
and comment data, and fails closed as before, so it can only repeat a decision,
never pass one. Every other case is a no-op. An unexpected API answer fails the
re-check run, which changes no pull request check; the re-check run itself
appears on the default-branch head commit, so a failed one shows there. Its job has `actions: write` (needed for the
re-run), `contents: read` and `pull-requests: read`, a 5-minute timeout and one
run per head at a time. On `workflow_run` GitHub always runs the default
branch's copy of the workflow, and its checkout is the default branch, never
pull request code. Re-check jobs for `push`, `merge_group`, `schedule`,
`workflow_dispatch` and fork runs are skipped by the job condition and take no
runner.

Limits. The re-check takes effect only once its workflow is on the default
branch. If merge-hold is still open two minutes after the last watched run
finished and its report predates that run, the re-check leaves it alone, and the
check stays at "Pending". The same holds after a failed or timed-out re-check
run, and for a pull request that renames a watched workflow, whose renamed runs
start no re-check until the rename merges. The fallback is
unchanged: re-run the failed merge-hold job once the gates have finished
(`gh run rerun <run id> --failed`, or any label change). An expected gate
workflow that GitHub never starts keeps merge-hold at "Pending"; re-runs
cannot resolve that, so push a new head or investigate the missing run. Only a
push between the second head read and the re-run request can still make the
re-check re-run the old head's merge-hold, which cancels the new head's first
merge-hold; the next gate completion re-runs it. Like
the earlier wait, the re-check lets merge-hold turn green shortly after the
last gate without a manual re-run, so auto-merge can follow. It does not remove
the window in which another merge to `main` makes the pull request `behind`
under the strict up-to-date rule. Only a merge queue does, and enabling one is
a project-owner decision. Expected cost per head, not yet measured on hosted
runs: a merge-hold run of about one minute, one short re-check job per finished
watched workflow run, a wait of up to two minutes only when merge-hold is still
open as the last watched run finishes, and usually one merge-hold re-run. While
merge-hold is red, each watched run that ends after the last gate re-runs it
once more; the Materiaal probe also runs, and so starts a re-check, each time
a label is added.

API use: a full decision costs about one pull request read, one or two
run-list pages, and one job list and one re-read per counted workflow run
(about 30 today). A re-check costs one pull request read and one or two
run-list pages, one or two more pages per wait poll (at most eight), and one
more pull request read plus one re-run request when it re-runs merge-hold. A successful run without an expected
gate is listed too, because a job with job-level `continue-on-error` can fail
inside a run that concludes success. This stays well inside the 1,000 requests
per hour per repository of `GITHUB_TOKEN` for several open pull requests at
once.

**merge_group.** The merge queue is not active on `main`; enabling it needs the
project owner's explicit confirmation at that moment (D1). Every workflow with
an automatic `core` or `rwa` job has an unfiltered `merge_group` trigger (D1
step 1), so the selected gates also report on a queue commit, at full scope.
merge-hold on `merge_group` still waits only for `host-quality` and
`qemu-smoke` and does not judge those gates there; that is D1 step 2. So each
queue member needs, besides its hold decision, a successful newest
`pull_request` merge-hold run on its own head and a queue entry whose tree
equals that head tree. This makes a queue with more than one pull request
ahead fail closed. merge-hold reads every page of a queue commit's check runs.

**Bootstrap.** The pull request that introduces this enforcement is judged by
the base copy that predates it, so its own merge-hold checks only the
acceptance hold. Its applicable gates must therefore be verified by hand on the
final head before the merge, with the run links posted on the pull request. Every later pull request
runs the new base copy; one still carrying the old `merge-hold.yml` fails
closed (missing `BASE_SHA`) until it is updated, which the strict up-to-date
rule requires anyway.

**Not applicable is not a pass.** The summary lists filtered gates separately
("Not a pass"). A plan or a merge-hold success says that the required runs
succeeded; native acceptance evidence stays with those runs.

**Emergency.** `policy.mode = "shadow"` in the base catalogue turns gate
failures into warnings, while the acceptance hold stays enforced. A pull
request that sets it is itself judged by the enforcing base copy, so if the
enforcement machinery itself is broken (for example after an incompatible API
change), only the project owner's ruleset bypass can land the repair. The CI
owner never uses that bypass.

## Selection (phase 4)

**Scope.** Three workflows narrow, on `pull_request` only:
`component-leaves.yml` per variant (16 registered leaves),
`app-management-quality.yml` and `net-tls-quality.yml` per job. The integrated
native suites, CodeQL, `ci.yml` and every other workflow are not narrowed by
job: their input relations are not provable more narrowly (the `native` input
set covers almost all code); only their path filters decide whether they start
(see "Tooling metadata" below). `push`, `merge_group`, `schedule` and
`workflow_dispatch` always run everything.

**Tooling metadata (phase 5 C2).** On `pull_request`, 13 native workflows with
a `push` backstop on `main` (`entropy-keys`, `nas-core`,
`native-display-input`, `native-foundation`, `native-hw-inventory`,
`native-power`, `native-rtc`, `native-smp`, `native-system`,
`native-window-session`, `net-broker-quality`, `process-management`,
`signed-updates`) ignore `"@nonnative"`: a change confined to documentation,
project metadata and `core_only` tooling metadata (`.agents/**`, `.claude/**`,
`plugins/**`, `LICENSE` and the rest of the list) starts none of them, and
merge-hold does not wait for them. Without a usable diff or classification,
merge-hold instead keeps those gates Pending or fails them as possibly hidden
inputs ("Expected set", K10). Their `push` trigger stays `"@metadata"`, so
the merged commit runs them on `main`; a failure there reopens the rule, and a
run cancelled by a newer push to `main` is no evidence either way. `validate`
keeps that backstop: it rejects `"@nonnative"` on a workflow whose push trigger
on `main` is missing or not `"@metadata"`, and a `core` or `rwa` job that runs
on `pull_request` but not on `push`, by its own condition or a `needs`
prerequisite.
`native-hid-input` and `native-xhci` (which execute `plugins/**` through
`tools/tokenguard.py`), the PR-only `resource-pressure`, `storage-resilience`
and `volume-acceptance`, and the advisory `windows-qemu-test-version` keep
their filters. A host-quality test follows an over-approximate execution
closure from every pull-request job of the native workflows and fails when a
reached file is, or names, a `core_only` path that the catalogue does not
register as that job's input. Activation check before the change: of the last
120 merged commits, 4 changed only such paths; on their pull request heads 12
of the 13 workflows ran (`native-hw-inventory` did not exist yet), with 47
successes, 1 run cancelled by hand after the merge and no failure. **Rollback:**
set the 13 pull_request filters back to `"@metadata"` in the catalogue and the
YAML; `policy.mode = "full"` does not restart a workflow that GitHub filtered.

**Mechanism.** Each of the three workflows has a `select` job (class `core`,
check name `<workflow> select`, no `needs`, no `if`, `contents: read`). It
runs `./.github/actions/select-ci` (the trusted base selector, now with
`base-ref`) and then `tools/ci_impact.py jobs` from the checkout, which writes
`run-<job>` and, for a variant job, `matrix-<job>`. A gated job has
`needs: select` and `if: ${{ needs.select.outputs.run-<job> == 'true' }}`;
`component-quality` takes `strategy.matrix` from
`${{ fromJSON(needs.select.outputs.matrix-component-quality) }}`.

`jobs` narrows only when select-ci's `mode` output is `select` (forced to
`full` when the base selector did not run) and the plan is a trusted
(`selector.trusted`) `pull_request` plan in `mode = "select"` under
`policy_mode = "select"`, for this run's `github.sha` and target branch, computed
from the base catalogue (`catalogue.source_sha` equals `shas.base`). Otherwise,
including a missing or malformed plan file, every job and variant runs. A job or
variant the base plan does not know (added by this pull request) always runs.
When nothing of a job is selected, `run-<job>` is `false` and the job is
skipped; its matrix stays the full list, so GitHub never sees an empty matrix.

**Matrix source.** For `component-quality` the catalogue is the only source of
the matrix: each variant's `matrix` table (component, manifest, toolchain,
features). Add or change a leaf in `.github/ci-impact.toml`, not in the
workflow. `coverage` checks the wiring (`matrix-mismatch`, `select-wiring`,
rendered check names, evidence names) instead of comparing YAML combinations.

**Verification.** merge-hold (phase 3) recomputes its own plan. Every gate it
expects must have succeeded, so a `select` job that chose too little, failed,
or was edited by the pull request blocks the merge; a skipped job is fine only
when merge-hold's plan does not expect it. A `select` failure skips the gated
jobs, and merge-hold then blocks on them (fail closed). Running more than the
plan needs is harmless. The `jobs` rendering runs the checkout's own copy of
`tools/ci_impact.py`, so it is untrusted convenience, not a control; only
merge-hold decides. `validate` accepts a select output only in the exact form
`${{ needs.select.outputs.run-<own job id> == 'true' }}` with `needs: select`.

**Limit (push and dispatch).** Before phase 4 the leaves ran unconditionally on
`push` to `main`. They now depend on the `select` job. select-ci turns selector
uncertainty into a full plan rather than a failure, so only an infrastructure
failure (checkout, Python setup) can fail `select`. A red `<workflow> select`
on `main` therefore means that workflow's gated jobs did **not** run; re-run the
workflow. The repository rule that only artifact uploads may tolerate errors
(`continue-on-error`) is kept, so this is documented rather than masked.

**Rollback to full.** Set `policy.mode = "full"` (or `enforce`) in
`.github/ci-impact.toml` on `main`. The `select` jobs then output `true` for
every job and the full matrix, so all three workflows run everything again,
while merge-hold keeps verifying every applicable gate. host-quality,
qemu-smoke, CodeQL and the acceptance hold do not depend on the mode. Removing
the `select` jobs is not needed for a rollback.

## Known limits

Status after phase 3; phase 4 adds no new limit beyond "Selection". Limits marked closed have tests in
`tools/tests/test_ci_impact.py`, `tools/tests/test_merge_hold.py` or
`tests/test_ci_workflows.py`.

- **K1, plan production on the pull request side: closed for the decision.**
  The shadow job, `select-ci` and its artifact still come from the pull
  request side, but merge-hold recomputes the plan with the base copies of
  `merge_hold.py` and `ci_impact.py` and trusts nothing a pull-request-side
  job publishes. **Residual:** under `pull_request` GitHub reads the workflow
  file itself, including `merge-hold.yml`, from the merge commit, so a pull
  request that edits `merge-hold.yml` or a gate's workflow can still change
  what runs. Review of `.github/**` (Protected files) is the control; a
  ruleset that requires an independent review of `.github/**` changes, or a
  base-side `pull_request_target` check, is an owner decision outside #494.
- **K2, command-line generations: managed.** merge-hold calls a selector of
  its own generation. `select-ci` (head) calls the base selector, so a new
  `plan` flag reaches `select-ci` only after the selector that knows it is on
  `main`; `--base-ref` was added in phase 3 and is passed by `select-ci`
  (only when set) from phase 4.
- **K3, shadow plans are not evidence: unchanged by design.**
- **K4, branch-blind applicability: closed.** `--base-ref`, see "Plan
  algorithm and precedence".
- **K5, runtime escapes: narrowed.** `coverage` reports
  `env!("CARGO_MANIFEST_DIR")` joins with a `..` literal and `include*!` of an
  environment variable; `services/nas/native-volume/tests/volume.rs:207` and
  `tools/update-tool/tests/roundtrip.rs:305` are registered. Paths built in
  other ways (variables, several statements, Python harnesses reading files)
  remain a limit; integrated native suites keep the broad `native` input set.
  A tracked `.rs` file reached through a `CARGO_MANIFEST_DIR` join is scanned
  as source even when a test only reads it as data; that can over-report an
  escape (a false block in host-quality, never a narrower plan).
- **K6, root-scoped library escapes: closed.** `escape-scope-mismatch`; the
  zune-jpeg test-image includes are now `always`.
- **K7, job conditions: closed.** See "Job conditions".
- **K8, protected files: closed.** `tools/ci_impact.py` and
  `tools/merge_hold.py` are on the Protected files list in `AGENTS.md`;
  `.github/ci-impact.toml` already was (`.github/**`).
- **K9, activation OR-guard heuristic: unchanged, report only.** Activation
  correctness is not inferred from it; merge-hold judges actual results.
- **K10, `@nonnative` gates that never start: open, fail-closed.** On a
  tooling-metadata-only pull request (phase 5 C2) GitHub starts none of the 13
  `@nonnative` workflows, but the plan can still expect them: after a pre-diff
  fallback that merge-hold accepts (`ambiguous-merge-base`, `shallow-history`,
  `missing-history`, `diff-failed`, `unknown-diff-status`, `diff-parse`), after
  `diff-too-large` (more than 5,000 files), or under a rollback or full
  `pull_request` mode with more than 300 changed files. merge-hold then reports
  them Pending until a re-run with a usable diff or a new head. A fallback before classification (for example
  `cargo-manifest-missing`, or a Cargo construct on the base) instead fails
  those rows as possibly hidden inputs, with a message that names the
  `core_only` paths. The causes are broken history, an oversized diff, a
  transient read failure, a broken base catalogue, an active rollback or an
  unmodelled Cargo construct on the base; only the last persists, because once
  merged onto `main` it fails the 13 rows on every later tooling-only pull
  request until the construct is modelled.

## Phases

- **2a (PR #499, merged):** catalogue, selector, tests and this contract. No
  CI wiring.
- **2b (PR #501, merged):** the `ci-impact-shadow` job and the `select-ci`
  action run the trusted base copy of the selector on every event and preserve
  the plan; a report-only step preserves the catalogue coverage report.
- **3 (enforce, merged):** merge-hold recomputes the plan from base-side code
  and blocks until every applicable gate passed on the pull request head;
  host-quality asserts catalogue consistency; limits K4, K6, K7 and K8 closed.
- **5 (fail-closed native selection):** C1, a path filter may hide only
  metadata (see "Expected set"); C2, `"@nonnative"` on the pull_request
  trigger of 13 backstopped native workflows (see "Selection"); C3, the
  advisory candidate shadow plan (see "Shadow-mode wiring").
- **4 (current, select):** component leaves per variant, app-management and
  net-tls per job; see "Selection".
- **Rollback:** setting `policy.mode = "full"` in the catalogue on the base
  branch makes every plan full immediately: every workflow that its own
  path filter starts runs everything while merge-hold keeps verifying every applicable gate, and host-quality,
  qemu-smoke, CodeQL and the acceptance hold are unchanged. `shadow` (above)
  is the separate emergency switch for the enforcement itself.
