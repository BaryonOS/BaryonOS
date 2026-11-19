# ACPI S3/S5 parser core

## Purpose and trust boundary

This dependency-free `no_std` crate validates bounded raw FADT and DSDT SDT
snapshots and extracts S3/S5 sleep types and legacy x86 PM1a/PM1b control
register addresses. The S3 route additionally validates a bounded FACS snapshot.
Public input is `AcpiTableSnapshot`, containing two
`TableSnapshot` values: exact bytes plus the original firmware address
established by the kernel. Caller-supplied metadata is not authorisation
evidence. In the opt-in `native-acpi-s5` profile, the FADT address comes from
`Discovery.fadt`; `FirmwareReader` reads the FADT and DSDT only from validated
UEFI ACPI reclaim/NVS memory regions. The parser then validates SDT signatures,
lengths, checksums and the FADT/DSDT binding. This native route requires no
pre-copied, version-bound BootInfo table snapshot; address and memory
authorisation remain with the kernel.

The parser checks table signatures, exact declared length, checksum, maximum
size and binding to the DSDT address designated by FADT. Legacy PM1 length
must be at least two bytes. The parser uses a present `X_PM1a/b_CNT` GAS only
if it describes a 16-bit System-I/O register with zero bit offset and a
permitted access size; otherwise it rejects it. Legacy register addresses
are used only when the corresponding GAS address is zero/absent. Addresses
outside the 16-bit I/O range and overlapping PM1a/PM1b registers are rejected.
Hardware-reduced ACPI is unsupported.

`S5Plan` and `Pm1Setting` have private fields and no public constructor.
Their getters return parsed values only; they grant no capability. `apply_to`
only calculates a masked 16-bit value from an already read value. There is no
port I/O, `unsafe`, pointer input, firmware-code execution, shutdown syscall,
kernel authorisation or host action. A plan must never be accepted over IPC as
a shutdown command. The kernel remains the privileged owner of table access
and the opt-in, QEMU-only PM1 write action. A service-driven power-off
capability and prior service flushing are not implemented.

## Bounded AML subset

The parser recognises root-scope `Name(_S5_, Package(...))` with two integer
constants or the four fields used by QEMU: two sleep types and two mandatory
zero constants as reserved fields. Supported constants are `Zero`, `One`,
`Ones`, `Byte`, `Word`, `DWord` and `QWord`; both sleep types must fit in three
bits. Other counts and non-zero reserved fields are rejected. Name-segment
requirements are deliberately stricter than all AML variants: ASCII uppercase,
digits and underscore. Root-relative and explicitly absolute `_S5_` names
are recognised. A second root definition is ambiguous and rejected.

A literal `Scope(\)` is traversed structurally, with at most eight nested
root scopes and a fixed stack, without recursion. All object boundaries
remain within the current scope; checking surrounding terms continues after
the scope ends. Duplicate `_S5_` definitions in separate root scopes are also
rejected. Other scopes, methods and extended namespace package objects are
skipped only with a checked `PkgLength` and cannot promote nested `_S5_`
bytes into the selected root definition. This does not resolve a general
namespace, aliases, dynamic definitions or method effects; the future kernel
action therefore remains bound to the explicit QEMU profile. Name objects
outside `_S5_` support only literal data objects, bounded package/buffer/string
data or a NameString. Top-level Alias, External, Noop, Mutex, Event and
OperationRegion are skipped with limited structural parsing; unknown AML
terms, expressions, malformed lengths, invalid packages and missing/duplicate
`_S5_` return an explicit error. This is not a general AML interpreter.
Firmware DSDTs outside the subset are not accepted; broaden the subset only
with evidence that object boundaries and namespace context remain correct.

For multibyte `PkgLength`, unused bits 4/5 in the lead byte must be zero;
the subset rejects encodings that set them rather than silently masking
them. All four length widths, truncated continuation bytes, undersized length
fields and package boundaries outside the table are specifically tested.
Truncated name, string, scope, method and extended objects are rejected even
after `_S5_` has been found: finding a result does not stop validation.

