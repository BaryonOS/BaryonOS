# GUI-BOOT-INIT-1: separate native init/supervisor

This contract describes the new CPL3 component `sdk/examples/desktop-init`.
The first component check is a host protocol test plus an actual scalar
ELF build, not native desktop acceptance. Kernel bootstrap, grants, image
construction, actual QMP interaction and complete release acceptance belong
to the main integration and require separate evidence.

## Authority and boot metadata

The seven-process profile uses init1, keyring2, volume3, Materiaal4,
compositor5, inputsource6 and inputpolicy7. Init has only RegistryRECV,
AppControlSEND and CompositorControlSEND. No personal volume, keyring,
network, display, raw input, physical memory, LaunchImage or ChildProcess.
Native IPC comes directly from the SDK. The shared input-daemon library
provides only the existing Ipc/NativeIpc adapter and capability cleanup;
init performs no driver, policy or display I/O.

Private DATA startup page: 1024 bytes at 0x40010000, all fields u64:

| Byte | Value |
| --- | --- |
|32|`u64::from_le_bytes(*b"NOSDINI1")`|
|40|BootGeneration>0|
|48|RegistryRECV|
|56|AppControlSEND|
|64|CompositorControlSEND|
|72|optional ControllerControlRECV, otherwise 0|
|80/88/96/104|actual App/Comp/Source/Policy PID, exactly 4/5/6/7 for this profile|
|112|SurfaceEpoch>0|
|120|PolicyEpoch>0|
|128|expected NativeDisplayEpoch from kernel GOP, >0|
|136|expected ClientID>0, for the manifest binding; no authority from the payload|
|144|optional ControllerReplySEND, otherwise 0|
|152|optional expected ControllerPID, a separate process above 7; otherwise 0|

The three controller values are either all present or all 0. All handles
have valid representations and are mutually distinct; the kernel checks
type/rights/holder. Fields 160..1023 are initially 0. PIDs or reply routes
are never chosen from a request. The normal seven-process variant has no
test controller, and init does not stop automatically in that variant.

## Readiness and actual focus

Init receives at most four messages per round on one RegistryRECV. An
IPDREAD1 message must carry the actual kernel sender, zero received
capabilities, exact tag, 32B length, `IPD1`, version 1, role, epoch and BootGeneration:

- SourcePID6→role1, PolicyEpoch, actual RawEpoch>0.
- PolicyPID7→role2, PolicyEpoch, RawEpoch0.
- CompositorPID5→role3, SurfaceEpoch and exactly the expected NativeDisplayEpoch.

Under its own contract, the source may report this only after actual hardware
Ready and a canonical SYNC ACK; the compositor only after rendering its full
chrome and empty assigned app rectangle through actual display blits. Init
does not read their memory and does not replace those separate native
properties with claims of its own. A fabricated payload field or tag grants
no sender authority.

AppPID4 sends SYSTEM tag 0x53595354454d0001 with payload[1] as **StartupReady**.
This is not a fully drawn frame, successful document I/O or evidence of
focus. Only after all three IPD roles and AppStartupReady are present does
init publish exactly one CMP1 FocusStart, requestID1, bound to
SurfaceEpoch/BootGeneration. Send-EAGAIN retains the same bytes. After sendOk,
init waits for the actual compositor reply; the request is not republished.

Only the canonical capability-free reply from CompPID5 with the correct
opcode|0x8000, ID1, epoch, generation, status0 and stateFocused1 allows SYSTEM
GO[2] to reach the app. GO counts only after actual SDK-sendOk. InitReady means
only this GO publication after checking dependencies and focus. AppReady400,
PresentedGeneration and actual UI/storage scenarios remain separate
observation/interaction criteria. Unexpected AppDONE outside an active STOP
does not authorise treating the system as shut down.

## Optional STOP route and refusal

