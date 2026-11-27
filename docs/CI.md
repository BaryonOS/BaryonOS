<a id="ci-voor-de-publieke-repository"></a>

# CI for the public repository

**Configuration:** all 23 workflow files select
`ubuntu-24.04` by default: 26 job definitions and up to 42 jobs after matrix
expansion, including the separate Copilot setup and the historically private
audit. That audit retains its private-only condition and does not run on the
public repository. No self-hosted runner selectors remain.

**Integration:** the existing PR #249 combined
the runner/tool pins with the evidence solution from PR #251 and the
streamlining described below. The repository was verified as `public` through
GitHub on 8 November 2026. Configuration on a working branch becomes the
default for new main runs only after merging.

**Evidence (8 November 2026 checkpoint):** the local host and configuration
checks are recorded in
[the integration report](verification/ci-public-runners-20261108.json).
On the published source `338bec8c`, 39 of the 40 started Actions jobs
subsequently passed, including both required checks, the complete SMP suite
and the Copilot setup. The window-session test was still running at that
observation; no job had failed. The actual runner labels, results, timings
and successful compact QEMU upload are recorded in the
[GitHub checkpoint](verification/ci-public-runners-20261108/hosted-checkpoint.json).
This checkpoint is not a final result for the test that was still running
or for later commits. The previously measured budget block on the repository
while it was still private is historical evidence; it is not used as a current
execution result.

<a id="wanneer-wat-draait"></a>

## What runs when

