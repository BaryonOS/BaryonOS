# M7 image-format detection — host verification

**Status:** beperkte codec-/dispatcherhostslice, geen volledige
file-interoperability.
**Implementatiecommit:** `c2c9b64ba5c8f234b9f2ef1a93cec08f4c569172`.
**Ouder:** `7ec76b727d758057c6ee8f6ee4007a025162cb0d`.

The existing image decoder now exposes PNG/JPEG signature detection and MIME
labels, and uses that detector for decoder dispatch. A signature is only a
routing hint; complete input validation remains in the maintained decoders.
No archive, document, or general file-import support is claimed.

## Changes and source hashes

| File | SHA-256 |
|---|---|
| `services/image-codec/src/lib.rs` | `29907b30ba23844aa8ab0696b9b9b4f2ecc73eee4cd8cce82e4b735e3bd2d3ba` |
| `services/image-codec/tests/decode.rs` | `a5bf7aef35e530ca5cc750278f3d344ac559b1ce08f2db91cddde925f1568832` |

## Checks

Commands used the existing Cargo 1.90.0 toolchain, repository `tools/dev.py`
wrapper, isolated GCC/libc files under `/tmp/nieuwos-terminal-toolchain`, and
`CARGO_TARGET_DIR=/tmp/nieuw-os-image-core-target`. The required environment
was:

```sh
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/tmp/nieuwos-terminal-toolchain/root/usr/bin/x86_64-linux-gnu-gcc-15
export COMPILER_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/libexec/gcc/x86_64-linux-gnu/15
export LIBRARY_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/lib/x86_64-linux-gnu:/tmp/nieuwos-terminal-toolchain/root/usr/lib/gcc/x86_64-linux-gnu/15
export LD_LIBRARY_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/lib/x86_64-linux-gnu
export RUSTFLAGS='-C link-arg=--sysroot=/tmp/nieuwos-terminal-toolchain/root'
export CARGO_TARGET_DIR=/tmp/nieuw-os-image-core-target
```

| Check | Result | Log |
|---|---:|---|
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 test --manifest-path services/image-codec/Cargo.toml` | exit 0; 12 decode, 5 JPEG allocation/retry, and 3 upsample tests passed | `/tmp/nieuw-os-image-core-target/m7-format-run-20261103-retry/test.log`, SHA-256 `ca0b1897c9e330b86cb3ddf8ed5ce782c11cd1a89a8e748f10ff7ca3727be4a4` |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 clippy --manifest-path services/image-codec/Cargo.toml --all-targets -- -D warnings` | exit 0 | `/tmp/nieuw-os-image-core-target/m7-format-run-20261103-retry/clippy.log`, SHA-256 `e95e791b539242aaf053e14f1d62f191ab022682c70f960c0a7e03757f1ad906` |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 fmt --manifest-path services/image-codec/Cargo.toml -- --check` | exit 0 | `/tmp/nieuw-os-image-core-target/m7-format-run-20261103-retry/fmt.log`, SHA-256 `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `python3 -B tools/dev.py /home/simon/.cargo/bin/cargo +1.90.0 check --manifest-path services/image-codec/Cargo.toml --target x86_64-unknown-none` | exit 0 | `/tmp/nieuw-os-image-core-target/m7-format-run-20261103-retry/none.log`, SHA-256 `d0e40ffe867a2c4c15236dee1a6825c5dd113c6932bfef7de0a8048956b52872` |

An earlier retry with only the GCC executable exited 101 because the host
linker could not find `libc_nonshared.a`; its log is
`/tmp/nieuw-os-image-core-target/m7-format-run-20261103/test.log` (SHA-256
`8e9972d4a9e54c7dae1d0d804500b24cbe366ca662ca678ac53e6e709f83ef5d`). The
successful run supplied the existing isolated sysroot and library paths. No
dependency or system package was installed. `pkg-config` is not installed on
this host, so WPE build dependencies remain unverified. These are host-test,
lint, formatting, and cross-target compile results only—not VM, hardware,
release, or full M7 acceptance evidence.

## PR #156 handoff at review check

At head `7ec76b727d758057c6ee8f6ee4007a025162cb0d`, GitHub reported
`DIRTY`/`CHANGES_REQUESTED`; the branch was 16 commits ahead and 2 behind
`origin/main`. A read-only `git merge-tree --write-tree HEAD origin/main`
reported content conflicts in `README.md`, `docs/PROJECT-STATE.md`, and
`docs/progress.json`. No conflict resolution, rebase, or force-push was
attempted. The queue names Manager 89 as the sole status/score writer, so those
overlapping shared files remain untouched.

The only unresolved review threads were a `PROJECT-STATE.md` link that should
separate the `ADR-064` link to `DECISIONS.md` from the status link, and the
stale `progress.json` source commit for the prior M5 parser implementation.
Both are small correctness fixes, but they touch the conflicted shared files
owned by the status/score writer and were left for that owner. The review
thread on `progress.json` already contains Simon's ownership explanation.
