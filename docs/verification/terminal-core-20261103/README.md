# Terminal core — host- and target-checkpoint

Datum: 3 november 2026. Branch: `simonvanieperen-vigilant-robot`.
Geteste basiscommit: `9f4b763511b88a33ea25da0a585f279fd5ab8711`.

## Resultaat en bewijsgrens

`sdk/terminal-core` levert een `no_std`, allocation-free UTF-8-regelbewerker
en begrensde argv-parser. Zes hosttests controleren tekenbewerkingen,
UTF-8-cursorgrenzen, invoer-/argumentlimieten, quotes, escaping, lege
argumenten, onvolledige invoer en geweigerde operator-/expansiesyntax.

Formattering, hostcheck, Clippy, hosttests en `x86_64-unknown-none`-check
slaagden. Dit is broncode-, hosttest- en cross-target-compilatiebewijs. Er is
geen native OS-uitvoering, Terminal-renderer, toetsenbordinvoerroute,
proces-I/O-protocol, proceslaunch of shellcommando getest.

## Exacte controles

Alle onderstaande commando's gebruikten Rust `1.90.0`, offline en de
SDK-lockfile. Voor hostlinking is GCC 15.2 met `libc6-dev` tijdelijk onder
`/tmp/nieuwos-terminal-toolchain` uitgepakt; er is niets systeembreed
geïnstalleerd. De hostcontroles gebruikten:

```sh
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER=/tmp/nieuwos-terminal-toolchain/root/usr/bin/x86_64-linux-gnu-gcc-15
export COMPILER_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/libexec/gcc/x86_64-linux-gnu/15
export LIBRARY_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/lib/x86_64-linux-gnu:/tmp/nieuwos-terminal-toolchain/root/usr/lib/gcc/x86_64-linux-gnu/15
export LD_LIBRARY_PATH=/tmp/nieuwos-terminal-toolchain/root/usr/lib/x86_64-linux-gnu
export RUSTFLAGS='-C link-arg=--sysroot=/tmp/nieuwos-terminal-toolchain/root'
```

| Exact commando | Resultaat |
|---|---|
| `$HOME/.cargo/bin/cargo +1.90.0 fmt --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core -- --check` | exit 0 |
| `$HOME/.cargo/bin/cargo +1.90.0 check --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core` | exit 0 |
| `$HOME/.cargo/bin/cargo +1.90.0 clippy --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core --all-targets -- -D warnings` | exit 0 |
| `$HOME/.cargo/bin/cargo +1.90.0 test --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core` | exit 0; 6 tests |
| `$HOME/.cargo/bin/cargo +1.90.0 check --offline --locked --manifest-path sdk/Cargo.toml -p nieuw-os-terminal-core --target x86_64-unknown-none` | exit 0 |
| `git diff --check` | exit 0 |

De gecombineerde uitvoer staat buiten Git in
`/tmp/nieuwos-terminal-toolchain/terminal-core-gates.log`.
SHA-256: `a3468f5d0c679ebaa652ff7bf671fcaf36b2a41a2feacbc591a504fa453268b0`.
Een eerste testpoging kon niet starten omdat `cargo` niet op `PATH` stond;
de directe toolchainaanroep miste vervolgens een hostlinker. Na de tijdelijke
toolchainextractie slaagden de controles hierboven.

## Bronvingerafdrukken

| Bestand | SHA-256 |
|---|---|
| `sdk/terminal-core/Cargo.toml` | `ac4458c81d17c78e0e8356b579fc7d03c124fabf16fc3101434c643a1cfabebe` |
| `sdk/terminal-core/src/lib.rs` | `0f7234b7b2fa1039fb0dbf28a8c304028a022939081711e776a11950278c6cba` |
| `sdk/terminal-core/README.md` | `76783d26cb5f592d0eda24bab34e7049927453483f76f4cb10b5505aac7a9d6f` |
| `sdk/Cargo.toml` | `a9f204dfb97004e4b3824dda29262abec9ee9114751bf50d603ef996ce713c88` |
| `sdk/Cargo.lock` | `417bb59cb343fb219ba2980e7a9d642e18716682517cad8b33d46bb7e587904e` |

Roadmaprubric: ongewijzigd, **41 → 41/100**. Deze slice voldoet niet aan een
bestaand criterium op het vereiste native bewijsniveau.
