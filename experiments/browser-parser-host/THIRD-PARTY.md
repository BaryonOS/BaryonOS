# Derdencomponenten van de browser-parserhostproef

Gecontroleerd op 3 november 2026. De hostproef is geen onderdeel van een
Nieuw-OS-image of runtime-distributie. Er is geen broncode van derden
gekopieerd of gewijzigd.

## Directe dependency

| Pakket | Herkomst en versie | Licentie | Controle |
|---|---|---|---|
| `scraper` | crates.io `0.29.0`; [rust-scraper/scraper](https://github.com/rust-scraper/scraper), VCS-revisie `149ae5c9dfbaca2e6b92470018fedc95ab254d5d` | ISC | Cargo-archivechecksum `6844b5f49a5579f12ba4f89d9f7eec6d71889e4aa0bc7181821040ce66f08fff`; package-manifest: editie 2024, geen `build.rs`, default features uit. De release gebruikt HTML5-parsing via html5ever en DOM/CSS-selectorquery's; geen layout, JavaScript of browserplatform. |

De gebruikte resolverboom is met `cargo tree --target
x86_64-unknown-linux-gnu -e normal,build` uit de exacte lockfile verkregen.
`scraper` is exact gepind; transitieve versies en registrychecksums staan in
`Cargo.lock`. De actieve Linux-hostboom bevat de volgende licentieverklaringen
uit de genormaliseerde Cargo-manifesten:

| SPDX-licentieverklaring | Actieve pakketten (exacte versies in `Cargo.lock`) |
|---|---|
| ISC | `scraper`, `ego-tree` |
| MPL-2.0 | `cssparser`, `cssparser-macros`, `dtoa-short`, `selectors` |
| MIT | `derive_more`, `derive_more-impl`, `new_debug_unreachable`, `phf`, `phf_codegen`, `phf_generator`, `phf_macros`, `phf_shared`, `precomputed-hash` |
| MIT OR Apache-2.0 | `bitflags`, `cfg-if`, `dtoa`, `html5ever`, `itoa`, `libc`, `lock_api`, `log`, `markup5ever`, `parking_lot`, `parking_lot_core`, `proc-macro2`, `quote`, `rustc_version`, `scopeguard`, `semver`, `servo_arc`, `siphasher`, `smallvec`, `stable_deref_trait`, `string_cache`, `string_cache_codegen`, `syn` 2.x, `syn` 3.x, `tendril`, `web_atoms` |
| Apache-2.0 OR MIT | `fastrand`, `rustc-hash` |
| Unlicense OR MIT | `memchr` |
| (MIT OR Apache-2.0) AND Unicode-3.0 | `unicode-ident` |

De lockfile bevat ook feature-/targetafhankelijke pakketten die niet in deze
Linux-hostbuildboom zitten. De directe dependency en alle gebruikte
transitieve versies zijn dus niet uitsluitend uit bovenstaande gegroepeerde
licenties af te leiden: de lockfile blijft de exacte pakket-/checksumbron.
Bij distributie van een build moeten de toepasselijke upstreamnotices van de
volledige resolved dependencyset worden behouden.

## Buildscripts en platformgeschiktheid

De actieve buildscriptdoelen zijn afkomstig van `libc`, `parking_lot_core`,
`proc-macro2`, `quote` en `web_atoms`. De directe release-manifesten zijn
vóór toevoeging gecontroleerd; deze transitive scripts zijn na de eerste
hostcompilatie geïnspecteerd en hieronder gehasht. De
compiler-/configuratieprobes gebruiken alleen de bestaande Rust-toolchain;
`web_atoms` genereert Rust-bron onder Cargo `OUT_DIR` uit pakketbronnen. Er is
geen download- of systeeminstallatiestap aangetroffen. De scripts blijven
onderdeel van de ontwikkelketen; hercontrole is nodig bij een versie-upgrade.
Hun SHA-256:

| Pakket/script | SHA-256 |
|---|---|
| `libc 0.2.191/build.rs` | `e70593206df4d2a3138eab463567e0ae8f91c1bea565da4ab5370ea8986840ac` |
| `parking_lot_core 0.9.12/build.rs` | `3698c936b75f8332d9b6b30a8a6838ea116c1ff0a819a025c9ea9696e4d8ce72` |
| `proc-macro2 1.0.107/build.rs` | `baeb20b52f6b536be8657a566591a507bb2e34a45cf8baa42b135510a0c3c729` |
| `quote 1.0.47/build.rs` | `cd6808c02e476b09a520105e2c6f6d325cccb1ecd542cbbcc836a0ae6f6fb0f1` |
| `web_atoms 0.3.0/build.rs` | `8b50922bbb295e90a26edc3e2ab34f068fee930cefd73ae03cca210cf08f9d89` |

`scraper`, `html5ever` en `tendril` gebruiken de Rust-hostbibliotheek; de
`x86_64-unknown-none`-check faalt daarom. Deze dependency is uitsluitend voor
de hostfixtureproef en is niet geschikt bevonden voor de huidige `no_std`-
kernel-/servicegrens. Dit is geen volledige codeaudit of advisory-database-
scan.
