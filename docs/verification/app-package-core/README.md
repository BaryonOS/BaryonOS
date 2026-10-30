# A02 app-package verifier checkpoint

**Status:** implementation substep only; A02 remains open and no roadmap
points are awarded.

| Binding | Value |
|---|---|
| Source commit | `917126f774838d2e53c8fd796b722e308e923d1b` |
| Branch | `codex/apps-sdk-packages` |
| Source fingerprint | `190bce7e518376560191029865bf2fa957153811248be45384eb70f58cf8338d` |
| `Cargo.lock` SHA-256 | `2f222cf30dbf782c31c6005aff2437e903854f11b1a1864ab47959f106cb7d5c` |
| Verifier SHA-256 | `7d4ed5b4ce0885e8324784002eceed0e78c730c67885b266255b583e473e254b` |

## Checks

All commands ran against the source tree above; each final command exited 0.

| Command | Result |
|---|---|
| `python3 -B tools/dev.py cargo +1.90.0 fmt --all` | Passed |
| `PATH="$PWD/.tools/nieuw-os/dev/root/usr/local/bin:$PWD/.tools/nieuw-os/dev/root/usr/bin:$PATH" GCC_EXEC_PREFIX="$PWD/.tools/nieuw-os/dev/root/usr/lib/gcc/" COMPILER_PATH="$PWD/.tools/nieuw-os/dev/root/usr/lib/gcc/x86_64-linux-gnu/15/" CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$PWD/.tools/nieuw-os/dev/cc-wrapper" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked -p app-package-core` | 3 tests passed |
| Same local linker environment; `python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked -p app-package-core --all-targets -- -D warnings` | Passed |
| Same local linker environment plus `CARGO_ENCODED_RUSTFLAGS=$'--cfg\x1fcurve25519_dalek_backend="serial"'`; `python3 -B tools/dev.py cargo +1.90.0 build --offline --locked -p app-package-core --target x86_64-unknown-none` | Passed; `no_std` library cross-build |
| Same local linker environment; `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked -p update-core` | 7 tests passed |
| Same local linker environment; `python3 -B tools/dev.py cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-sdk-runtime` | 20 tests passed |
| `python3 -B tools/dev.py cargo +1.90.0 fmt --all -- --check` | Passed |
| `python3 -B tools/dev.py cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml --all -- --check` | Passed |
| `git diff --check` | Passed |
| `python3 -B tools/preflight.py --strict --firmware '/mnt/c/Program Files/qemu/share/edk2-x86_64-code.fd'` | Exit 0; inventory only, QEMU 11.1.0 and code firmware found |

The initial bare-metal build selected the SIMD Curve25519 backend and crashed
rustc 1.90.0. Repeating with the already-used portable `serial` backend passed.
An earlier preflight without the explicit external firmware path exited 2;
the strict inventory above passed after wiring the user-provided Windows QEMU
binary into the ignored worktree-local tool directory. No system or Windows
installation was modified.

## Not established

No guest VM was booted for this package verifier. This crate does not parse ELF,
install or persist a package, choose/rotate multiple source anchors, update or
remove apps, create a kernel `LaunchImage`, or grant user rights. The VM inventory
does not validate a boot, and the host/target checks do not close A02, A03, or
A04. The signature proves only provenance under a selected key, not app safety.
