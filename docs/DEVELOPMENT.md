# Local development environment and execution

Recorded on 27 October 2026. This guide describes the Windows development route that was actually set up. The individual build, QEMU, product and quantum results are in [PROJECT-STATE.md](PROJECT-STATE.md) and their raw reports. Available tools and these commands are not, on their own, a passed OS test.

## Native M6c/M6d on the Linux development variant

The current M6c/M6d gate uses the toolchain described below and the
kernel feature `milestone-m6c`. From the repository root:

```bash
python3 -B tools/preflight.py --strict
python3 -B tools/build.py --kernel-feature milestone-m6c
python3 -B tools/run_m6c.py --case normal --smoke
python3 -B -u tools/test_m6c.py
python3 -B tools/capture_m6c.py --destination "docs/verification/m6d-local-$(python3 -c 'import uuid; print(uuid.uuid4().hex)')"
python3 -B tools/build.py --kernel-feature milestone-m6c
```

The capture command uses a new UUID to choose its own direct subdirectory of
`docs/verification/`. Use a new destination for every new run; the default
destination `docs/verification/m6d` contains accepted historical evidence and
must not be replaced by a local repeat run.

The smoke test starts two isolated apps for named objects, catalogue and
rename. The full gate also includes real external QEMU kills and cold
recoveries; success requires guest output, the raw exit and the independent
disk oracle. Every new trial creates its own synthetic volumes. An occupied
M6b-v1 volume is rejected and not migrated. Suites share build outputs: run
them one after another. The last command restores the M6c build after the gate
has checked the ordinary M1 boot; this is not an additional VM run.

The current M6c/M6d gate comprises **35 groups / 31 VM invocations**; the separate
M6b regression comprises **18 groups / 14 VM invocations**. Recorded results,
source binding and archives: [M6D-ACCEPTANCE.md](verification/M6D-ACCEPTANCE.md)
for the original Linux run and
[M6D-CLOSEOUT.md](verification/M6D-CLOSEOUT.md) for the additional
Windows run on `326d2f6`. The commands above are an execution route,
not new test results. The historical M6c gate of 19 groups / 15 VMs is
in [M6C-ACCEPTANCE.md](verification/M6C-ACCEPTANCE.md).

## Toolchain and existing native M6b route

A local Linux toolchain has also been set up on the M6b working branch: Rust 1.90.0,
both existing targets, QEMU 10.0.2 from Debian bookworm-backports and OVMF
2025.02 from a single matching CODE/VARS package. The machine remains `pc-q35-10.0`,
TCG, `qemu64`, one CPU and 256 MiB. Hashes and official provenance:
`verification/native-linux-toolchain/`. This is a separately tested
host profile; existing Windows results are retained as historical records.

`tools/common.py` discovers the tool directory `.tools/nieuw-os/` under the checkout.
On other machines the existing toolchain route can be used. Both
firmware variables `OS_OVMF_CODE` and `OS_OVMF_VARS` must point to a single matching
build when automatic discovery does not find a pair. No global
PATH, package or shell changes are needed for the recorded local route.

```bash
python3 -B tools/dev.py cargo +1.90.0 fetch --locked
python3 -B tools/preflight.py --strict
python3 -B tools/build.py --kernel-feature milestone-m6b
python3 -B tools/run_m6b.py --case normal --smoke
python3 -B -u tools/test_m6b.py
python3 -B tools/capture_m6b.py --destination "docs/verification/m6b-local-$(python3 -c 'import uuid; print(uuid.uuid4().hex)')"
```

On Linux, the host linker `cc` is additionally required for Rust host tests; the
kernel cross-build uses the linker shipped with Rust itself. Check that
dependency separately with `python3 -B tools/preflight.py --strict-host-tests`.
The existing `--strict` check continues to enforce only the boot tools and
firmware. On Debian/Ubuntu the `build-essential` package provides the
standard `cc` driver.

This capture also uses its own new directory for every run; the existing
`docs/verification/m6b` is retained. In PowerShell the equivalent
capture commands below can be used, each directly after the corresponding
passed gate and before a following suite replaces the shared build outputs:

```powershell
python -X utf8 -B tools/capture_m6c.py --destination "docs/verification/m6d-local-$(python -c 'import uuid; print(uuid.uuid4().hex)')"
python -X utf8 -B tools/capture_m6b.py --destination "docs/verification/m6b-local-$(python -c 'import uuid; print(uuid.uuid4().hex)')"
```