The public primary implementation reference for length encoding is ACPICA,
tag `R2025_04_04`, `AcpiPsGetNextPackageLength` and
`AcpiPsGetNextPackageEnd` in
[`source/components/parser/psargs.c`](https://github.com/acpica/acpica/blob/R2025_04_04/source/components/parser/psargs.c).
That source confirms the lead-byte layout, at most three continuation bytes
and the length origin at the start of the length field. Rejecting unused bits
is an explicit strict parser choice here; ACPICA masks them. The UEFI ACPI
6.5 HTML specification returned HTTP 403 when checked. No source code was
copied and no complete AML interpreter specification conformance is claimed.

QEMU `v10.2.1`, `hw/i386/acpi-build.c`, lines 1157–1184, actually builds `_S5_`
as a four-element package within `Scope(\)`. The older two-element/root-Name
fixture was therefore not an appropriate QEMU fixture. This public primary
emulator source was read and recorded with SHA-256:
`263bcea198131385448fb5827b6a30a5943313712b7caf99ca18343e07ea1b77`.
Source: <https://github.com/qemu/qemu/blob/v10.2.1/hw/i386/acpi-build.c#L1157-L1184>.
The synthetic test fixture models this object structure. In addition, the
complete FADT/DSDT from a separate QEMU/OVMF firmware guest was read through
QMP and parsed on the host: PM1a `0x604`, S5 type zero, no PM1b. This demonstrates
compatibility with those specific firmware bytes, not the independent loader/
kernel handover or guest poweroff. Profile, table hashes, command trace and
the explicitly invoked external-fixture test are in `verification/`.

## Targeted tests and evidence limits

Unit fixtures cover SDT signature/length/checksum, source-address binding,
PM1 GAS/ranges/overlap, hardware-reduced mode, AML constants, missing,
duplicate and non-root `_S5_`, invalid `PkgLength`, wrong package size and
unknown AML. For both PM1a and PM1b, the GAS matrix independently mutates
address space, bit width, bit offset, access size and address boundary while
the legacy address remains valid: a present invalid extended description
is rejected, not replaced. Extended register overlap and incorrect `X_DSDT`
binding are also rejected despite valid legacy fields. The regular suite uses
synthetic host fixtures. A separate, normally ignored test accepts explicitly
supplied external QEMU firmware tables and also checks mutations; missing
fixture paths or addresses cause that test to fail, not silently skip.
That host test runs no guest code or shutdown. Since 5 November 2026, the
`native-acpi-s5` kernel profile has used this crate for native power-off in
QEMU. Evidence and limitations are documented in
[`docs/verification/h05-acpi-s5`](../../docs/verification/h05-acpi-s5/README.md).
Service- or user-initiated shutdown, H05 acceptance and physical-hardware
verification remain outstanding.

## Host-only S3 and FACS observations

`parse_s3(tables, facs)` reuses the same bounded AML walk, SDT/DSDT binding
and PM1 validation to select root `Name(_S3_, Package(...))`. `_S3_` and
`_S5_` are selected independently; existing S5 interfaces and rejection
behavior are preserved. `S3Plan` is opaque and exposes parsed PM1 settings
and `FacsInfo` observations. Neither is an authority to enter sleep or jump
to a waking vector. No kernel caller, trampoline, firmware method execution,
FACS mutation, global-lock acquisition or I/O is implemented.

`fadt_facs_address` resolves the validated FADT pointer so the trusted kernel
reader can authorize and obtain the snapshot without duplicating pointer parsing.
The FACS snapshot must match the FADT's `FIRMWARE_CTRL` (offset 36) or
`X_FIRMWARE_CTRL` (offset 132), with a nonzero address aligned to 64 bytes.
Partial extended fields, inconsistent nonzero pointers, snapshot mismatch
and overflowing source extents are rejected. **Stricter profile:** ACPI
prefers a nonzero extended pointer; this parser rejects contradictory
nonzero FADT pointers instead. This rule does not apply to the two waking
vectors: a nonzero extended waking vector supersedes the legacy value.

FACS is shared firmware state, **not a checksum-bearing SDT**. Its signature
must be `FACS`, its declared length must equal the complete supplied extent,
and the extent must be between 64 bytes and the existing 1 MiB parser bound.
This profile accepts fixed-layout versions 2 and 3 only. Reserved bytes
(including any extension beyond byte 64), global-lock reserved bits,
unknown flags and unsupported versions are rejected. Hardware signature and
global-lock pending/owned bits are not authentication or ownership evidence.
Only FACS flag bits 0/1 and OSPM flag bit 0 are accepted; requesting 64-bit
wake requires the firmware support bit. A selected legacy vector must be
below 1 MiB; a selected extended vector above the 32-bit address range
requires a supported 64-bit wake request. Zero vectors are accepted as
unprepared state. These are field checks, not executable-memory, mapping,
platform support or resume-readiness validation. Firmware can mutate FACS
after a snapshot; future integration must provide its own synchronization,
memory authorization and execution validation.

Primary layout references: [UEFI ACPI 6.6, sections 5.2.9 and 5.2.10](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html#firmware-acpi-control-structure-facs)
(version 3) and [TianoCore EDK II ACPI 6.1 definitions](https://github.com/tianocore/edk2/blob/master/MdePkg/Include/IndustryStandard/Acpi61.h)
(version 2). No specification or third-party implementation code was copied.

`tests/s3_facs.rs` uses synthetic host fixtures for independent S3/S5
selection, root-scope handling, malformed/missing/duplicate S3, FADT/FACS
binding, exact extent and address bounds, versions, flags, reserved fields
and vector precedence. Existing S5 tests remain unchanged. Host tests and
freestanding compilation do not establish native S3 entry, physical
sleep/wake, thermal/battery behavior or H05 acceptance.
