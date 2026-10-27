# Derdencomponenten van de eerste bootproef

Vastgelegd op 27 oktober 2026, op basis van de daadwerkelijk opgehaalde Cargo-package en de root-`Cargo.lock`.

## r-efi 7.1.0

| Gegeven | Gecontroleerde waarde |
|---|---|
| Functie | UEFI-referentietypen, protocoldefinities en constanten; geen bootloader of kernelimplementatie |
| Upstream | [r-efi/r-efi](https://github.com/r-efi/r-efi) |
| Directe pin | `r-efi = { version = "=7.1.0", default-features = false }` in `boot/loader/Cargo.toml` |
| Bron | Cargo-registry `https://github.com/rust-lang/crates.io-index` |
| Cargo-checksum | `b0ff379dd174eec5b2cab797063a2e7592dfcf0bf0323a50eeee724ee3537f13` |
| Verpakte VCS-verwijzing | `a45901440b107b966273dcacc5eaff95c9a5bfbd` uit `.cargo_vcs_info.json`; package-identiteit blijft bepaald door versie/checksum |
| Minimum-Rust volgens manifest | 1.68 |
| `no_std` | `#![cfg_attr(not(test), no_std)]` in de echte `src/lib.rs` |
| Buildscript | `build = false` |
| Dependencies in deze configuratie | Geen externe transitieve dependency; optioneel `rustc-std-workspace-core` staat uit |
| Upstream licentiekeuze | MIT OR Apache-2.0 OR LGPL-2.1-or-later |
| Gekozen route voor dit project | MIT |

De broncontrole betrof het genormaliseerde verpakte `Cargo.toml`, `README.md`, `src/lib.rs`, `.cargo_vcs_info.json`, `AUTHORS` en de werkelijk gegenereerde root-lockfile. De dependency is alleen aan de UEFI-loader gekoppeld; de eigen kernel gebruikt deze crate niet. Dit bewijst geen volledige codeaudit of afwezigheid van kwetsbaarheden.

[r-efi-MIT.txt](r-efi-MIT.txt) is een **ongewijzigde kopie van upstream `AUTHORS`** uit package 7.1.0. Dat bestand bevat de MIT-toestemming, copyrightvermeldingen en auteurs, naast de twee alternatieve upstreamlicentieroutes. Het volledige origineel is behouden om notices niet af te snijden; dit project kiest de MIT-route. Er is geen zelfgeschreven vervangende licentietekst toegevoegd.

Versie- of checksumwijzigingen zijn een bewuste dependencywijziging: controleer herkomst, manifest, licentie, features en eventuele buildscripts opnieuw en voer de loader-/boottests uit. Behoud notices bij distributie. Verwijderen/vervangen van r-efi vereist herbeoordeling van de ruwe UEFI-ABI-bindings; het mag geen stilzwijgende overstap naar een bestaande loader of kernel veroorzaken.

## Andere ontwikkelcomponenten

Compiler, emulator en firmware zijn afzonderlijke ontwikkelcomponenten. Hun daadwerkelijke lokale herkomst en hashes staan in [DEVELOPMENT.md](../DEVELOPMENT.md) en `build/toolchain-provenance.json`. De EDK II-firmware is de packaged ontwikkelvariant; de firmwarebuildcommit is niet onafhankelijk vastgesteld. Deze bootdependency-notice is geen compleet distributie-SBOM of publicatiegoedkeuring.

Het quantumhostexperiment heeft een eigen lockfile, [dependencyregistratie](../../experiments/quantum/THIRD-PARTY.md) en bewaarde upstreamnotices. De quantumcore heeft geen externe dependencies. Het Materiaal-browserprototype heeft geen externe runtimepackages of fonts.
