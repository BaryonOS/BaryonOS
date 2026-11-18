# Pipeline continuation — 18 November 2026

**Source:** `65ae5bacdee969a6e9da7d10909248d76af07fd8`, based on
`7e406c7c3ec71691939acca6db0c594bb8ce03a9`, existing PR #355.
The authorization manifest remains disabled. This checkpoint does not establish
complete remediation, production client compatibility, or merge readiness.

## Verified corrections

The controller rechecks both its local owner policy and the manifest at the freshly
fetched main SHA before creating a worktree or starting L0. Previously, a
manifest-only revocation could leave the cached grant effective. The new regression
fails on the previous implementation because L0 is invoked after revocation.

Only a successful model attempt may import validated edits. Failed or rate-limited
attempts retain their raw logs and discard their disposable changes; a fallback
starts from the unchanged controller source. The regression fails before the fix
because a failed attempt changes controller source.

The execution-hold test uses a consistent `unittest.mock` import. The native
descendant-termination fixture waits for an explicit child readiness message before
testing cleanup; an initial real run exposed the former scheduling assumption.
The timeout and normal-exit assertions remain enforced.

## Executed verification

The command records, raw exit statuses, file hashes and local log digests are in
[`checks.json`](checks.json). On the committed source, the complete tools suite
ran 445 tests: 442 passed and three Rust host probes were skipped because the
activation script was absent from that invocation. Those three probes were run
separately with the existing toolchain activated. The complete suite includes nine
actual Linux boundary tests and one offline transaction fixture using real Git and
dry-run publication. Synthetic fixture output is not authenticated model-client
parity. Strict preflight, the native build and QEMU smoke passed; QEMU returned 33
with `BOOT:EBS_OK`, `KERNEL:ENTRY`, `BOOTINFO:OK` and `M1:PASS`.

The fixture uses a locally built, unprivileged bubblewrap 0.13.1 from the official
release asset with its GitHub-published SHA-256 checked before extraction. Its
libcap 2.78 packages were checked against the Debian package-index SHA-256 values.
The minimal runtime contains the host Python standard library, Python, Bash and
their dynamic libraries, with a deterministic fixture agent. The backend and
runtime digests are pinned in the local fixture configuration. Neither runtime
binaries nor credentials are published. This runtime does not supply production
resource enforcement or the complete host-quality environment.

## Recovery PR #357 disposition and remaining blockers

Recovery commit `8a32bc582790fa16234dbadf37a0a176de620900` remains preserved,
unmerged and unaccepted. Its four changed files were reviewed; none was imported
into this checkpoint. The proposed resource design still needs verification or
replacement in the following areas:

- Periodic workspace accounting does not establish a kernel-enforced aggregate
  scratch quota. Source staging, retained gate outputs and their cleanup also
  require a bounded complete lifecycle.
- Child process limits alone do not establish bounds on all launcher output or
  controller-side retained output.
- Process-group termination is not evidence of complete resource-scope cleanup
  under every startup, cancellation and failure path.
- The Cloud host exposes no writable delegated cgroup or private user-manager
  bus. Its installed systemd is 257; the recovery design requires at least 259.
  User and network namespaces are available, and the separately provisioned
  bubblewrap 0.13.1 works for the bounded fixtures above. No unisolated fallback
  was used.
- Authenticated selected model clients still require a reviewed controller-side
  interaction boundary and an owner-controlled engine allowlist. Their output
  remains untrusted. The fixture agent does not complete this requirement.
- The complete selected CI gate still needs reviewed offline dependency
  preparation, Git metadata and explicit writable cache locations. Successful
  ordinary hosted CI does not demonstrate this isolated gate's compatibility.

Independent review and current-head hosted CI remain required. The security finding
must remain open until the complete boundary and the repaired integrated main are
verified through the supported resolution workflow. Score: 41/100, unchanged.
