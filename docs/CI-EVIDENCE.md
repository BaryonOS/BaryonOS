# CI test evidence independent of artefact quotas

**Decision:** ADR-CI-EVIDENCE-20261108. The test outcome, compact run evidence
and availability of a download are separate results. A failed GitHub upload
does not make a passing test fail. A failed test, build, validator or evidence
collector remains an error.

## Normal runs

The workflows that preserve evidence use
`.github/actions/preserve-evidence/action.yml`. The action reads only the
explicitly selected existing evidence and performs the following steps:

1. Before uploading, record the actual checkout SHA, GitHub SHA, run ID,
   attempt, job status and inventory. `tools/ci_evidence.py` writes these to
   the run log, job summary and `_meta/receipt.json` in a temporary directory.
2. Preserve compact UTF-8 reports and text logs with their original repository
   paths and SHA-256. JSON results are not rewritten. Binary builds, disk
   images, raw frames and ZIP files fall outside this selection. Missing,
   oversized and excluded files remain visible in the receipt.
3. Attempt to upload the compact package as `<old-artifact-name>-reports`:
   three-day retention, compression level 9. The compact upload, full upload
   and separate Windows package upload all have `continue-on-error: true`
   and are therefore best effort.
4. Show the actual upload outcome through `steps.<id>.outcome`, with a download
   URL only on success. When a quota error occurs, the run evidence already
   written remains available.

The collector limits the preserved collection to 8 MiB and 500 files,
including its own receipt; an individual report defaults to at most 1 MiB.
The Rust Cobertura report in `ci.yml` explicitly uses `max-file-bytes: '4194304'`
(4 MiB) and is a required exact path. Run `43594581854` generated a
2,349,735-byte report that exceeded the default. This allowance preserves the
complete XML; it does not truncate the report or change coverage measurements.
The Python Cobertura caller also requires its exact path and explicitly permits
2 MiB. Run `45311246616` produced a 1,057,659-byte report; its tests passed, but
the previous 1 MiB allowance correctly failed to preserve the required evidence.
The caller repair retains the complete original XML and does not change coverage
measurements or missing-evidence failure.

The native window workflow preserves each phase's original `result.json` and
`serial.txt` in a separate compact proof collection. Each file is limited to
4 MiB, and the pair must fit within the unchanged 7 MiB payload budget plus
1 MiB receipt reserve. The workflow runs five independent native legs, each
on its own runner with its own probe build: `native-scale1`, `native-scale2`,
`library` (library write, then cold recovery of that write's parent image in the
same leg), `switcher-scale1` and `switcher-scale2`. Each leg preserves its own
proof pair; the `library` leg preserves the write and cold pairs separately.
Required proof cannot be silently omitted for a missing file, individual-file
limit or aggregate-budget limit. Each leg's final collection,
`native-window-session-<leg>-evidence`, holds that leg's logs and manifests and
offers its optional full archive. Host checks, the read-only compile and the
normal build with the Windows package keep their own collections. Phase
collections add no duplicate raw archive or package upload.

This separation addresses run `45311246725`: its successful scale-2 native job
produced a 1,553,981-byte result and 1,556,536-byte serial log that exceeded the
old per-file default. Runtime success remained distinct from the unavailable
complete report. Copies of one report and other phases must not compete with a
required original result/serial pair for the same compact budget. An uploaded
receipt or hash alone does not supply omitted report bytes for revalidation.

Other callers keep the 1 MiB default. The collector accepts per-call limits
up to 7 MiB, always reserving a fixed 1 MiB for the receipt within the unchanged
8 MiB total budget. Receipt metadata records the actual selected file limit.
The console contains small JSON reports and labelled log excerpts, with a
combined output budget of approximately 256 KiB. Truncated excerpts are
explicitly marked. Hidden files, symlinks and paths outside the checkout are
not followed. This is not a content-aware secrets scanner: the calling
workflow must select only non-sensitive evidence files intended for CI.

GitHub does not count run logs and job summaries towards the artefact storage
quota. Their normal retention still applies; this policy does not shorten it.

## Full archive for revalidation

Open the relevant workflow under **Actions → Run workflow** and enable
`full_evidence`. Tests run again on the selected branch. The action then also
attempts to preserve the original full path selection under the original
artefact name, with **one-day** retention. The compact record is retained
for this run as well.

Use this route when raw disk images, pixels or build bytes are needed for
independent revalidation. In `native-window-session.yml`, `full_evidence`
still enables the full archive, but each native leg now uploads its own archive
as `native-window-session-<leg>-evidence`, containing only that leg's build
directory `build/window-session-ci/<leg>/`. Download the archive of every leg
the revalidation requires, all from the same run, so that every archive is
bound to the same source commit. The library write and cold runs are both in
the `library` leg; the switcher profiles are in `switcher-scale1` and
`switcher-scale2`.
`docs/verification/materiaal-library-20261102/verify-ci-artifact.py` is a
historical example: it expects the earlier fixed `build/library-ci-640/data`
layout, its source fingerprint and the former shared `probe/` directory, and
does not apply to the per-leg layout unchanged.
Download the required archive before it expires. Compact reports and hashes
do not replace raw bytes. A failed full upload means that archive is
unavailable; a green test job does not demonstrate otherwise. Acceptance
requiring raw bytes remains open without that evidence.

## Downloadable Windows test packages

