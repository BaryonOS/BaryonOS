# Baryon OS — native desktop research for Windows/QEMU

This is an experimental VM test build of the custom x86_64 kernel with the
native desktop profile (see `manifest.json`). `native-window-session` opens
a public research workspace at 1280×720 with Materiaal and Rekenmachine,
custom windows and proportional text. The profile starts eight isolated
processes for the two applications and their system services. No account,
password, biometric or FIDO2 authentication is connected: the workspace opens
directly and is not a secured personal user session.

Older supported profiles are `native-system-materiaal` and
`native-system-session-preview`. The session preview adds user selection and a
desktop; only the explicit VM guest test opens that preview. Real authentication
is not connected there either. Automatic fault/probe profiles and the separate
HiDPI test profile are not interactive Windows bundles and are refused by this
launcher. This is not an installer, daily-use release or proof that Baryon OS
works on a physical Windows PC. The included `manifest.json` binds the kernel,
loader and firmware to the source version.

`Start-NieuwOS.ps1` is retained as a legacy script identifier; its filename is
not a statement about the current project name.

## Starting the VM

1. Install a 64-bit QEMU build for Windows from the
   [official QEMU download page](https://www.qemu.org/download/). The launcher
   requires QEMU 10.2.1 or newer, an SDL display backend and `rng-builtin`.
2. Extract this ZIP into a dedicated directory. Open PowerShell in that
   directory and run:

   ```powershell
   .\Start-NieuwOS.ps1 -QemuPath "C:\Program Files\qemu\qemu-system-x86_64.exe"
   ```

   Omit `-QemuPath` when `qemu-system-x86_64.exe` is on `PATH` or installed
   in the default installation directory. The script does not change the
   PowerShell execution policy.

   To check only the bundle hashes and QEMU requirements without starting a VM
   or creating `state\`:

   ```powershell
   .\Start-NieuwOS.ps1 -CheckOnly
   ```

   The default `-Acceleration Auto` setting checks whether QEMU offers WHPX.
   Before the real launch, it initialises a paused test machine without disks,
   networking or guest execution, with a maximum duration of 10 seconds. This
   probe does not load QEMU user configuration (`-no-user-config`). A
   successful probe selects WHPX for the real VM. If WHPX is absent or the
   probe fails, a message is shown and TCG software emulation is used.
   `-CheckOnly` only queries the advertised backends and does not test
   initialisation or start a VM.

   To require hardware acceleration without an automatic fallback:

   ```powershell
   .\Start-NieuwOS.ps1 -Acceleration Whpx
   ```

   WHPX uses CPU virtualisation through Windows Hypervisor Platform. That
   Windows feature and Intel VT-x/AMD-V in firmware must be available; this
   script does not change them. Select `-Acceleration Tcg` for the original
   software profile. The launcher records the selected backend, reason and
   startup time in `state\launch.json`. A successful initialisation probe does
   not yet prove a correct Baryon OS session. The real VM is not restarted after
   an error, preventing unnoticed repetition of disk operations.

   The SHA-256 check detects damaged or modified bundle files, but it is not a
   signature or proof of origin. Download only from the agreed project source;
   do not replace the manifest or scripts with files of unknown provenance.
3. Close the VM through the QEMU window. A subsequent start reuses the VM disk
   and OVMF variables under `state\`. To reset only the VM to its empty initial
   state, close QEMU completely and remove `state\volume.img` and
   `state\OVMF_VARS.fd`; the launcher recreates them.

The VM uses one vCPU (WHPX when initialisation succeeds, otherwise TCG),
256 MiB of RAM, a synthetic writable document image and a read-only EFI
directory. No network device, directly attached host disk, USB pass-through or
clipboard integration is configured. The `fat:esp` mapping gives the guest
read-only access to the bundle files under `esp\`. The `state\` directory
contains the writable VM disk, OVMF variables and serial output, but it is not
mounted as a host directory. This describes only the configured QEMU channels;
it is not a guarantee against vulnerabilities in QEMU.

## Limits and evidence status

This image is intended for exploring the current native research workspace.
The profile has a limited scope and complete product/release acceptance has not
been reached. This bundle does not prove networking, audio, physical hardware,
complete desktop control, accessibility or the complete storage/recovery
matrix. Earlier `Start-NieuwOS.ps1 -CheckOnly` and synthetic launcher tests
were executed in real Windows PowerShell without starting a VM. The new
profile/licence checks and eight-process session must be checked again locally
for this exact source and ZIP. VM-operation evidence on Windows or Linux is
separate and applies only to its recorded source, bundle and configuration,
not automatically to every newly packaged ZIP.

The new acceleration selection can be checked separately with synthetic tests:
run `./Test-Acceleration.ps1` from the repository directory
`tools/windows-qemu/`. That test neither initialises a hypervisor nor measures
performance. Windows/WHPX runtime evidence for this change remains open;
historical TCG experiments do not satisfy it, and earlier helper checks do not
provide Windows runtime acceptance for this new window session. QEMU documents
limitations for legacy VGA and SIMD access to MMIO; the launcher does not
disable security measures to work around them. A more modern GPU driver and
fewer rendering transactions remain separate improvements.

The firmware files are supplied with their corresponding copyright notices
under `licenses\`. The Open Sans Regular/Semibold 1.10 atlas is embedded in
the native code. `licenses\OpenSans-NOTICE.md`,
`OpenSans-Apache-2.0.txt`, `Native-graphics-README.md` and
`OpenSans-font-provenance.json` contain the copyright/licence text, atlas
limits, and hashes of the original fonts and derived atlas. The packager checks
the atlas against this provenance data. The launcher checks all supplied
notices with the same bundle hashes. No TTF files, host fonts or dynamic font
parser are loaded. QEMU itself is not included in this ZIP.

Source for Windows binaries: QEMU Project, “Download QEMU”, accessed
30 October 2026, https://www.qemu.org/download/. The Windows section links to
64-bit binaries/installers provided by Stefan Weil and lists MSYS2 as an
alternative.

QEMU Project, “Windows Hypervisor Platform” and “Security”, development manual
11.1.50, accessed 2 November 2026:
https://www.qemu.org/docs/master/system/whpx.html and
https://www.qemu.org/docs/master/system/security.html.
