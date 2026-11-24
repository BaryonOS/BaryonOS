# CI contract: impact catalogue and selector

Status: phase 2a of issue #494. **Phase 2a does not change CI execution.** No
workflow, action or required check reads the catalogue or runs the selector
yet; every existing trigger, path filter and required gate behaves exactly as
before. This document describes the data model and the selector so that later
phases can wire them in without redesign.

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

`experimental-m3-m4-smp` in `native-smp.yml` is automatic and blocking despite
its name, so it is registered as `rwa`. `windows-qemu-test-version.yml` is
registered as `advisory`; the project owner may promote it.

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
  selector also hard-codes `merge_group` as full. `workflow_dispatch` must be
  `full` or `manual`. `pull_request` may be `select`, `full` or `manual`.

`[input_sets]`: named glob lists. `native` covers the integrated QEMU suites.

`[[crate_escape]]`: `crate` (directory), `when` (`always`: whenever the crate is
in a closure; `root`: only when it is the gate's own root, for tests and
binaries), `inputs` (globs) and `evidence` (`file:line`).

`[[workflow]]`: `path`, `owner` (default `ci-owner`), `granularity`
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
`dynamic = true`.

Workflow paths are `.github/workflows/<id>.yml` (or `.yaml`) with an id of
`[A-Za-z0-9_][A-Za-z0-9_-]*`; job ids match `[A-Za-z_][A-Za-z0-9_-]*` and
variant ids `[A-Za-z0-9_.*-]+`. The step summary escapes every table cell
(`&`, `|`, `<`, `>`, `[`, `]` and backticks become HTML entities), so even a
hostile head catalogue listed in a bootstrap plan cannot inject Markdown or
HTML.

Execution ids are `<workflow stem>#<job id>[/<variant id>]`, for example
`component-leaves#component-quality/image-viewer`. Execution ids and rendered
check names are unique across the catalogue.

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
full plan (`cargo-workspace-inheritance`, `cargo-patch-unresolved`).

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
   `missing-history`. For a pull request, a tested merge commit must have the
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
   `filter_eval_uncertain` is set for zero or more than 300 changed files.

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
  `blocked_by_filter`;
- `excluded`: `id`, `class`, `reason` (class name, `not-automatic`,
  `event-not-applicable`, `no-input-match` or `manual-event`);
- `plan_id`: `sha256` over the schema version, catalogue blob and revision,
  selector blob, event, mode, fallback code, SHAs, changes digest and selected
  ids. The same inputs always give the same id.

In full mode `selected` is informational; `mode = "full"` itself means every
automatic core and rwa gate runs.

## Commands

```sh
python3 -B tools/ci_impact.py validate .github/ci-impact.toml   # exit 0 valid, 1 invalid
python3 -B tools/ci_impact.py plan --event pull_request --base-sha <base> \
  --head-sha <head> [--tested-sha <merge>] [--out plan.json] [--summary file] [--no-fetch]
python3 -B tools/ci_impact.py coverage   # JSON drift report, always exit 0
```

`coverage` needs PyYAML and otherwise reports `coverage-unavailable`. It
compares the catalogue with `.github/workflows/*.y*ml` (workflows, events,
trigger filters, jobs, names, conditions, needs, timeouts, max-parallel,
rendered matrix variants and check names, evidence names). It also checks:

- that each job's `activation` agrees with the workflow's `on:` keys; a
  `schedule` or `dispatch` job in a workflow that also has `pull_request`,
  `push` or `merge_group` needs a condition such as
  `github.event_name == 'schedule'` (`activation-mismatch`);
- obligation citations: the anchor must occur on the cited line or range;
  within five lines it reports `citation-moved` with the new line, otherwise
  `citation-anchor-missing` or `citation-missing`;
- that Cargo roots exist, and scans closure crates for unregistered source
  escapes;
- every `git ls-files` path is classified.

It is a report, not an enforcement; enforcement in host-quality is a later
phase.

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

## Phases

- **2a (this change):** catalogue, selector, tests and this contract. No CI
  wiring.
- **2b:** a `ci-impact-shadow` job and a composite action run the trusted base
  copy of the selector on every pull request and preserve the plan. Shadow
  plans are recorded, never enforced.
- **3 (enforce):** per-workflow `select` and `final` jobs and merge-hold verify
  that every selected gate passed, bound to the `plan_id`.
- **4 (select):** workflows run only the selected executions.
- **Rollback:** setting `policy.mode = "full"` in the catalogue on the base
  branch makes every plan full immediately.