After successful build and test steps, `windows-qemu-test-version.yml` and
`native-window-session.yml` continue to offer their ZIP separately as
`<old-artifact-name>-bundle`, with one-day retention and no extra compression
of the already compressed ZIP. During a full manual evidence run, the package
is included in the full archive and is not uploaded twice.

The job summary states separately whether a **verified download** is available.
If an upload fails, an earlier test fails or a package step is skipped, no
Windows test version has been delivered. Existing native tests and package
validation remain mandatory. This does not demonstrate execution of the ZIP
on Windows.

## Quotas and existing artefacts

The repair deletes no old evidence archives and changes no billing or
repository permissions. Current account-wide storage could not be fully
inventoried in this session. The checked recent run-specific artefact lists
were empty; this does not establish total storage usage.

Shorter retention applies to new uploads. Old artefacts keep their existing
expiry dates. For targeted cleanup: first inventory names, runs, dates and
sizes; preserve required acceptance evidence; delete only the selected
outdated artefacts. Do not delete complete runs to free storage, because that
also removes the run-log fallback. GitHub may need 6–12 hours to recalculate
storage usage. The workflow no longer needs to wait for that calculation to
establish its test outcome.

## Workflow syntax validation

The required `host-quality` job runs actionlint v1.7.12 immediately after
checkout. It checks GitHub Actions YAML, expressions, and workflow semantics.
The invocation uses `-shellcheck= -pyflakes=` because shell and Python quality
remain owned by the existing required checks; this step does not claim those
optional integrations.

The job downloads only `actionlint_1.7.12_linux_amd64.tar.gz`, verifies SHA-256
`8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8`, and
extracts only the `actionlint` executable into the runner's temporary directory.
The upstream `v1.7.12` tag points directly to commit
`914e7df21a07ef503a81201c76d2b11c789d3fca`; the release was published on
March 30, 2026, under the MIT license. The tag commit is unsigned. The verified
release checksum therefore detects asset substitution relative to the upstream
checksum list, but it is not a signed provenance chain or an audit of the
actionlint source and build process.

## Coverage tool binary

The required `host-quality` job no longer compiles cargo-llvm-cov from source
on every run (#494 B3). It downloads only
`cargo-llvm-cov-x86_64-unknown-linux-gnu.tar.gz` from the upstream `v0.9.1`
release, verifies SHA-256
`b3f68e625481fed9b16444174f3fa5ebcdbde4a1878803a35eabe2dcefcdc41a`, extracts
only the `cargo-llvm-cov` executable and checks the reported version. The
`v0.9.1` tag points directly to commit
`0dc905679ae6b4740b186f6e2c2ef1ab78fb76d7`; the release was published on
September 6, 2026, under Apache-2.0 or MIT. The pinned value equals the
digest that GitHub records for the release asset, and GitHub lists two
artifact attestations for that digest (read through the REST API on
November 25, 2026; not verified in CI). The checksum detects asset
substitution; it is not an audit of the cargo-llvm-cov source or build.

The host build cache also holds `target/llvm-cov-target`. By default,
cargo-llvm-cov cleans the artifacts that could affect coverage results before
each run, and only a push to `main` saves the cache. A restored entry is
therefore expected to shorten the build without changing a test verdict or a
coverage report. This expectation rests on the documented cargo-llvm-cov clean
step; the build-time gain has not yet been measured on a warm CI run.

## Validation and rollback

`python3 -B -m unittest discover -s tests -p 'test_ci_evidence*.py'`
checks only the evidence helper on the host. `host-quality` now also runs this
check. Existing native test commands and gates are unchanged. Locally
simulated upload outcomes are not actual GitHub uploads or new QEMU acceptance.

Rollback: revert the CI evidence commit through a PR. This restores the old
upload steps and their blocking behaviour. Do not remove source, progress
or evidence changes from other commits.

## Primary sources

GitHub. (n.d.). *GitHub Actions billing*. Retrieved November 8, 2026,
from https://docs.github.com/en/billing/concepts/product-billing/github-actions

GitHub. (n.d.). *Contexts reference*. Retrieved November 8, 2026,
from https://docs.github.com/en/actions/reference/workflows-and-actions/contexts

GitHub. (n.d.). *Metadata syntax reference*. Retrieved November 8, 2026,
from https://docs.github.com/en/actions/reference/workflows-and-actions/metadata-syntax

GitHub. (n.d.). *Workflow commands for GitHub Actions*. Retrieved
November 8, 2026, from
https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-commands

GitHub Actions. (n.d.). *upload-artifact* [Source code and documentation].
Retrieved November 8, 2026, from https://github.com/actions/upload-artifact

rhysd. (2026). *actionlint v1.7.12 release*.
https://github.com/rhysd/actionlint/releases/tag/v1.7.12

rhysd. (2026). *actionlint v1.7.12 checksums*.
https://github.com/rhysd/actionlint/releases/download/v1.7.12/actionlint_1.7.12_checksums.txt

rhysd. (2026). *actionlint source at v1.7.12*.
https://github.com/rhysd/actionlint/tree/914e7df21a07ef503a81201c76d2b11c789d3fca

taiki-e. (2026). *cargo-llvm-cov v0.9.1 release*.
https://github.com/taiki-e/cargo-llvm-cov/releases/tag/v0.9.1

taiki-e. (2026). *cargo-llvm-cov source at v0.9.1*.
https://github.com/taiki-e/cargo-llvm-cov/tree/0dc905679ae6b4740b186f6e2c2ef1ab78fb76d7
