# B03-REL05 S6 step 4: firmware entropy source for the loader (25 November 2026)

**Native TEST experiment, not acceptance evidence.** The step-1 probe found no
`EFI_RNG_PROTOCOL` under the TEST QEMU line (U25). This experiment adds an
opt-in entropy source to the TPM loader fixture and observes whether firmware
then offers the protocol before `ExitBootServices`. The loader, the kernel and
the contract source (`EFI_RNG_PROTOCOL`, default algorithm) are unchanged. It
accepts no criterion and grants no progress points.

- Source: branch `claude/b03-rel05-tpm-20261125`, parent `a592e319`, run on the
  final uncommitted tree with `tools/common.source_hash()`
  `b75a8d4f157be4c4…` (full value in the run reports under the ignored
  `build/`).
- Change: `tools/run.py` `run_guest(virtio_rng=False)`; when `True` (only
  together with a TPM socket and without an M5 disk) QEMU gets
  `-object rng-random,id=rng0,filename=/dev/urandom` and
  `-device virtio-rng-pci,rng=rng0,id=rng,addr=0x6,disable-legacy=on,max-bytes=1048576,period=1000`,
  the same device line as the existing HID/system fixture. New
  `tools/run_tpm.py --case loader-rng` pairs it with the unchanged
  `tpm-loader-probe-test` loader and `native-tpm-crb` kernel.
- RNG result pinned per case in `run_tpm.evaluate`: `loader-probe` must report
  `RNG:ABSENT`, `loader-rng` must report `RNG:PASS`; `RNG:FAIL` (protocol
  present, `GetRNG` failed or returned only zero bytes) fails both.
- swtpm 0.10.1 (fresh state per run), QEMU 10.2.1, machine `pc-q35-10.0`, TCG,
  `-cpu qemu64`, 1 CPU, 256 MiB. OVMF: Ubuntu `ovmf` 2025.11-3ubuntu7.3,
  `OVMF_CODE_4M.fd` SHA-256 `874f15a7c9110e78…`.
- Loader `BOOTX64.EFI` SHA-256 `5f6038d6f7c83cc0…` and kernel SHA-256
  `ba349c3b83755ec4…`, identical to the step-1 evidence.

## Commands and results

Run sequentially in one checkout on the final tree:

| Command | Exit | Raw QEMU exit | `LOADER-TPM:RNG` | Guest stdout SHA-256 |
|---|---|---|---|---|
| `python3 -B tools/run_tpm.py --case loader-probe` | 0 | 33 | `ABSENT` | `5336210692e0d506…` |
| `python3 -B tools/run_tpm.py --case loader-rng` | 0 | 33 | `PASS` | `f8d9e155bf8ce42c…` |
| `python3 -B tools/run_tpm.py --case normal` | 0 | 33 | – | `783d1b60ee0dbb62…` |
| `python3 -B tools/run_tpm.py --case absent` | 0 | 33 | – | `e6fa721b3827f35c…` |
| `python3 -B -m unittest tools.tests.test_tpm` | 0 | – | – | 6 tests |

Checks on the retained reports:

- The `loader-probe` guest stdout hash equals the step-1 evidence
  (`5336210692e0d506…`), so that case is byte-identical in output.
- `loader-rng` differs from `loader-probe` in exactly the two QEMU arguments
  above; `normal` and `loader-probe` have equal commands apart from the run
  directory. Every non-loader serial line of `loader-rng`, counters included,
  equals `loader-probe`: the kernel markers are unchanged by the device.
- The `run.py` change adds arguments and a profile suffix only inside
  `if virtio_rng:`, so every existing profile keeps its command.

## Exploratory RDRAND run (not committed)

On an earlier uncommitted tree (`source_hash` `ff70cad2e5726e45…`) a
temporary `rdrand` value added `-cpu qemu64,+rdrand` without any RNG device.
Firmware also reported `RNG:PASS`, QEMU printed no unsupported-feature warning
and the guest stdout equalled the virtio run (`f8d9e155bf8ce42c…`). It was not
kept: the virtio-rng device is the one the B01 kernel keyring already owns
(ADR-043) and already runs in the hosted system profile on QEMU 8.2.2, and it
leaves the kernel's CPU model unchanged.

## Reading

- U25 (this profile): with the virtio-rng device, OVMF 2025.11 provides
  `EFI_RNG_PROTOCOL` with its default algorithm before `ExitBootServices`;
  without it, the protocol is absent. A provisioning profile must therefore
  include the device; the contract source stays `EFI_RNG_PROTOCOL` and
  provisioning still halts when it is absent.
- `RNG:PASS` means only: the protocol was located and `GetRNG` returned 32
  bytes that were not all zero (wiped, never printed). It says nothing about
  entropy quality; under TEST QEMU the bytes come from host `/dev/urandom`.

## Hosted stack (NOT RUN)

The hosted swtpm job runs Ubuntu 24.04 packages: QEMU 8.2.2, machine
`pc-q35-8.2`, `ovmf` 2024.02-2ubuntu0.10 (`OVMF_CODE_4M.fd` SHA-256
`ad23261d…`), swtpm 0.7.3. That OVMF image is not on this host; obtaining it is
a download that needs explicit approval, and no workflow runs the loader cases
(`.github/**` is protected). Expected, not observed: upstream `OvmfPkgX64`
includes `VirtioRngDxe`, so the hosted image should also report `RNG:PASS`
with the device and `RNG:ABSENT` without it.

## Limits

One local run per case on one host with one OVMF build. Not run: hosted CI,
the noble OVMF image, a profile in which the kernel keyring drives the same
virtio-rng device after the loader used it, physical hardware or firmware,
entropy-quality testing, provisioning or record code.
