# FR-I01 — combined native system services

29 October 2026. Base `73ebc1e169b893eb5aba4b4f8ac06d7798136de3`.
This is the task contract for the first integration step of the full
test release. The full product acceptance remains separately open in
[FULL-TEST-RELEASE.md](../FULL-TEST-RELEASE.md).

## Invariant and result

One own kernel concurrently runs a ring-3 init, Key Service,
document volume service and ordinary app. A worker started by init can stop
and start again. The app uses real capability IPC, virtio block I/O and
cryptographic processing by the Key Service. The kernel stores no
content keys and interprets no document format.

The existing profiles remain regression trials. `native-system` is a
combined integration profile, not yet a full release, interactive
desktop or general installation of dynamic services. The fixed
256 MiB regression profile does not prove support for the planned
8 GiB product profile.

## Ownership and interfaces

- Kernel worker: `kernel/**`, new `sdk/examples/system/**` and exclusively
  necessary generic adjustments of existing service adapters.
- Integrator: build/run/test tools, workspace members and lockfiles, CI,
  acceptance evidence, project status and progress. One writer per file.
- The existing Message320/SDK interface, volume-v6 and keyring-KEY1 wire
  remain leading. Block and entropy capabilities belong only to the
  corresponding service; a signature or service status grants no right.
- Boot ELF input: `NOS_SYSTEM_INIT_ELF`, `NOS_SYSTEM_APP_ELF`,
  `NOS_SYSTEM_WORKER_ELF`, `NOS_VOLUME_SERVICE_ELF`,
  `NOS_KEYRING_SERVICE_ELF`. The build checks each ELF separately.

## Acceptance before signing off FR-I01

1. Start from a new own supervised synthetic disk; both real
   services become READY, the app saves via the volume service and reads back.
2. The Key Service generates with real virtio entropy, encrypts and decrypts
   an unknown plaintext from the app. Wrong domain/tampering fails.
3. The app gets no direct block, entropy or process management rights;
   actually forbidden syscalls are refused.
4. Init starts and restarts a separate worker; error/exit is handled via the
   normal cleanup path. All process, capability and frame counters
   return to the recorded baseline after the finite trial.
5. The independent Python disk oracle checks actually stored
   bytes, revisions, geometry and canaries. Serial markers alone are insufficient.
6. The harness check refuses timeout, error marker, missing/duplicate or
   invalid observation, wrong raw exit, changed source and wrong disk state.
7. Repeat the relevant supervisor, keyring, volume and recovery regressions on
   the integrated source. New host tests or compilation do not replace these.

## Open product boundaries

This step still uses the existing small trial geometry and RAM keys.
Persistent secure storage, cold rollback anchoring, general package/
service start, full memory/thread support, graphical session,
network, apps and AI remain mandatorily open. The profile must not present those functions
as hardware absence or successful simulation. The fixed score
does not rise from this integration alone.

The new B02 context records and persistent key records first get their
own cryptographic and error tests. Native wire/storage integration and a real
trusted rollback anchor source remain necessary after that.

## Security and evidence

Only newly generated files under the designated build directory, private OVMF-VARS,
no host disk, personal data, network connection or production keys. Evidence
contains source commit and fingerprint, tool/firmware versions and hashes, exact
QEMU command, raw exit, serial, disk hash before/after and independent oracle.
These tests yield no physical-hardware, human-audit or production claim.
