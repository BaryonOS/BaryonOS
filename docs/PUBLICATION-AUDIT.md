# Local publication audit

The repository is public. Historical GitHub Actions logs and artifacts may
therefore contain bytes that were published outside the current Git tree. The
local publication audit inventories the Git refs and GitHub Actions resources
that remain enumerable, scans them in bounded batches, and records a
content-bound result. It is a post-publication remediation aid, not a release or
publication approval.

The audit does not add another GitHub Actions workflow. Public-repository
execution in GitHub Actions is rejected because the private state can contain
downloaded logs, artifact bytes, filenames, and scanner reports. The state must
remain on a trusted local filesystem and must never be committed or uploaded.

## State machine

The command in [`tools/publication_audit.py`](../tools/publication_audit.py) has
three phases.

1. `snapshot` binds the repository ID, canonical name and visibility; the
   configured origin after an API-verified rename redirect; the exact local
   HEAD; two equal inventories of every advertised branch, tag, pull-request
   head and pull-request merge ref; a GitHub server-time cutoff; two equal,
   complete Actions inventories at that cutoff; the scanner, configuration and
   auditor hashes; and all aggregate limits. Every advertised tip must already
   exist locally. The canonical JSON manifest hash is the audit identifier.
2. `scan-batch` selects at most the requested number of missing resources in a
   deterministic order. Each receipt binds the manifest, immutable resource
   identity, scanner, retained payload hash and retained report hash. The phase
   never reports audit success, including after the last batch.
3. `finalize` repeats the complete Actions inventory twice at the original
   cutoff, compares it with the manifest, rechecks HEAD and all advertised
   remote refs, and
   validates every retained payload, report and receipt. Only this phase can set
   `snapshot_audit_passed` to `true`.

The single-writer state lock and monotonic usage ledger cover all phases. API
and storage requests, elapsed time, downloaded bytes, expanded bytes, archive
entries and retained state have explicit aggregate bounds. Git and scanner
subprocesses use the same command deadline. A killed process leaves an active
time reservation; the next invocation charges the intervening interval rather
than resetting it.

## Fail-closed conditions

Finalization remains open when any of the following is observed:

- an active run or expired artifact existed at the cutoff;
- a resource returns HTTP 404 or 410, is missing, or changes identity;
- an inventory, HEAD or ref differs from the snapshot;
- a receipt, payload, scanner report or tool binding is absent or modified;
- a finding exists;
- historical archive paths, Git LFS pointers, submodule repositories, nested
  archives, encrypted members, non-UTF-8 historical paths, or
  non-UTF-8/NUL-containing blob, commit, tag or Actions-member classes require
  separate content coverage;
- an API, rate-limit, time, download, expansion, entry or state bound is reached.

Expired artifacts are retained in the manifest as blockers. They are not
silently excluded. Resources deleted before the snapshot cannot be enumerated,
and deleted prior publication history cannot be reconstructed by this tool.
Consequently, `publication_clearance` is always `false`, even when the bounded
snapshot passes.

## Local operation

Use a non-shallow checkout containing the object graph for every branch, tag,
pull-request head and pull-request merge ref advertised by GitHub. A main-only
fetch is insufficient. Fetching refs is a separate, explicit operator action;
the audit itself performs no Git or GitHub mutation. The configured origin must
resolve through the GitHub API to repository ID `1435549142`; the current
canonical clone URL is `https://github.com/simonvanieperen/BaryonOS.git`.

Use Gitleaks 8.30.1 and verify its release
archive before use. The existing private history workflow records the expected
Linux x86-64 release-archive SHA-256 as
`551f6fc83ea457d62a0d98237cbad105af8d557003051f41f3e7ca7b3f2470eb`.

Set `GH_TOKEN` to a read-only token that can enumerate and download the
repository's Actions data. Choose a new private state directory with mode
`0700`; files are written with mode `0600`. The state directory must be outside
the checkout, and the checkout must not be inside the state directory.

```sh
python3 -B tools/publication_audit.py \
  --state-dir /private/audit-state \
  --repo . \
  snapshot \
  --gitleaks /private/bin/gitleaks \
  --source-commit "$(git rev-parse HEAD)" \
  --allow-public-local

python3 -B tools/publication_audit.py \
  --state-dir /private/audit-state \
  --repo . \
  scan-batch \
  --gitleaks /private/bin/gitleaks \
  --batch-size 20

python3 -B tools/publication_audit.py \
  --state-dir /private/audit-state \
  --repo . \
  finalize
```

Repeat `scan-batch` until no missing receipts remain. A zero batch exit status
means only that the bounded phase completed and its state was recorded. A zero
`finalize` exit status means that the exact enumerable snapshot passed; it does
not broaden the scope described above.

Default aggregate bounds are 100,000 HTTP requests, 512 GiB downloaded,
1 TiB expanded, 10,000,000 archive entries, 1.5 TiB retained state, seven days
of charged time, and one hour per invocation. Snapshot options can lower or
raise these values only within compiled ceilings. The chosen values become part
of the immutable manifest and cannot change between batches.

Console output contains only phase state, counts, stable hashes and redacted
error codes. Raw findings, paths, signed URLs and payload bytes remain in the
private state. A later evidence record may carry the manifest hash and redacted
aggregate counts, but must not copy the private state or reinterpret the result
as historical deletion or publication clearance.

## Contract tests

The host-only suite uses synthetic inventories, archives, receipts and HTTP
responses. It does not contact GitHub or execute a real Gitleaks audit.

```sh
python3 -B -m unittest tests.test_publication_audit -v
```

The tests cover public-route rejection, repository identity, cutoff behavior,
inventory stability, deterministic resource ordering, aggregate budgets,
single-writer locking, archive traversal and nesting, authorization stripping,
stale scanner reports, receipt and retained-byte tampering, unavailable
resources, findings, coverage gaps, Git/ref changes, and the permanent
`publication_clearance: false` boundary.