| Trigger | Execution |
|---|---|
| PR opened, changed or reopened, including drafts | Required `host-quality` and `qemu-smoke`; applicable specialist workflows also run |
| Draft PR marked ready for review | No rerun of the same source |
| Push to main | Required checks on the integrated source; specialist workflows skip only documentation and project metadata, so tooling-metadata changes skipped on the PR run here |
| Only `docs/**` or root Markdown changed | Required CI remains available; broad specialist workflows do not rerun. On PRs, tooling metadata (the catalogue's `core_only` paths, such as `.agents/**`, `plugins/**` or `LICENSE`) also skips 13 native workflows with a main backstop (`@nonnative`, docs/CI-CONTRACT.md); workflows without `@nonnative` keep their own filters, so `native-hid-input` and `native-xhci` still run (docs/CI-CONTRACT.md) |
| Manual run through Actions | Each regular workflow can be started separately; evidence workflows provide `full_evidence` |
| Copilot setup changed | Targeted setup check; after merging, the configuration becomes the Copilot development environment |
| A gate workflow is re-run, or a listed catalogue workflow finishes, for a PR from this repository | At the start and completion of a watched gate re-run, `merge-hold-recheck.yml` invalidates older passing (`success`, `neutral` or `skipped`) evidence, even while another gate is open. An older active hold is cancelled, followed by exact run identity and force-cancelled if necessary; a passing completion race is re-run. A confirmed blocking hold is retried only after no gate run remains open. Initial starts are skipped and `main` activities are filtered. A `merge-hold` failure with only "Pending:" lines is expected and needs no code fix; act only on "Failed:" lines. If it is still failed once every gate has finished, re-run it by hand (`gh run rerun <run id> --failed`) ([Waiting and Re-check](CI-CONTRACT.md#enforcement-phase-3)) |

`ci.yml` deliberately has no path filters. The existing main and release
rules require `host-quality` and `qemu-smoke`; skipping an entire workflow
could otherwise leave a required check Pending. Both jobs have their own
checkout and build and now run in parallel. Both must pass.

Superseded PR runs are cancelled through the existing concurrency groups.
The SMP workflow also protects the running main run; this does not guarantee
that every queued intermediate commit is executed separately.

The component matrix was reduced from 16 to **14 jobs**, with at most four
running concurrently so that the matrix does not occupy all capacity needed
by the other gates. `ui/native` is already checked in full in
`native-display-input.yml`. `artifact-store-core` has its own workflow, now
also with the scalar freestanding build flags from the former matrix entry.
No component test or native test case was removed for this change.

<a id="vaste-gereedschappen"></a>

## Pinned tools

| Component | Version/selection | Managed in |
|---|---|---|
| Runner | Default `ubuntu-24.04`, Linux x64 | `.github/workflows/` |
| Python | 3.12.12 | `.github/actions/setup-python/action.yml` |
| PyYAML for host tests | 6.0.3, hashed CPython 3.12/Linux x64 wheel | `setup-python/requirements.txt` |
| Rust | 1.90.0; existing 1.91.1 checks retained | Toolchain files and workflow steps |
| Node | 24.22.1 for prototype/NAS tests, the net-tls H2 OpenSSL peer and Copilot | CI and Copilot setup |
| QEMU x86/common/data | `1:8.2.2+ds-0ubuntu1.18` | `.github/actions/setup-qemu/action.yml` |
| OVMF | `2024.02-2ubuntu0.10` | Same QEMU setup |
| SeaBIOS | `1.16.3-2` | Same QEMU setup |
| iPXE-QEMU | `1.21.1+git-20220113.fbbdc3926-0ubuntu2` | Same QEMU setup |
| swtpm / libtpms0 | `0.7.3-0ubuntu5.24.04.1` / `0.9.3-0ubuntu4.24.04.1` | Native-system and Copilot |

The 17 regular QEMU job definitions plus Copilot reuse one installation from
a pinned package bundle (release `ci-qemu-apt-20261111-r2`, ADR-CI-QEMU-BUNDLE-20261111).
The bundle contains the Ubuntu-signed indexes and packages; APT checks the
signature and hashes as it would for a normal source. If the bundle becomes
unavailable, setup falls back with a warning to the signed Ubuntu snapshot
**20261108T000000Z** and then the signed archive, with the same pins. Setup
checks exact package versions, `pc-q35-8.2`, TCG and the SHA256 of OVMF
CODE/VARS and the VGA ROM. There is no fallback to other tool versions.
Firmware paths:

```text
OS_OVMF_CODE=/usr/share/OVMF/OVMF_CODE_4M.fd
OS_OVMF_VARS=/usr/share/OVMF/OVMF_VARS_4M.fd
```

TCG and disposable images remain the regular test baseline; KVM or a
dedicated server is not a prerequisite. The SMP profiles retain their own
CPU counts and substantive checks. The existing guest and job time limits
are retained; any capacity problems must first be measured on the current run.

`ubuntu-24.04` fixes the distribution, not every byte of GitHub's VM image.
Setup therefore records `ImageOS`, `ImageVersion`, packages and firmware
hashes. Existing host libraries may change with the runner image.

<a id="cache-zonder-overgeslagen-controles"></a>

## Caching without skipped checks

The existing download caches are retained. The host Rust build cache now
has a key containing both a compatibility hash and a source SHA;
`restore-keys` reuses the latest matching cache. This allows a later source
version to save its updated intermediate artifacts. GitHub caches cannot be
overwritten under the same key.

All test and build steps continue to run after cache restoration. Native
disk images, acceptance reports and secrets are excluded from this cache.
The change is an execution optimization, not a measured speed claim.

<a id="host--en-qemu-controles"></a>

## Host and QEMU checks

`host-quality` retains Rust tests, Clippy/fmt, SDK and target builds,
Python/JavaScript tests, quantum experiments and the progress/clean-tree
check. The additional existing runner, preflight, harness, artifact-binding
and M6c host tests from PR #249 remain included. New configuration regression
tests guard runner selection, restricted permissions, pinned Action SHAs,
required gates and preservation of the combined component checks.

The main QEMU gate retains six starts:

| Check | Starts on complete success |
|---|---:|
| M6c normal storage | 1 |
| DocumentSession write and cold recovery | 2 |
| Native quantum job interface | 1 |
| SDK app with permission | 1 |
| SDK app with denied permission | 1 |

Host tests or a QEMU tool probe do not replace these native gates.
A failing test remains a failure; only best-effort artifact uploads tolerate
upload errors.

<a id="bewijs-en-testpakketten"></a>

## Evidence and test packages

All sixteen evidence workflows, including `ci.yml`, use
`preserve-evidence`: first source-bound reports in the run log/summary, then
a bounded compact artifact with three days of retention. Full selections
can be requested manually with one day of retention. Windows test ZIPs
remain available separately when both the package check and upload pass.

The additional `qemu-smoke` selection contains only the known reports and
text logs; its manual full selection also excludes raw disk images and
binaries. Preflight writes to the job log, not to an assumed
`build/preflight.json`. See [CI-EVIDENCE.md](CI-EVIDENCE.md) for limits,
actual upload status and independent rechecking.

<a id="publieke-bijdragen-en-onderhoud"></a>

## Public contributions and maintenance

PR tests use `pull_request`, `contents: read` and checkout without persisted
credentials. There is no `pull_request_target` with executable PR code.
External Actions are pinned to full commit SHAs. The regression tests
prevent an old branch from silently reintroducing a self-hosted selector.

Copilot uses the same Python/QEMU pins, both Rust toolchains and Node.
Setup does not install a WSL runner or require KVM or firewall changes.
The required job ID is `copilot-setup-steps`. This setup is not a new
regular native acceptance gate.

Update pins deliberately with source/hash verification and actual host/native
gates. Historical package provenance is recorded in
[ci-packages-20261108.json](verification/ci-packages-20261108.json).
Rollback: a targeted revert through a PR, preserving other changes.
Product progress remains **41/100** in the recorded assessment; CI setup does
not close an OS criterion.

<a id="coderabbit-en-mergeblokkades"></a>

## CodeRabbit and merge blocks

The check of 8 November 2026 confirmed that the active main rule
`Bescherm main branch` listed only `host-quality` and `qemu-smoke` as required
status checks. At that checkpoint, review conversations also had to be
resolved and the branch had to be up to date. The required number of
approvals was zero. CodeRabbit can still block merging through a submitted
`CHANGES_REQUESTED` review or an open review conversation; a green
`CodeRabbit` check only means that the review task has finished.

`.coderabbit.yaml` keeps automatic incremental reviews active and selects
`quiet`: important comments remain inline; other suggestions appear in the
summary. `auto_approvals.enabled: false` prevents new automatic rejections
and approvals by the bot. Docstring coverage remains a warning. The
configuration inherits the other configured review tools and project
instructions. No source path is excluded and the GitHub branch rules are
unchanged.

Existing rejections do not disappear because of this YAML. Check each
comment against the current source, fix valid defects and only then resolve
the corresponding conversation. Dismiss an old bot rejection only with a
reasoned explanation once the associated findings have been addressed.
An existing open substantive finding remains open; a settings change is
not evidence that the code is correct.

CodeRabbit reads the configuration from the PR branch. After integration,
update older branches with the current main content while preserving their
own changes. This is also necessary for old workflows still waiting for
removed self-hosted runners: rerunning the same old workflow does not change
the runner selection. Merge conflicts require separate integration that
preserves content. The counts and source versions from this check are
recorded in
[the CodeRabbit report](verification/coderabbit-unblock-20261108.json).

<a id="primaire-bronnen"></a>

## Primary sources

- GitHub. (n.d.). *GitHub-hosted runners reference*. Retrieved 8 November
  2026, from https://docs.github.com/en/actions/reference/runners/github-hosted-runners
- GitHub. (n.d.). *Workflow syntax for GitHub Actions*. Retrieved
  8 November 2026, from https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax
- GitHub. (n.d.). *Dependency caching reference*. Retrieved 8 November
  2026, from https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching
- GitHub. (n.d.). *Configure the development environment for agents on GitHub
  Actions*. Retrieved 8 November 2026, from
  https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/customize-cloud-agent/customize-the-agent-environment
- Canonical. (n.d.). *Ubuntu Snapshot Service*. https://snapshot.ubuntu.com/
- CodeRabbit. (2026, 1 November). *Configuration reference*.
  https://docs.coderabbit.ai/reference/configuration
- CodeRabbit. (n.d.). *Automatic Approvals*. Retrieved 8 November 2026,
  from https://docs.coderabbit.ai/pr-reviews/automatic-approvals
- CodeRabbit. (n.d.). *Configuration inheritance*. Retrieved
  8 November 2026, from https://docs.coderabbit.ai/configuration/configuration-inheritance
- CodeRabbit. (n.d.). *Configuration via YAML file*. Retrieved
  8 November 2026, from https://docs.coderabbit.ai/getting-started/yaml-configuration
