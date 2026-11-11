# M6b — bounded native object storage

## Objective and security invariants

This step turns the single M6a test value into a small local document service
in actual ring 3. It uses the custom kernel, capability IPC and virtio I/O.
Initial limits make exhaustion and recovery fully testable; they are not final
product limits.

- Four object IDs (0–3), two per domain, at most 128 value bytes per object.
- The receiving endpoint determines the domain. A request, sender number or
  physical sector in app data grants no rights. The service also checks the
  sender PID populated by the kernel.
- Only the service receives READ/WRITE/FLUSH for sectors 4–35 of a synthetic
  volume. Each app has only SEND to its own domain endpoint and RECV for its
  own responses; temporary broad bootstrap rights close.
- PUT or DELETE compares the current object revision. DELETE retains a
  tombstone with an incremented revision. An old request therefore cannot
  restore a deleted object. An empty value is an existing object.
- Validate and prepare first; then WRITE completion, FLUSH completion,
  update live state and only then ACK. On uncertain outcome: Unknown,
  no automatic replay and no further use of potentially stale in-memory data.
- 32 append-only records. Full returns Full before mutating I/O, without
  wraparound or overwrite. This step has no space reuse yet.
- Recovery scans a complete valid prefix. Any corrupt occupied record, gap,
  ordering error or invalid revision transition closes the service. The
  service does not silently choose an older version. Only an entirely empty
  journal is empty.

## Interfaces

`services/objects` is a dependency-free `no_std` crate with `forbid(unsafe_code)`.
`user_service.rs` is the separate native adapter using the existing scalar
syscall runtime. `userspace/m6b_probe.rs` contains the two test apps.
`kernel/src/m2/m3/m6b.rs` contains the bounded block broker and an explicit
acceptance observer; the ring-3 service executes object policy.

IPC version 3 uses 224 payload bytes within Message320. Fields are:
version at 0, operation at 2, request ID at 8, service epoch at 16, object ID
at 24, expected/response revision at 32, length at 40, status at 44 and value
at 64. Other bytes are canonically zero. GET=1, PUT=2, DELETE=3. NotFound=-19
may return the tombstone revision for an authorised object; Access does not
reveal it.

A 512-byte journal record has magic `NOSM6B01`, version 1, header size 64,
global sequence at 16, request ID at 24, object ID at 32, object revision at
40, operation at 48, length at 50, data at 64 and FNV64 at 504. The checksum
is calculated over the complete record with a zero checksum field. This is
not cryptography.

## Native acceptance

Build: `python3 -B tools/build.py --kernel-feature milestone-m6b`.
Smoke: `python3 -B tools/run_m6b.py --case normal --smoke`.
Complete gate: `python3 -B -u tools/test_m6b.py`; then
`python3 -B tools/capture_m6b.py` for rechecking and text archiving.

Normally, each app creates two objects, updates one, deletes one and reads
both back. Final state has sequence 8: objects 0/2 live at revision 2, objects
1/3 tombstones at revision 2. A cold second VM must change no disk bytes and
read back exactly that state.

Negative trials check every CRUD operation across the domain boundary,
malformed/padding/epoch/revision, incorrect handles, rights and closed handles.
The observer checks that rejected service requests cause no device I/O. The
full trial performs 32 actual writes; additional writes return Full without
overwrite. The host independently checks all journal bytes and canaries
outside scope.

Two external QEMU kills: before the first WRITE publication and after the
first actual mutation FLUSH but before ACK. A second cold VM must read back
the old empty state or the written first record respectively. The host must
actually terminate a still-running QEMU child; a marker followed by voluntary
exit is insufficient evidence. A damaged journal must be rejected before READY.

<a id="bewijsgrenzen"></a>

## Evidence boundaries

### K08 follow-up: deadlines, cancellation and quarantine

After `7b4945f`, five separate cases follow: WRITE_TIMEOUT, FLUSH_TIMEOUT,
RESET_TIMEOUT, WRITE_POLL_LIMIT and FLUSH_POLL_LIMIT. Suppressed notify is an
actual queue publication without notification; unexpected actual completion
must not pass as timeout. Kernel deadline and peer progress are observed.
The reset trial masks only observation of reset acknowledgement and retains
the five pinned DMA pages without reuse until VM stop.

