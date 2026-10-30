# App package naar user-ELF-policy — A02-deelstap

**Status:** implementatie-/hostbewijs; A02 blijft open en er worden geen
roadmappunten toegekend.

| Binding | Waarde |
|---|---|
| Broncommit | `81b45969442ffd72d386ff9d484b878ff95ae1f4` |
| Branch | `codex/apps-sdk-packages` |
| `app-package-core/src/lib.rs` SHA-256 | `fbffc90812043fd963ea8e7fee748968d29e5712602b1efef45cfdccf404ef37` |
| `app-package-core/Cargo.toml` SHA-256 | `eadc438bb22a77896946273c3e3f90d82cfccf0bfc902d971ac840d59fda415d` |
| `Cargo.lock` SHA-256 | `8685abd7b7e8c028be05064be4ac1affbcc7d7d4d50b798db4104401f95885d0` |

`verify_executable_package` controleert package framing, ABI, bronankers,
Ed25519 + ML-DSA-65 en imagehash, en roept daarna de bestaande
`kernel_core::user::parse` aan. De ELF-parser blijft de enige bron voor
user-image-adres-/segment-/quota-policy. Een succesvolle uitkomst bevat de
bytes en vaste load plan; zij alloceert geen geheugen, maakt geen proces of
`LaunchImage` en verleent geen launchbevoegdheid.

## Controles

De projecttoolchain is Rust 1.90.0. Hostlinking gebruikt de bestaande
worktree-lokale `tools/dev.py`/`cc-wrapper`-omgeving; er zijn geen packages
geïnstalleerd of dependencyversies toegevoegd.

| Commando | Resultaat |
|---|---|
| `python3 -B tools/dev.py cargo +1.90.0 fmt --all` | exit 0 |
| `python3 -B tools/dev.py cargo +1.90.0 fmt --all -- --check` | exit 0 |
| `PATH="$PWD/.tools/nieuw-os/dev/root/usr/local/bin:$PWD/.tools/nieuw-os/dev/root/usr/bin:$PATH" GCC_EXEC_PREFIX="$PWD/.tools/nieuw-os/dev/root/usr/lib/gcc/" COMPILER_PATH="$PWD/.tools/nieuw-os/dev/root/usr/lib/gcc/x86_64-linux-gnu/15/" CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$PWD/.tools/nieuw-os/dev/cc-wrapper" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked -p app-package-core` | exit 0; 5 tests |
| Zelfde lokale linkeromgeving; `python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked -p app-package-core --all-targets -- -D warnings` | exit 0 |
| Zelfde lokale linkeromgeving plus `CARGO_ENCODED_RUSTFLAGS=$'--cfg\x1fcurve25519_dalek_backend="serial"'`; `python3 -B tools/dev.py cargo +1.90.0 build --offline --locked -p app-package-core --target x86_64-unknown-none` | exit 0; `no_std` library build |
| `git diff --check` | exit 0 |

Volledige uitvoer van de test-/lint-/targetbuild staat in
`/tmp/nieuw-os-app-load-checks.log` voor deze sessie.

## Bewijsgrens

Tests accepteren een geldig gesigneerd synthetic user ELF en weigeren een
correct ondertekend W+X ELF. Dit is geen native install-/launchproef.
Ontbrekend blijven onder meer bronbeheer/rotatie, persistente transacties,
update/verwijderen, cold recovery, het dynamische kernel-launchpad, apprechten
en dagelijkse native applicatietaken.
