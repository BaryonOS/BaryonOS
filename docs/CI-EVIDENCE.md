# CI test evidence independent of artefact quotas

**Decision:** ADR-CI-EVIDENCE-20261108. The test outcome, compact run evidence
and availability of a download are separate results. A failed GitHub upload
does not make a passing test fail. A failed test, build, validator or evidence
collector remains an error.

## Normal runs

The sixteen workflows that preserve evidence use
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
including its own receipt; an individual report is at most 1 MiB.
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
independent revalidation, as with
`docs/verification/materiaal-library-20261102/verify-ci-artifact.py`.
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
