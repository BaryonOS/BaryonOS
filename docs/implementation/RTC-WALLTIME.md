# RTC as an unauthenticated UTC source

`drivers/rtc` is a standalone allocator-free `no_std` crate with
`forbid(unsafe_code)` and no dependencies. Base commit:
`cda42f2d4017e65bd8ba89399cb1633f85a1c2da`. The crate provides only the CMOS
read procedure, date codec and explicit source quality. The opt-in
`native-rtc` profile adds a native port backend and bootstrap acceptance.
The separate `native-system-clock` profile adds ABI 1.9 syscall `0x73`:
a ring-3 app requests a current UTC/Unix-second value, which the kernel reads
directly and read-only from CMOS. The SDK labels every result
`UnauthenticatedRtc`. This is not a trusted TimeProvider or certificate
integration. Sys3 remains the separate boot-monotonic deadline clock; ticks
are not converted into a calendar date.

## Properties and trust basis

| Property | What this crate enforces | Required assumption/open boundary |
| --- | --- | --- |
| Bounded I/O | Only typed index selection, settle and data read; no CMOS data write. | The opt-in native backend uses only ports 0x70/0x71; no general app I/O. |
| NMI/index | Every selection uses the same explicit NMI policy; the original index is restored on success and error. | The platform owner supplies a reliable index/NMI shadow and excludes all other CPU/IRQ/NMI users. |
| Consistent date | Two complete identical snapshots, UIP before/after, constant mode and valid fields. | Correct device behaviour and bounded adapter operations; this does not authenticate the clock. |
| Bounded execution | One global register-read budget and a bounded number of snapshot attempts. | Native I/O and settle themselves must return within bounds; this is not a measured time deadline. |
| UTC and century | Only an explicit UTC platform basis plus an explicit century register. | The RTC actually contains UTC; no local-time detection or configuration authentication. |
| Unix conversion | Valid Gregorian date, year 1–9999, checked i64 operations. | No leap-second representation; second 60 is rejected. |
| Rollback | Comparison with an earlier accepted reading or caller floor; a lower time returns an error. | In-memory state is lost on restart. The crate neither authenticates nor stores an external floor. |
| Trusted certificate time | **Not provided**: the SDK always returns `UnauthenticatedRtc`. | An authorised TimeProvider, freshness, uncertainty margin and rollback anchor are absent. |

A stable clock may be wrong, stopped or set back by an attacker. An equal
value is accepted; large forward jumps are not recognised as attacks. The
presence of a timestamp therefore gives no permission to skip certificate
dates, accept an invalid certificate, use TOFU or present this source as
reliable time. With missing or insufficient time quality, the future TLS
boundary must return a concrete time/validity error. That policy is not
simulated here as already implemented TLS security.

## Interface and lifetime

`Rtc::new(io, initial_selection, configuration)` takes exclusive access to
the index/data pair without I/O. `CmosIo` exposes:

```rust,ignore
fn select(&mut self, selection: IndexSelection) -> Result<(), Self::Error>;
fn settle(&mut self) -> Result<(), Self::Error>;
fn read_data(&mut self) -> Result<u8, Self::Error>;
```

`select` writes only index port 0x70 with `selection.encoded()`;
`read_data` reads only 0x71. There is no data-write interface. `settle`
provides only the bounded delay, without an additional register read or
index/NMI change; restoring register C must not acknowledge an interrupt
either. The entire read, including restoration, is serialised against all
other users of the pair. A Rust `&mut` or trait is not a kernel permission
and does not itself exclude another CPU/NMI handler.

Port 0x70 is not read back to guess NMI state. Bootstrap must know/own the
original index and explicit NMI policy and provide an `IndexSelection`.
The reader does not change the NMI mask bit during the procedure. If index
restoration fails, `ReadError` contains both the primary error and restoration
error, and the instance becomes poisoned. Subsequent reads perform no I/O.
`into_io()` transfers the pair for controlled adapter recovery; no hidden
I/O occurs in Drop.

Configuration selects a century register from 0x0e through 0x7f inclusive.
The position must come from checked platform configuration/FADT. The first
native QEMU trial will explicitly use 0x32. A missing century or unknown
time basis returns an error before I/O; there is no pivot based on build
year, host time, current date, local timezone or a prepopulated timestamp.

`read_utc()` returns `Reading { utc, date, source, quality, rollback }` or an
error. `UnixUtc` contains signed i64 Unix seconds for year 1–9999; `date`
retains the actually decoded fields. `source=CmosRtc` and
`quality=UnauthenticatedRtc` are fixed, visible values. An external minimum
time appears as `FloorSource::CallerSupplied`; the crate thereby claims no
authenticated storage. A successful read updates its own in-memory floor;
failed reads and failed index restoration do not.

The crate has its own `[workspace]` and lockfile. The later integrator must
explicitly arrange connection to the shared workspace; no shared Cargo,
kernel, SDK or build files have been changed at this stage.

## Hardware codec and error paths