The smoke test creates a new disposable volume itself and has two native apps
create, modify and delete objects. It shuts the VM down after the assertions.
The complete gate also starts separate recovery VMs on exactly the
preceding images; the CLI does not accept an arbitrary existing disk.
`M6B:PASS` is only valid together with the expected raw exit and disk oracle.
After the gate the normal M1 build is in place; rebuild M6b for a demo.

Run suites one after another per checkout: the ESP, manifests and output locations
are shared. Separate, frozen Git worktrees have their own build outputs.
Keep `build/run/` until capture has checked the raw logs, hashes and images.

## Working directory and tools

Project root: `C:\Users\simon\OneDrive\Documenten\ChatGPT\Operating System`.

The additional tools are kept separately under `C:\Users\simon\AppData\Local\nieuw-os`:

| Component | Actual setup |
|---|---|
| Rust | Exactly 1.90.0, host `x86_64-pc-windows-msvc`; rustfmt and clippy |
| Cross targets | `x86_64-unknown-uefi` and `x86_64-unknown-none` |
| Cargo/rustup | Own directories `cargo/` and `rustup/` under the tool directory |
| Host linker | Existing Visual Studio 2026 C++ tools and Windows SDK; no additional C++ installation for this route |
| Kernel/UEFI linker | Bundled Rust LLVM linker; the kernel receives only its own ELF link options |
| QEMU | Local Windows package in `qemu/`; the binary reports `11.1.0 (v11.1.0-12130-ge470268ff4)` |
| Firmware | CODE/VARS from that same QEMU package, linked by `share/firmware/60-edk2-x86_64.json` |
| Python, Git and Node | Host tools that were already available |

[tools/common.py](../tools/common.py) creates only a subprocess environment: `CARGO_HOME`, `RUSTUP_HOME`, tool paths and, where needed, the discovered MSVC/SDK library paths. It does not write any global PATH or shell settings. `RUSTUP_AUTO_INSTALL=0` prevents automatic toolchain installation by these helpers. [tools/dev.py](../tools/dev.py) uses the same environment for a single command.

The first run may fetch public development dependencies. After that, repeated builds use the exact lockfiles; `--offline` is available once the cache has been populated. This download path is development tooling and is not networking functionality of the new OS.

## Recommended execution route

Open PowerShell in the project root. First check the inventory:

```powershell
python tools/preflight.py --strict
```

This takes an inventory of tools, targets and firmware. It does not build or boot anything. Exit 2 means missing prerequisites; a tool that was found is not yet a proven compatible build combination.

Then build the project's own loader and kernel, inspect the produced PE/ELF artifacts and start the smoke test:

```powershell
python tools/build.py
python tools/run.py --smoke
```

The full boot acceptance trial starts with:

```powershell
python tools/test.py
```

That command covers host tests, three cold VM starts, a missing/corrupt kernel, the test with a stale firmware map key, a kernel panic and a real guest timeout. The hang trial may deliberately take 60 seconds. After the feature trials the normal build is restored. Only a current suite with `status: completed` and `passed: true` counts as a passed full run; `running` and `failed` are not a green result.

The separate M2 gate builds real fault variants and then restores the ordinary M1 image:

```powershell
python -u tools/test_milestones.py --through m2
python tools/capture_gate.py m2
```

`capture_gate.py` checks the raw exit/markers, serial output and artifact hashes before it copies the compact evidence to `docs/verification/m2-m4/m2/`. It does not change test outcomes. Existing older runs still refer to the hashed source for the unchanged VARS template; new runs also store that template in their own directory.

The full M2–M3 gate was run on `6802b17` and starts with:

```powershell
python -u tools/test_milestones.py --through m3
python tools/capture_gate.py m2
python tools/capture_gate.py m3
```

This sequence runs eight M2 groups and twenty M3 groups. After the separate fault variants it restores the ordinary M1 image. For the normal M3 development trial with the project's own user processes, explicitly build the M3 profile and require the corresponding markers:

```powershell
python tools/build.py --kernel-feature milestone-m3
python tools/run.py --smoke --milestone m3
```

The M3 CLI gate requires actual KS-04/07/08 and M3 markers; an ordinary M1 image therefore cannot pass as M3. The normal M3 trial runs the fixed synthetic programs and then shuts QEMU down. There is no interactive OS shell or native Materiaal desktop yet.

`userspace/` contains the project's own static user probe and thirteen host artifact tests. `tools/build_users.py` uses direct rustc arguments with the user linker script; the Cargo kernel link options are not reused for this. M4 adds a separate IPC probe via `--kind ipc`; that track only counts as an achieved result after its own M4 gate.

The complete M1–M4 check was run with these commands:

