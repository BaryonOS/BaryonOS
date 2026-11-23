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

## S3, FACS and the waking-vector plan

`parse_s3(tables, facs)` reuses the same bounded AML walk, SDT/DSDT binding
and PM1 validation to select root `Name(_S3_, Package(...))`. `_S3_` and
`_S5_` are selected independently; existing S5 interfaces and rejection
behavior are preserved. `S3Plan` is opaque and exposes parsed PM1 control
settings, the PM1a/PM1b **event blocks** (`Pm1Event`, status register first,
enable register at `status + PM1_EVT_LEN / 2`, `WAKE_STATUS` = bit 15) and
`FacsInfo` observations. None of these is an authority to enter sleep or to
jump to a waking vector; this crate performs no I/O and no FACS store.

`fadt_facs_address` resolves the validated FADT pointer so the trusted kernel
reader can authorize and obtain the snapshot without duplicating pointer parsing.
The FACS snapshot must match the FADT's `FIRMWARE_CTRL` (offset 36) or
`X_FIRMWARE_CTRL` (offset 132), with a nonzero address aligned to 64 bytes.
Partial extended fields, inconsistent nonzero pointers, snapshot mismatch
and overflowing source extents are rejected. **Stricter profile:** ACPI
prefers a nonzero extended pointer; this parser rejects contradictory
nonzero FADT pointers instead. This rule does not apply to the two waking
vectors: a nonzero extended waking vector supersedes the legacy value.

The PM1 event blocks come from `PM1a_EVT_BLK`/`PM1b_EVT_BLK` (offsets 56/60)
or their `X_` GAS (offsets 148/160), which supersede the legacy value when
nonzero and must then describe System I/O, `bit_width == PM1_EVT_LEN * 8`,
bit offset 0 and access size 0 or 2. `PM1_EVT_LEN` (offset 88) must be even
and at least 4. A block past the 16-bit I/O space, a missing PM1a block, a
truncated GAS, and any overlap with a PM1 control register or with the other
event block are rejected.

FACS is shared firmware state, **not a checksum-bearing SDT**. Its signature
must be `FACS`, its declared length must equal the complete supplied extent,
and the extent must be between 64 bytes and the existing 1 MiB parser bound.
Fields are validated per version:
- **Version 0 (ACPI 1.0b).** Offsets 24..end are reserved and must be zero,
  only `S4BIOS_F` may be set, and the legacy vector must be below 1 MiB.
  QEMU 8.2.2 `pc-q35-8.2` with OVMF builds exactly this form (observed in
  guest memory on 19 November 2026); `origin/main` before this change
  rejected it with `UnsupportedFacsFields`.
- **Version 1 (ACPI 2.0/3.0).** `X_Firmware_Waking_Vector` exists but must
  fit 32 bits (no 64-bit wake), offsets 33..end must be zero and only
  `S4BIOS_F` may be set.
- **Versions 2 and 3.** Flag bits 0/1 and OSPM flag bit 0 only; a 64-bit
  wake request requires the firmware support bit; offsets 33..36 and 40..end
  must be zero.

Global-lock reserved bits are rejected for every version. Hardware signature
and global-lock pending/owned bits are observations, not authentication or
ownership evidence. Zero vectors are accepted as unprepared state.

`FacsInfo::wake_plan(real_mode_entry, protected_entry, policy)` returns the
exact stores that arm the OSPM waking vectors, and `disarmed_plan(policy)`
the stores that clear them again. The real-mode entry must be nonzero,
16-byte aligned and below `0xA0000`; the optional 32-bit entry must be
nonzero, 16-byte aligned and below 4 GiB. Offset 24 is written whenever the
FACS version defines it (cleared without a 32-bit entry), and versions 2 and
later get OSPM flags with `64BIT_WAKE_F` cleared, so firmware never enters
the OS in long mode. `ExtendedVectorPolicy::Specification` refuses a 32-bit
entry for a version-0 FACS. `ExtendedVectorPolicy::Edk2Version0Extended`,
used only by the fixed QEMU/OVMF kernel profile, also stores it into the
reserved, zero-validated offset 24 of a version-0 FACS and restores zero on
disarm; see ADR-H05-S3-WAKE-20261119 for the measured reason.

Primary layout references: [UEFI ACPI 6.6, sections 5.2.9 and 5.2.10](https://uefi.org/specs/ACPI/6.6/05_ACPI_Software_Programming_Model.html#firmware-acpi-control-structure-facs)
(version 3) and [TianoCore EDK II ACPI 6.1 definitions](https://github.com/tianocore/edk2/blob/master/MdePkg/Include/IndustryStandard/Acpi61.h)
(version 2). The edk2 resume behaviour was read in tag `edk2-stable202402`
(`UefiCpuPkg/Universal/Acpi/S3Resume2Pei/S3Resume.c`,
`MdeModulePkg/Universal/Acpi/BootScriptExecutorDxe/X64/S3Asm.nasm`). No
specification or third-party implementation code was copied.

`tests/s3_facs.rs` covers independent S3/S5 selection, root-scope handling,
malformed/missing/duplicate S3, FADT/FACS binding, exact extent and address
bounds, every FACS version's reserved fields, flags and vector precedence,
the observed QEMU/OVMF FACS and FADT PM1 layout, event-block validation and
both wake-plan policies. The ignored external-fixture test additionally
parses real FADT/DSDT/FACS bytes when `NOS_ACPI_FACS` is set. Host tests do
not establish native S3; the native QEMU route is the kernel `native-acpi-s3`
profile, documented in
[`docs/verification/h05-native-s3-20261119`](../../docs/verification/h05-native-s3-20261119/README.md).
Physical sleep/wake, thermal/battery behavior and H05 acceptance remain open.