The original core register contract is the QEMU 10.0.13 MC146818 model;
the native trial below explicitly measures QEMU 10.2.1 with the q35 profile.
This does not support every physical RTC model. Register A must indicate
the running 32.768-kHz divider; UIP must not change during the read. SET in
B or a stopped/unexamined divider returns an error. Automatic DST in B is
rejected for this UTC profile. D must have exactly the valid battery/time
value 0x80. A floating 0xff bus or explicit offline I/O returns no time;
a zero value is conservatively `BatteryInvalid` and does not independently
prove whether a device is absent. Register C is not read, so reading does
not acknowledge an RTC interrupt.

The reader handles BCD/binary values and 12/24-hour mode without changing
mode. AM 12 becomes 00; PM 12 becomes 12. Non-decimal BCD nibbles, invalid
fields, calendar days, century, leap days or a PM bit in 24-hour mode are
rejected. A month/year transition or changed mode causes another attempt.
At most 65,536 register reads and 32 snapshot attempts are configurable;
a smaller configuration may legitimately fail before a complete snapshot.
Every error path attempts to restore the original index/NMI word.

No alarms, IRQ modes, RTC date, CMOS contents, firmware or physical clocks
are changed. The pure RTC core itself performs no actual port I/O; only the
opt-in bootstrap adapter below performs it.

## Sources, reuse and licences

- QEMU, *MC146818 RTC emulation*, version **10.0.13**:
  <https://github.com/qemu/qemu/blob/v10.0.13/hw/rtc/mc146818rtc.c> and
  <https://github.com/qemu/qemu/blob/v10.0.13/include/hw/rtc/mc146818rtc_regs.h>.
  This is the primary source for the preliminary emulator profile, not
  hardware certification. Source files were read; no emulator code was
  incorporated.
- Howard Hinnant, *chrono-Compatible Low-Level Date Algorithms*,
  1 September 2021, `days_from_civil`:
  <https://howardhinnant.github.io/date_algorithms.html#days_from_civil>.
  The March/400-year formula is reused with validated input and checked i64
  operations. The author explicitly dedicates the formulas to the public
  domain. The source read was obtained through the upstream GitHub mirror;
  <https://github.com/HowardHinnant/HowardHinnant.github.io/blob/master/date_algorithms.html>.
- ACPICA, `ACPI_TABLE_FADT.Century`, read 30 October 2026:
  <https://github.com/acpica/acpica/blob/master/source/include/actbl.h>.
  The field supplies a register index; this crate implements no FADT parser.
- Supplementary reading index: OSDev, *CMOS*, **W001717**, revision 30263
  of 16 March 2026:
  <https://wiki.osdev.org/index.php?title=CMOS&oldid=30263>.
  The PC NMI/index convention is fixed at the explicit adapter boundary.

The manufacturer's datasheet could not be retrieved from this environment
(HTTP 403). No physical RTC support is therefore approved. Sources actually
read are recorded with SHA-256 in the verification report. There are no new
libraries, build scripts or external dependency versions.

## Native bootstrap contract RTC-NATIVE-1

Baseline `5c334f9`, branch `simonvanieperen-expert-disco`. This is a direct
technical dependency of FR-S03/FR-N03, not complete acceptance. The purely
diagnostic `native-rtc` feature runs after the project's own GDT/IDT, page
tables and M2 memory check, on the single CPU with IF=0. Combining it with
ring 3 or framebuffer is rejected at compilation. No production profile
enables the feature implicitly; no process receives new I/O permissions.
In this terminal trial, the small codec runs in the kernel to test the
actual hardware adapter.

Platform ownership is explicitly established by selecting register D with
NMI Enabled. This is a known starting selection established by the OS,
**not reconstruction of the firmware selection/NMI policy**. The trial does
not mask NMI; its own NMI handler is terminal and does not use CMOS. There
are no firmware calls, other CPUs, IRQ/userspace CMOS users or shared port
references. The backend has only `out8(0x70)` and `in8(0x71)`, no data writes,
general port arguments or port-0x80 delay. QEMU processes the index write
synchronously; the bounded scalar settle loop is not a physical-hardware
timing contract. Register C is not read. The q35 platform configuration
explicitly selects century 0x32 and UTC; no FADT parser or automatic platform
detection is claimed.

Acceptance: two actual UTC readings with visible `UnauthenticatedRtc`,
date/Unix equality and an in-memory floor; an actually exhausted read budget,
rejection of an unattainable caller floor, unknown time basis/missing century
without I/O, then another successful read. After success and both I/O error
paths, the starting selection is restored: a software index check plus an
actual data read **without reselection** must yield register D=0x80. This
checks register selection on the device; NMI policy is an explicit code/
platform invariant, not an independently measured hardware guarantee. Pure
host tests retain corrupt/offline/UIP/mode/poison counterexamples; native
negative cases inject no fictional device replies.

## ABI 1.9 trial NATIVE-CLOCK-1