```powershell
python -u tools/test_milestones.py --through m4
python -u tools/test.py
python tools/capture_gate.py m1
python tools/capture_gate.py m2
python tools/capture_gate.py m3
python tools/capture_gate.py m4
```

That is 52 acceptance groups and 44 separate VM runs. The deliberate hang in the M1 regression takes 60 s; the negative raw guest outcomes are not turned into success when evidence is copied. The current suites and explicit host/QEMU boundaries are in verification/m2-m4/.

Start the normal M4 development trial with:

```powershell
python tools/build.py --kernel-feature milestone-m4
python tools/run.py --smoke --milestone m4
```

The kernel runs three synthetic processes and a normal IPC exchange and terminates QEMU after checking. For separate negative cases use `test_milestones.py`; the normal M4 CLI gate requires the exchange/survivor/cleanup markers and does not accept an ordinary M1 image as M4.

The later source fingerprint version 2 excludes generated `verification/`, `build/`, `target/` and caches, and includes user source. Build manifests and guest results state the version explicitly. Older reports without this field keep their original fingerprint; artifact hashes remain directly verifiable.

A single development command in the same environment:

```powershell
python tools/dev.py cargo +1.90.0 test --locked -p boot-protocol -p elf-loader-core
python tools/dev.py rustc +1.90.0 -vV
```

The loader and kernel use separate targets. Setting a single global workspace target would disrupt the host tests and is not part of this setup.

## Starting and checking the M5 block service

Explicitly build the M5 profile and start the finite development trial:

```powershell
python tools/build.py --kernel-feature milestone-m5
python tools/run.py --smoke --milestone m5
```

For each start, the runner creates a new synthetic 64 KiB image under `build/run/`, attaches it read-only to modern virtio-blk at `00:05.0` and stores the full command line, serial output, stderr, firmware/binary/image hashes and the external data-check result. The separate boot ESP remains a different device. There is no parameter for a real host disk. The service reads sectors 4 and 6 via IPC, after which QEMU shuts down following the assertions. This is not yet an interactive OS shell.

The M5 gate runs host tests, ELF checks, the same kernel on two different images, 17 fault variants and an M1 regression start:

```powershell
python -u tools/test_m5.py
python tools/capture_gate.py m5
```

For the combined M2–M5 regression and the full boot fault sequence:

```powershell
python -u tools/test_milestones.py --through m5
python -u tools/test.py
```

These suites restore the ordinary M1 build at the end. Then rebuild `milestone-m5` for a standalone M5 start. For fault features use the gate with the corresponding case markers; `run.py --milestone m5` deliberately requires the normal READ case. Only `status: completed` and `passed: true`, together with re-checked raw evidence, count as a passed gate. The current executed evidence is in PROJECT-STATE.md.

The service protocol and all injection boundaries are in [M5-BLOCK-SERVICE.md](implementation/M5-BLOCK-SERVICE.md). The kernel transport and the protocol-conforming device are trusted; M5 provides no IOMMU containment or physical-hardware evidence. If the reset acknowledgement is missing, the fixed DMA pool stays in quarantine until the VM stops.

## Windows: exclusive test images outside synchronised folders

Run writing M6 gates in a separate Git worktree outside synchronised
folders. On 28 October 2026 the single-link guard correctly rejected a real normal M6c run:
during execution a second hard link to the same image appeared
under `.tmp.driveupload`. `fsutil hardlink list` confirmed both names; which
process created that link has not been established. The guest result raw 33 and all
success markers did not make this host-boundary failure green.

A separate worktree keeps the same commit and source fingerprint. Store
raw images only in its own `build/run`; include only verifiable
text/hash reports in Git. The `st_nlink == 1` condition remains mandatory;
there is no exception for sync software or a retry that hides the observed second link.
Do not change sync, network or other host settings for the trial.
The failed run and the exact hard-link observation are under
`docs/verification/m6d-closeout/`.

## Fixed QEMU profile

The local development variant uses `pc-q35-10.0`, `qemu64`, TCG, one CPU, 256 MiB RAM, no guest network and no host disk. The test creates its own ESP copy and a fresh VARS copy in `build/run/`; CODE is used read-only. The concrete arguments, raw exit status, hashes and serial output are stored per run in `result.json`, `serial.txt` and `stderr.txt`.

The normal success observation requires complete lines in order: `BOOT:EBS_OK`, `KERNEL:ENTRY`, `BOOTINFO:OK`, `M1:PASS`, plus raw QEMU exit 33 and no failure marker. A normal host exit 0 or only a firmware banner does not count as a kernel boot. A failure uses raw QEMU exit 35 in this test profile; a timeout and missing markers remain failures.