On its own poll LIMIT, the service closes its sole block handle through
existing cap_close. Only verified loss of actual manifest authority can inform
the kernel observer of uncertainty; no user boolean counts as kernel evidence.
The owned pending ticket is detached, and published DMA still requires a
confirmed reset. No new syscall or right. A compile-time test budget shortens
only the selected mutating poll; startup and normal profiles retain their budget.

All five cases must demonstrate nine Unknown responses and zero ACKs,
handle repeated mutations/GET fail-closed and preserve domain rejection.
Suppressed WRITE leaves the volume empty; suppressed mutation FLUSH leaves
exactly the first record. Every case receives cold recovery without writes.
The expanded complete gate requires 32 groups / 28 actual VM runs. This
contract is not a test result; current execution appears in PROJECT-STATE.md.

Historical limitation at base `1f24f99`: on a WRITE or FLUSH error,
`kernel/src/m2/m3/m6b.rs` can cause a kernel panic because `sent()` still
compares the reply against expected `Ok` before `Unknown` is processed
correctly. Older native gates do not cover this error path. The seven host
ABI trials check only the service adapter with simulated I/O outcomes.

### Native follow-up: unknown WRITE/FLUSH outcome

Contract before execution, 28 October 2026: repair the observer generically,
including for `milestone-m6b`. Mark uncertainty on a terminal mutation error
actually returned to the service. Then expect only a canonical Unknown response
with zero revision/length/data; no in-memory commit or app ACK. All subsequent
valid GET/PUT/DELETE and repeated PUT return Unknown without I/O; invalid
domain access remains Access.

Two compile-time fault cases, `test-m6b-write-error` and `test-m6b-flush-error`,
replace one local completion status with IOERR after an actual WRITE or the
first mutation FLUSH respectively. Startup FLUSH remains normal. No descriptor
or actual device response is changed. This trial is not an actual failing
backend, partial write, timeout or power failure.

The first PUT is deterministically object 0, revision 1, request ID 10. Both
client processes then check Unknown, no replay, no old GET content and
unchanged rights. The observer requires nine sent and received Unknown
responses, zero mutation ACKs and requests `[6,5]`. Publications are
`[128,1,1]` for WRITE error and `[128,1,2]` for FLUSH error (including startup);
one error completion is counted separately. Log no successful live-state claims.

Exact guest markers after BOOT/EBS, kernel entry and BootInfo: one
`M6B:INJECT:WRITE_IOERR_SNAPSHOT` or `M6B:INJECT:FLUSH_IOERR_SNAPSHOT`,
`M6B:OUTCOME:UNKNOWN`, `M6B:NO_REPLAY:PASS`, `M6B:NO_STALE_READ:PASS`, then
case/survivor/resource/M6B/M1 success markers and raw exit 33. Missing,
repeated or incorrect markers, panic and timeout fail. Other cases must not
adopt injection/Unknown markers.

The external disk oracle requires exactly the first valid PUT record and
otherwise empty journal space/unchanged canaries. Because injection occurs
after actual completion, that record may already be on disk while the service
must not acknowledge it. A separate cold VM must read back the same bytes,
report sequence 1 and write nothing. The complete gate expands by four VM runs
to 22 groups / 18 VM runs.
`capture_m6b.py --destination docs/verification/m6b-storage-errors` retains
new evidence without replacing the old archive. Actual results and remaining
coverage appear in PROJECT-STATE.md; these requirements alone are not a
successful test outcome.

Host unit tests, native execution and independent disk checking are recorded
separately. A QEMU kill is not physical power loss. The QEMU device and kernel
transport are trusted, without an IOMMU or hostile-DMA claim. There are not
yet filenames, directories, a native desktop, encryption, NAS sync, a general
service supervisor or protection against hostile disk rollback. Historical
payload bytes remain in the append-only journal after a tombstone.
