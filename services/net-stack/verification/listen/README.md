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
geauthenticeerde principal verzonnen. De eerstvolgende controle was de
bestaande native actieve TCP normal/cold-pair als regressie van de gedeelde
engine; die is inmiddels geprobeerd, met onderstaande fout en baseline.
Daarna blijft een eigen luisterprobe met releaseowner-manifestkoppeling nodig.
Een groene actieve clientgate wordt niet als passief acceptbewijs gelabeld.

## Echte native regressiepoging: bootgate faalt ook op baseline

Geteste listenerbron: `c643b2d6627d62654560e92ee4a0955e53e039db`,
v3 `96c371de1d635ad7d2c713417050ffcbd8b2c823616f4a51a1d7e3f01dd7560a`.
Er is tijdens de gates geen bron gewijzigd. Omgeving: QEMU 10.2.1,
`pc-q35-10.0`, TCG single thread, `qemu64`, 256 MiB, dezelfde OVMF
2025.11-code/template als de eerdere ARP/UDP/DNS-proef.
Eerdere resultaten worden niet naar deze bron overgezet.

De eerste uitvoering stopte op de bestaande veilige-padcontrole:
`ValueError: FPU artifact escaped checkout`, exit 1, 13,31 s
(`path-guard-failure.json`). De buildmap stond in de primaire werkboom,
buiten de geïsoleerde TCP-checkout. De guard is **niet** aangepast:
alle runimages zijn daarna onder de eigen checkout gebouwd.

Exacte vervolgcommando's, vanuit de primaire werkboom, met de beschreven
lokale linker/firmware en `LOCALAPPDATA=<primaire werkboom>/.tools`:

| Commando | Exit | Uitkomst |
|---|---:|---|
| `cd build/tcp-listen-worktree && python3 -B -u tools/test_tcp_system.py` | 1 | Vijf hostgates groen, native normal faalt; 4,90 s totale gate |
| `cd build/tcp-baseline-worktree && python3 -B -u tools/test_tcp_system.py` | 1 | Ongewijzigde baseline `0ade8014`, dezelfde native bootfout; 14,65 s totale gate |

`NOS_BUILD_DIR` was per checkout `<checkout>/build/native-tcp-regression`;
`NOS_TARGET_DIR` en `CARGO_TARGET_DIR` waren de gedeelde, sequentieel
gebruikte primaire `target/tcp-listen-native`. Geen globale `RUSTFLAGS`.
De baseline is een schone detached worktree op de expliciete broncommit,
niet een handmatig teruggedraaide versie van de listener.

| Bron | Raw QEMU-exit | Normalduur | Laatste serial | Native gate |
|---|---:|---:|---|---|
| `c643b2d` | 0 | 2,606 s | `KS-01:PASS`, `KS-02:PASS` | `passed=false`, `tcp_gate=false` |
| baseline `0ade8014` | 0 | 2,588 s | `KS-01:PASS`, `KS-02:PASS` | `passed=false`, `tcp_gate=false` |

Beide bereiken BOOTINFO maar **geen KS-03, ring-3-probe of netwerkverkeer**.
Cold recovery wordt niet bereikt. Beide wegwerpvolumes blijven bytegelijk.
De runner verwerpt raw 0 terecht; dit is geen geslaagde boot.
De identieke fout is dus ook zonder listenerwijziging aanwezig; de precieze
kernel-/omgevingsrootcause is **niet vastgesteld**. Stderr meldt een
ontbrekende `kvmvapic.bin`; dat is niet bewezen als oorzaak.
Releaseowner `2ab17599` heeft deze gerichte open kernelbootgate ontvangen;
geen kernelwrite of omzeiling van de gate uitgevoerd.

Kernel-SHA256 candidate:
`d66785d3a0ce696d6daaaf9a59846b99ebcc28468921b51bd10c26775b30e10e`;
baseline:
`efa3efc8b3a7a7d900d66d5f5a37f538681cc198f7143d66ed00eba86456d66c`.
Baseline-v3:
`25b38722c1e748ad8877bab95bd88551b6b047138dc8dd6ce194da11987c29d2`.
Firmwarehashes, QEMU-argv en de overige input/outputhashes zijn rechtstreeks
uit de runresultaten overgenomen in `*-result-extract.json`; de hash van het
volledige originele resultaat is daar eveneens vastgelegd.
`*-gate.json`, `*-serial.txt` en `*-stderr.txt` zijn ongewijzigde kopieën.
`SHA256SUMS` dekt de JSON-/tekstbewijzen; geen binaries/images in Git.

De vijf kandidaat-hostgates zijn daadwerkelijk uitgevoerd, elk exit 0:
16 Python-harnastests, 16 packetpeertests, 22 stackmodellen,
52 capabilitytests (één bestaande ignored benchmark) en 20 runtime-/procestests.
Exacte commando's, stdout/stderr en exitcodes staan in
`candidate-gate.json` en `candidate-host-*`. Baseline gebruikt dezelfde
checks maar de bestaande 15 stacktests.
Volledige commandologs lokaal:
`/tmp/nieuw-os-tcp-listen-native-active{,-retry}.log`,
`/tmp/nieuw-os-tcp-baseline-native-active.log`.

**Niet geaccepteerd:** native TCP-clientregressie, passief luisteren, TLS,
NAS of N01–N04. Score blijft 34/100 en netwerk 0/10. Eerstvolgend:
kernelbootafbakening bij de releaseowner en een eigen passive-probeartifact;
parallel kan een begrensde onderhouden TLS-portproef worden voorbereid,
maar nooit met hostbewijs als native handshake gelabeld.
