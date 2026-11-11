# K05 example: supervisor, worker and client

Three ordinary `no_std` programs for the Nieuw OS ABI 1.3 (profile
`native-supervisor`, ADR-042). Together they prove that a ring-3 process with
only capabilities can start, monitor, restart and terminate services.

| Program | Authority at start | Behaviour |
|---|---|---|
| `k05-supervisor` | two `Launch` capabilities from the boot manifest | creates a registration endpoint, starts worker and client with a `SEND` capability on it, brokers the worker ingress to the client, monitors the worker with `proc_status`, restarts it after a `proc_kill` (normal) or an observed `#UD` (crash) and terminates all children before its exit |
| `k05-worker` | one `SEND` capability to the registration | creates its own ingress endpoint, registers a `SEND\|TRANSFER` capability on it, answers numbered requests from the one client the supervisor names; in crash mode `ud2` after eight replies |
| `k05-client` | one `SEND` capability to the registration | creates a reply endpoint, sends 24 numbered requests and after `ECLOSED` continues with the same sequence number as soon as a new worker has been published |

The protocol (`src/protocol.rs`, lib `k05_protocol`) is an app agreement in
ordinary Message320 payloads; the kernel does not know it and only reads the
evidence words on the supervisor's start page plus its own counters.

Modes via start word 0 (low byte) and the PID of the starter (from bit 8):
worker `0` serve, `1` crash after 8, `2` idle (quota test), `3` negative
checks; client `0` 24 requests, `3` negative checks. The supervisor
reads its case from word 0/112: normal, crash, denied, parent-exit, quota.

The separate sixth case `wait` (`test-supervisor-wait`, ABI1.10)
combines this same process mechanism with IPC waiting. Worker modes4/5
register a real empty-ingress wait with a deadline40/500 ticks.
The supervisor lets both actually park, checks that a
Process handle without DERIVE may not be derived to OBSERVE and that
an endpoint is not a Process kill right, terminates the first with KILL,
closes its handles and starts a fresh PID in the same slot. Old status/
kill handles must then give `EBADHANDLE`. After80 ticks, past the old
deadline but before the new one, the successor must also still really be Waiting
at its authorised kill. Every unexpected return from the worker
wait fails explicitly; no readiness or kill is simulated.

Only this explicit test profile reads, at the existing kill/spawn
boundary, the actual Waiting status and cleared wait metadata, without
changing capability enforcement, scheduler or cleanup. The final oracle
requires two Waiting kills, one clean slot reuse, four real blockings,
two parent deadline wakes and complete Engine/frame recovery. The host gate
checks those numbers independently, including actually counted idle
IRQs within the monotonic tick counter. The five legacy cases remain mandatory;
`tools/test_supervisor.py` now runs six separate QEMU VMs.
Execution status and source-bound evidence are in
`docs/verification/native-test-version/`; this description is not native
acceptance in itself.

The first native wait case rightly failed on an incorrect test assumption:
K05 gives Process exclusively `OBSERVE|KILL`, without `DERIVE`. The test therefore
does not create an OBSERVE-only handle and does not widen any rights to make that possible.
Instead it expects `EINVAL` for that derivation attempt and `ETYPE` for
the endpoint kill, alongside both stale-handle refusals.
`Engine::require` refuses DERIVE first as invalid for the Process
object type; it is not merely a missing permitted right (`EACCESS`).
The second red step104 run rightly kept those error codes apart.

```sh
python3 -B sdk/examples/supervisor/build.py
```

Default output: `build/sdk-supervisor/x86_64-unknown-none/release/k05-*`.
The kernel gate `tools/test_supervisor.py` builds these binaries itself via
`tools/build.py --kernel-feature native-supervisor` and variants.

## Boundaries

No block, device or network right; no persistent service configuration;
no policy for who may be supervisor. The programs are
acceptance fixtures for K05, not a product service manager.
