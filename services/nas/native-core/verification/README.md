# NAS-CORE-1 — eerste controlecheckpoint

Datum: 30 oktober 2026, Europe/Amsterdam. Basis:
`f52dab50af281dab2c94ab2c9e31aaabd299246a`, branch
`simonvanieperen-legendary-broccoli`. De bijgevoegde hashes binden de
implementatie en tests op implementatiecheckpoint
`de8ff67ffbd44207024895afeb470c43fbced298`; dit is uitsluitend
host-/crosscompilebewijs. Controleer de hashes tegen die commit, niet tegen
latere vervolgwijzigingen. Het vervolg staat bij `../../native-volume/verification/`.

| Uitgevoerd commando | Exit | Uitkomst / wandtijd |
|---|---|---|
| `cargo test --offline --locked --manifest-path services/nas/native-core/Cargo.toml` | 0 | 18 integratietests daadwerkelijk uitgevoerd, 0,036 s |
| `cargo clippy --offline --locked --manifest-path services/nas/native-core/Cargo.toml --all-targets -- -D warnings` | 0 | geen waarschuwingen, 0,260 s |
| `cargo fmt --manifest-path services/nas/native-core/Cargo.toml --all -- --check` | 0 | geen afwijkingen, 0,034 s |
| `cargo build --offline --locked --manifest-path services/nas/native-core/Cargo.toml --release --target x86_64-unknown-none` | 0 | Rust 1.90.0, `no_std`, 0,166 s |

Volledige lokale logs: `target/nas-core/logs/host-gates.log` en
`target/nas-core/logs/bare-metal-build.log`. Geen binaries of buildcache
opgenomen in Git. De eerste testpoging zonder de gedeelde linker eindigde
met exit 101 (`cc` ontbreekt); geen testresultaat. Daarna zijn uitsluitend
bestaande binaries read-only hergebruikt, zonder installatie/fetch of schrijven
in gedeelde toolhomes:

```sh
P=/home/simon/.copilotd/worktrees/wsl-Ubuntu/nieuw-os/simonvanieperen-studious-sniffle/.tools/nieuw-os
export PATH="$P/rustup/toolchains/1.90.0-x86_64-unknown-linux-gnu/bin:$P/cargo/bin:$PATH"
export LD_LIBRARY_PATH="$P/host-cc/usr/lib/x86_64-linux-gnu"
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_LINKER="$P/cargo/bin/cc"
export CARGO_TARGET_DIR="$PWD/target/nas-core"
```

**Assessment P01–P04: 0/5 → 0/5; totaal 34/100 → 34/100.**
De 18 tests tonen caller-/namespacebinding, quota inclusief staging,
optimistische revisieconflicten, duurzame intrekking, logische verwijdering,
herstel zonder oude handles, canonieke parsergrenzen en opslagfoutgedrag.
De schijf en entropy zijn expliciete hostfixtures; transportidentiteit is
nog een adaptervoorwaarde, geen getest authenticatiepad.

**Niet uitgevoerd/open:** native NAS-proces, kernelgebonden beheer-/gastingress,
echte volumeadapter, TLS, productie-entropy/tijd, E2EE-client, sync,
apparaatkoppeling/rotatie/herstel, QEMU-/koude-diskproef, CI en menselijke
review. De grote vaste state en kopieën moeten vóór native aansluiting tegen
het stack-/geheugenbudget worden gemeten. Geen absolute veiligheidsclaim.

De gedeelde voortgangs-/statusbestanden zijn op expliciet scopeverzoek niet
gewijzigd. Dit rapport levert bron en +0-assessment aan hun integratie-eigenaar;
het is geen tweede roadmap. Eerstvolgende implementatiestap: aansluiten op de
bestaande volumeclient binnen `services/nas/**`, zonder wijzigingen in
kernel, SDK, netwerk of gedeelde documentatie.
