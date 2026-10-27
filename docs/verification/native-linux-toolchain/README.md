# Local native OS toolchain — 2026-10-27

Rust 1.90.0, Cargo 1.90.0, LLVM 20.1.8, rustfmt and clippy are installed under `cargo/` and `rustup/`. Installed targets: x86_64-unknown-linux-gnu (host), x86_64-unknown-none, x86_64-unknown-uefi. `tools/common.py` discovers these automatically.

QEMU 10.0.2 (Debian 1:10.0.2+ds-2+deb13u1~bpo12+1) is extracted under `distro/`. `qemu/qemu-system-x86_64` supplies only this process's library and module paths and QEMU data directory. `--version`, `-machine help` (pc-q35-10.0) and `-accel help` (tcg) all passed. OVMF 2025.02-8+deb13u1 plain 4 MiB CODE/VARS pair is linked under `qemu/share/`, with the package's descriptor, for automatic discovery. No Secure Boot functionality is inferred from plain OVMF.

This is a Linux development variant of the documented host toolchain, retaining pc-q35-10.0, qemu64, TCG and the existing guest profile. It does not replace or import any existing OS kernel into Nieuw OS. No package maintainer script ran; no system installation, /etc edit, global environment or shell-profile change occurred. The libraries not present on the Ubuntu host were locally extracted. Existing compatible Ubuntu libraries remain in use and are inventoried.

Verification: Rust installer/manifest SHA256 sidecars and rustup component verification; signed Debian InRelease → SHA256 Packages index → SHA256/size package verification. Debian archive key fingerprints were checked against the official HTTPS listing. Details, all package URLs/checksums, signature verification output, Rust component manifest hashes, installed binary/firmware hashes and linked host library hashes are in the JSON/TXT files alongside this file.

Runtime caveat: `rustup --version` aborts in the restricted sandbox after printing its version because wait-timeout's SIGCHLD handler receives EPERM. The same read-only metadata check passed with approved escalation. rustc/cargo version probes, QEMU version/machine/TCG probes and common.py tool/firmware discovery passed in the sandbox. Actual OS build/boot evidence is collected separately by the parent task and is not claimed here.