The starting documents named QEMU 10.0.14 and a self-built EDK II `edk2-stable202508` as the reference. The current Windows package and bundled firmware are **not byte-identical to that reference**. The exact firmware build commit has not been independently established; the descriptor, local hashes and package provenance have been recorded. This concerns a development profile without Secure Boot evidence.

**Agreed profile correction during integration:** because of observed firmware overlap, the lead agent designated the kernel reservation **32–48 MiB** (`[0x02000000, 0x03000000)`) and **BootInfo ABI 1.1** as the replacement for the original 2–18 MiB/ABI 1.0. This is an explicit change to the build baseline. This correction has been implemented and the full boot acceptance sequence has passed. The decision and execution evidence are in [DECISIONS.md](DECISIONS.md) and [PROJECT-STATE.md](PROJECT-STATE.md).

## Product prototype and quantum trial

Start Materiaal exclusively on the local loopback interface:

```powershell
python -m http.server 8765 --bind 127.0.0.1 --directory ui/prototype
```

Open [Materiaal on localhost](http://127.0.0.1:8765/). The final check after resuming used port 8776, because 8765 returned empty responses. If needed, use the same command with 8776 and open http://127.0.0.1:8776/. Stop the server with `Ctrl+C`. This is the browser prototype with fictitious data; it does not yet run on the project's own kernel. The extended controls and known evidence boundaries are in [ui/prototype/README.md](../ui/prototype/README.md).

Check the quantum trial and run the Bell fixture:

```powershell
python experiments/quantum/verify.py
Get-Content -Raw experiments/quantum/fixtures/bell8.json | python tools/dev.py cargo +1.90.0 run --manifest-path experiments/quantum/Cargo.toml --locked --offline -p qsim-cli
```

The quantum workspace is outside the kernel workspace. `verify.py` writes output, source/binary hashes and exact command lines to `experiments/quantum/verification/`. This is local host simulation; not a physical QPU, an own-OS port or a cryptographic random number generator. See [the quantum guide](../experiments/quantum/README.md).

## Provenance and hashes

The values below come from the [build/toolchain-provenance.json](../build/toolchain-provenance.json) that was actually created. Hashes identify the files used; they do not prove error-free or secure software.

| File | Algorithm | Recorded hash |
|---|---|---|
| rustup-init.exe | SHA-256 | `00f139da614dced87d3c3d4121c43eebdac3c1bf8a5ee86b9d80dfd71c8d70fd` |
| qemu-w64-setup-20260811.exe | SHA-512 | `5bcf9eed634e8575a37b74f445af41a2fe4106da512d0c30c368301d4c105037fdfab40a5287367a28a957624cddebbc8c07e16c88ab6634f554cdf3d16bf543` |
| edk2-x86_64-code.fd | SHA-256 | `33090cc07675baa5190d9f1e84bf5176b33bcbfa9bacac522961150cdb6dbb2a` |
| edk2-i386-vars.fd, template | SHA-256 | `5d2ac383371b408398accee7ec27c8c09ea5b74a0de0ceea6513388b15be5d1e` |

On its [official installation page](https://rust-lang.org/tools/install/), Rust links to [the x64 rustup download that was used](https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe). The [official QEMU download page](https://www.qemu.org/download/#windows) refers to Stefan Weil for Windows; the package used is [qemu-w64-setup-20260811.exe](https://qemu.weilnetz.de/w64/qemu-w64-setup-20260811.exe). These references were checked on 27 October 2026. They are provenance references, not an instruction to reinstall.

The loader dependency `r-efi 7.1.0` provides only UEFI types/constants. Provenance, the Cargo checksum and the chosen MIT route are in [third-party/README.md](third-party/README.md).

## Recovery and cleanup

Keep the source files, Git commits, lockfiles and any evidence reports you want to retain. `build/`, the root `target/` and `experiments/quantum/target/` contain generated files that can be recreated with the commands above. Keep `build/toolchain-provenance.json` and relevant `build/run/` reports if the evidence must remain verifiable later; rebuilding does not reconstruct the old run.

First stop QEMU, the local web server and any running Cargo processes. The separate tool directory `C:\Users\simon\AppData\Local\nieuw-os` can then be removed if desired, after checking the exact path and any downloads/logs still wanted. That removes the project-local compiler/cache/emulator, not the source repository or the existing Visual Studio installation. The development commands will only work again once the required tools are available again. This guide does not perform any removal.

No physical disk was used as a VM disk, no firmware was flashed and no NAS configuration was changed. A source rollback is done while preserving work and history, for example via a targeted revert of a later implementation commit; no destructive reset of user work.