`native-system-clock` is a separate opt-in single-CPU system profile, not
general hardware support. The app calls syscall `0x73` without arguments or
pointers. On success, it returns `RAX=0` and `RDX` holds signed Unix seconds
as a two's-complement value; invalid extra arguments return `-22`, an
unusable RTC returns `-5`, and profiles without the syscall continue to
return the normal unknown-syscall error (`-2`). The SDK provides
`native::wall_clock_utc()` as `WallClockReading { unix_seconds, quality }`;
the only quality is `UnauthenticatedRtc`.

Each call performs a bounded read-only snapshot, checks the RTC core and
register-D restoration, and restores the original index selection. The call
has no user pointer, capability or file access. This is public local time
data, not a new permission to other resources. The adapter still assumes
exclusive CMOS ownership on one CPU with IF=0, an explicit UTC basis,
century register 0x32, no IRQ/NMI/other CMOS user and an NMI handler that
does not itself read CMOS. There is no persistent anti-rollback,
authentication, freshness/uncertainty evidence, time synchronisation,
suspend correction or TLS certificate policy. Monotonic PIT ticks are not
used to estimate or correct wall-clock time.

The native trial has the existing ring-3 system app read two times, checks
their order and monotonic ticks, and tests a syscall with a non-zero argument.
The independent runner compares the reported seconds with an explicit QEMU
RTC base time. The executed trial is in
`docs/verification/system-clock/README.md`. This closes no FR-S03/FR-N03
product criterion.

```sh
NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B tools/build.py --kernel-feature native-rtc
NOS_QEMU_MACHINE=pc-q35-8.2 python3 -B tools/run_rtc.py --case all
python3 -B -m unittest tools.tests.test_system_clock
python3 -B -u tools/run_system_clock.py
```

The runner uses the existing `run_guest` with a dedicated ESP/VARS/CODE copy
per actual VM and `-rtc base=<expliciete UTC>,clock=vm,driftfix=none`
(the placeholder denotes explicit UTC). Five calendars: millennium 2000,
leap day 2000, 2026, the boundary after signed-i32 2038 and leap day 2096.
Python datetime independently checks both the six fields and Unix conversion,
and the window base..base+60 seconds; this window covers firmware/boot and
does not prove real-time accuracy. No host time as an OS fallback, no network,
no real disk and no writes to RTC data. Raw exit 33, complete ordered output,
source fingerprint, manifest and artifact hashes are required. Timeout,
wrong date, duplicates or changed source fail; a stale latest report is
withdrawn first. CI has its own mandatory executing workflow `native-rtc.yml`.
`--case epoch` and `--case century` remain explicit firmware diagnostics,
outside normal five-calendar acceptance: on the measured OVMF pair, 1970
actually became 2070 and 2100 actually became 2000. The unchanged oracle
rejects both with exit 1, despite raw 33 and passing read/restore markers.
EDK2 `edk2-stable202511`, `PcAtChipsetPkg/PcatRealTimeClockRuntimeDxe/PcRtc.c`,
`ConvertRtcTimeToEfiTime` reconstructs the century from the PCD minimum-year
period; `PcRtcAcpiTableChangeCallback` writes it to the FADT century register.
This explains the measured behaviour; the native backend neither corrects
nor guesses another century. Core Gregorian host tests remain the broad
date evidence; native 1970/2100 acceptance and a complete firmware-independent
date range remain open. New firmware packages are actually retested, not
automatically treated as equivalent to this development variant. This trial
provides no authentic time, persistent anti-rollback, NTP, TLS, userspace use,
physical-device or complete product acceptance.

## Historical core evidence and next production gate

Executed with Rust 1.90.0 and separate target directories:

```sh
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-rtc-target \
  python3 -B tools/dev.py cargo +1.90.0 test \
  --manifest-path drivers/rtc/Cargo.toml --locked --offline
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-rtc-target \
  python3 -B tools/dev.py cargo +1.90.0 clippy \
  --manifest-path drivers/rtc/Cargo.toml --locked --offline --all-targets -- -D warnings
LOCALAPPDATA=/workspace/nieuw-os/.tools CARGO_TARGET_DIR=/tmp/nieuw-os-rtc-none-target \
  python3 -B tools/dev.py cargo +1.90.0 build \
  --manifest-path drivers/rtc/Cargo.toml --locked --offline \
  --release --target x86_64-unknown-none --lib
```

Eleven tests pass without skips: overflow, fixed epoch anchors and all
146,097 days of a Gregorian 400-year cycle, BCD/binary hour modes, invalid
dates, update/mode/midnight transitions, stuck UIP, missing device, read
budget, restore/poison and visible rollback floor. Clippy and the freestanding
library build pass. `drivers/rtc/verification/` contains bounded source/
artifact-hash evidence. This is host evidence and a target build, not native
execution of the RTC driver.

The next production gate remains open: a minimally authorised broker/
TimeProvider, shared CMOS/NMI ownership with IRQ/SMP, actual transitions/
device errors and then time quality during certificate validation. A physical
clock, secure time and a persistent rollback anchor are not yet proven.
This step does not independently change release scope or progress points.
