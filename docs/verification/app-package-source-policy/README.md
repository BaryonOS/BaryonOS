# Appbronbeheer-policy-core — A02-deelstap

**Status:** in-memory host-/no_std-kern; A02 blijft open en er worden geen
roadmappunten toegekend.

| Binding | Waarde |
|---|---|
| Broncommit | `be830818bd88a6670d0facb524e8f1f7da3274b6` |
| Branch | `codex/apps-sdk-packages` |
| `app-package-core/src/lib.rs` SHA-256 | `3c37abf87595cd13f8dde0af73a7b95b5b978aff01584d0ebc8aefa12080a4b0` |
| `app-package-core/Cargo.toml` SHA-256 | `eadc438bb22a77896946273c3e3f90d82cfccf0bfc902d971ac840d59fda415d` |
| `Cargo.lock` SHA-256 | `8685abd7b7e8c028be05064be4ac1affbcc7d7d4d50b798db4104401f95885d0` |

`SourceRegistry` heeft maximaal acht vaste slots en verwerkt een expliciet
gekozen `SourceId`; pakketten kunnen geen bron of sleutel aanwijzen. Rotatie
en intrekking vergelijken de verwachte generatie en verhogen die atomair.
Revocation voorkomt toekomstige packageverificaties en updates. Reeds
geïnstalleerde apps blijven startbaar; apps/data worden niet automatisch
verwijderd (gebruikersbesluit 30 oktober 2026). De registry is alleen
geheugenresident en de mutaties zijn zonder service-autorisatie niet veilig
aan een app bloot te stellen.

## Controles

Projecttoolchain: Rust 1.90.0; hostlinking via bestaande worktree-lokale
`tools/dev.py`/`cc-wrapper`. Geen dependencyversies toegevoegd.

| Commando | Resultaat |
|---|---|
| `python3 -B tools/dev.py cargo +1.90.0 fmt --all` | exit 0 |
| `python3 -B tools/dev.py cargo +1.90.0 fmt --all -- --check` | exit 0 |
| `PATH="$PWD/.tools/nieuw-os/dev/root/usr/local/bin:$PWD/.tools/nieuw-os/dev/root/usr/bin:$PATH" GCC_EXEC_PREFIX="$PWD/.tools/nieuw-os/dev/root/usr/lib/gcc/" COMPILER_PATH="$PWD/.tools/nieuw-os/dev/root/usr/lib/gcc/x86_64-linux-gnu/15/" CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$PWD/.tools/nieuw-os/dev/cc-wrapper" python3 -B tools/dev.py cargo +1.90.0 test --offline --locked -p app-package-core` | exit 0; 8 tests |
| Zelfde lokale linkeromgeving; `python3 -B tools/dev.py cargo +1.90.0 clippy --offline --locked -p app-package-core --all-targets -- -D warnings` | exit 0 |
| Zelfde lokale linkeromgeving plus `CARGO_ENCODED_RUSTFLAGS=$'--cfg\x1fcurve25519_dalek_backend="serial"'`; `python3 -B tools/dev.py cargo +1.90.0 build --offline --locked -p app-package-core --target x86_64-unknown-none` | exit 0; `no_std` library build |
| `git diff --check` | exit 0 |

Uitvoer van de finale commando's staat in
`/tmp/nieuw-os-app-sources-final.log` voor deze sessie.

## Bewijsgrens

Tests controleren twee onafhankelijke bronnen, keypairselectie,
generation-checked rotatie/intrekking, stale acties, dubbele identiteit/
sleutels, capaciteitsgrens en dat een signer-valid maar W+X ELF wordt
geweigerd. Geen native installer-, UI-, duurzame opslag-, crash-/cold-recovery-
of app-launchproef is uitgevoerd. Acht bronnen is een implementatiegrens, geen
productlimiet.
