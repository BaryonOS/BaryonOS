# TCP-LISTEN-1: brongebonden deelcheckpoint

Basis `0ade8014c8179241dcfc68ea0ba8083664691240`, branch
`simonvanieperen-tcp-listen`. Exclusieve wijzigingen `services/net-stack/**`;
geen kernel/manifest/SDK/NAS/ABI/tooling of dependencies veranderd.
Centrale status/assessment zijn manager-owned (`89f84098`).
Puntvoorstel N01–N04 **0/10 → 0/10**, totaal **34/100 → 34/100**.

Taakcontract: één bounded passive accept per bestaande socket, exacte
monotone acceptdeadline vóór ingress, geen verborgen backlog/relisten of
autoriteitsverlening. Zie component-README voor API en resterende grenzen.

## Geteste bron

| Bestand | SHA-256 |
|---|---|
| `src/lib.rs` | `59a9dafcfd4cc94700713dd8b02f0545233bda59ccb0500fca50c94350a61381` |
| `tests/adapter.rs` | `28c69982cc7fe4d8d0ae84da58b478dadbf894fd6a4a9947452ee4173178d50c` |
| `tests/listen.rs` | `b34ac62ede6812d4c40814c72b2d156fb06910ea5e947e09462e215070b106e3` |

## Uitgevoerd

Werkboom geïsoleerd onder `build/tcp-listen-worktree` van de primaire
sessiewerkboom. Onderstaande exacte manifestpaden zijn relatief aan die
primaire werkboom. Cargo-targetmap: primaire `target/tcp-listen-host`;
geen gelijktijdige Cargo-writers. Rust 1.91.1 is al de bestaande vereiste
van smoltcp 0.14.0. Lokale GCC-sysroot onder
`/tmp/nieuw-os-dhcp-hostcc/root`, linker
`/tmp/nieuw-os-qemu-102/host-cc`, expliciete `LD_LIBRARY_PATH`,
`GCC_EXEC_PREFIX`, `COMPILER_PATH`, `LIBRARY_PATH` en
`CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER`; geen globale `RUSTFLAGS`.

| Exact commando | Exit | Uitkomst / duurzaam log |
|---|---:|---|
| `python3 -B tools/dev.py cargo +1.91.1 test --manifest-path build/tcp-listen-worktree/services/net-stack/Cargo.toml --locked --offline` | 0 | 22/22: 12 adapter, 6 bestaande actieve conversaties, 4 passieve scenario's; `tests.txt`, 0,47 s |
| `python3 -B tools/dev.py cargo +1.91.1 clippy --manifest-path build/tcp-listen-worktree/services/net-stack/Cargo.toml --all-targets --locked --offline -- -D warnings` | 0 | `clippy.txt`, 1,05 s |
| `python3 -B tools/dev.py cargo +1.91.1 fmt --manifest-path build/tcp-listen-worktree/services/net-stack/Cargo.toml --all -- --check` | 0 | `fmt.txt` |
| `python3 -B tools/dev.py cargo +1.91.1 build --manifest-path build/tcp-listen-worktree/services/net-stack/Cargo.toml --lib --release --target x86_64-unknown-none --locked --offline` | 0 | `none.txt`, 1,53 s |

Eerdere pogingen faalden door ontbrekende cache (`smoltcp`, offline exit101),
Rust1.91-formatter/Clippy/none-target en een niet doorgegeven hostlinker.
Pas na die concrete fouten zijn de bestaande vastgepinde crates hersteld met
`python3 -B tools/dev.py cargo +1.91.1 fetch --locked --manifest-path services/net-stack/Cargo.toml`
en de ontbrekende toolchaincomponenten/target via `tools/dev.py rustup`
toegevoegd. Geen dependency-/lockfile- of globale compilerwijziging.
Die toolfailures zijn geen geslaagde tests; de tabel beschrijft de latere
daadwerkelijk uitgevoerde suites.

## Open bewijs

**Niet uitgevoerd in dit checkpoint:** native passieve TCP-proef, echte
NIC/klok voor listen, native NAS, peer-/clientauthenticatie, TLS, per-app-/
bestemmingsgrants of revoke bij queued TX. Geen native artifact of
geauthenticeerde principal verzonnen. De eerstvolgende controle is de
bestaande native actieve TCP normal/cold-pair als regressie van de gedeelde
engine, daarna een eigen luisterprobe met releaseowner-manifestkoppeling.
Een groene actieve clientgate wordt niet als passief acceptbewijs gelabeld.