The optional native controller has only its fixed initControlSEND and
personal ReplyRECV. DINITCTL is tag `*b"DINITCTL"`, payload48B:
`DIN1`@0,versionu16=1@4,opcodeu16=Stop1@6,requestIDu64@8,
SurfaceEpoch@16,BootGeneration@24,statusu64@32,reservedu64@40. Request status
and reserved are 0. RequestID is nonzero and strictly monotonic. The exact
kernel PID and fixed controller route are required. Reply opcode is 0x8001;
status is Stopped0, Refused1 or Failed2, reserved0. A reply receives the same
bindings and uses only the fixed ControllerReplySEND.

Init publishes SYSTEM STOP[5] once and waits for AppPID4. App[6]
STOP_REFUSED means unsaved/busy work: init replies Refused and remains
Running; AppDONE/Retire/DONE are not recorded as achieved. App[3] DONE is
valid only within this active STOP after publication. CMP1 Retire ID2 then
follows. Only the actual compositor Retired ACK (Focus0 plus actually visible
app-rectangle clearing blits under the compositor contract) permits Stopped.
A missing or invalid Retire ACK produces Failed and never Stopped.
Pending controller replies remain unchanged on EAGAIN; no second STOP,
Retire or implicit save retry. Messages already queued cannot be retrieved
from another queue. The existing one-byte app protocol correlates by phase
and one outstanding STOP; it is not a general transaction service.

## Separate test-controller process

Only `native-system-materiaal-probe` may receive an eighth CPL3 process, PID8.
This is explicit test orchestration, not a user feature or an extra right
of an ordinary app. The normal desktop variant remains seven processes.
Controller metadata: 32`NOSDCTL1`,40BootGeneration,48InitControlSEND,
56ownReplyRECV,64InitPIDexact1,72SurfaceEpoch,80ownPIDexact8.
88..399 and 464..1023 are reserved 0.

The controller reads only its own barrier word 400. Initially 0; the kernel
test observer may set it to 1 **after** independently checked actual QMP
actions, Saved and Presented. The barrier injects no UI or document content
and does not itself prove saved/presented work.
The controller uses actual Sys3/PIT time, publishes Stop ID1, waits for the
exact canonical Stopped reply from InitPID1, closes its two handles and only
then sets its own Done456=1 before exit0. Refused, Failed, extra capabilities,
incorrect binding or timeout produce no Done. Its own BootGeneration at 40
is retained.

Controller words: 400barrier,408request-sendOk,416canonicalStopped-reply,
424phase,432fatal code,440starttick,448endtick,456Done. No access to other
processes' RAM, keys, display, input, documents or host functionality.

## Bounds and observation

All deadlines are specified in advance in actual 100Hz PIT ticks: bootstrap5000,
CMP exchange500, AppSTOP1000, ControllerReply200, testbarrier20000 and
testcontroller-RPC2000. Each phase retains its original deadline;
EAGAIN does not renew it. Backward-moving or overflowing time is fatal.
This is monotonic time, not a clock for certificates or wall time. One round
performs at most one controller receive, four registry receives and one
pending send; all wait loops yield CPU time. Unexpected received capabilities
are closed once before further processing. Pixel/key/document payloads are
not logged. Native duration and deadlines must still pass on the actual images.

Init words: 400SourceReady,408PolicyReady,416CompReady,424AppStartupReady,
432actualFocusACK,440GOsendOk,448STOPsendOk,456STOP_REFUSED,
464actualRetireACK,472ControllerReplySendOk,480phase,488fatal code,
496rejections,504closed received capabilities,512AppDONE,
520registryreceives,528InitReadyTick,536StartTick. Phases are
Boot0/Focus1/Go2/Running3/Stop4/Retire5/Reply6/Stopped7/Failed8.
These are supporting observations, not authority or independent evidence
of rendering, storage or correct shutdown.

There is not yet an actual crash-notification/restart agreement for the
critical GUI services. Init therefore receives no invented status or kill
rights and makes no automatic crash-recovery claim. Deadline rejection is
not recovery. Complete session/shutdown policy, multiple windows, restarting
input epochs, accessibility and full product acceptance remain open.
