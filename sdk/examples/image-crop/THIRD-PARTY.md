# Dependencycontrole beeld-crop

De decoder blijft de bestaande `services/image-codec` met zijn gepinde
ledger/vendorbron. Geen decoderfoundation gewijzigd.

De optionele **host-only** PNG-encoder is `png = "=0.18.1"`,
defaultfeatures uit, uit <https://github.com/image-rs/image-png>.
Cargo.toml.orig, CHANGES.md en LICENSE-MIT van de registrybron op docs.rs
zijn gelezen voordat de dependency werd toegevoegd.
Licentie MIT OR Apache-2.0, MSRV 1.73. De package-include bevat broncode,
licenties en voorbeelden, geen build.rs. Onderhouden door image-rs;
de 0.18.1-changelog bevat actuele encoder-/decoderverbeteringen.
Geen nightly/unstable of zlib-rs feature ingeschakeld.

Dit is een std-encoder, **geen** native no_std-encoderport. Alleen de
host-cli-feature kiest deze dependency; de crop-kern gebruikt bestaande
no_std-codecinterfaces. Registryversies/checksums van de volledige
closure staan in Cargo.lock. De daadwerkelijke closure is vóór de eerste
compilatie met cargo metadata, registry-Cargo.toml.orig, aanwezige licenties
en alle custom-build-sources gecontroleerd:

| Crate | Versie | Licentie | Buildscript |
|---|---|---|---|
| png | 0.18.1 | MIT OR Apache-2.0 | geen |
| bitflags | 2.13.2 | MIT OR Apache-2.0 | geen |
| crc32fast | 1.5.4 | MIT OR Apache-2.0 | gecontroleerde uitzondering hieronder |
| cfg-if | 1.0.5 | MIT OR Apache-2.0 | geen |
| fdeflate | 0.3.7 | MIT OR Apache-2.0 | geen |
| flate2 | 1.1.10 | MIT OR Apache-2.0 | geen |
| miniz_oxide | 0.8.9 en 0.9.1 | MIT OR Zlib OR Apache-2.0 | geen |
| adler2 | 2.0.1 | 0BSD OR MIT OR Apache-2.0 | geen |
| simd-adler32 | bestaande 0.3.10 | bestaande codecledger | geen |

De repositories zijn die in de registrymanifests: image-rs/image-png,
bitflags/bitflags, srijs/rust-crc32fast, rust-lang/cfg-if, image-rs/fdeflate,
rust-lang/flate2-rs, Frommi/miniz_oxide en oyvindln/adler2.
Onderhoud blijkt uit registryreleases/manifests en png-changelog, geen
onafhankelijke code-/securityaudit. Checksums in Cargo.lock pinnen alle
transitives zonder losse onnodige directdependencies toe te voegen.

**Buildscriptuitzondering:** crc32fast/build.rs is geheel gelezen. Het
voert alleen de door Cargo gekozen `RUSTC --version` uit en emit
compile-time cfg/check-cfg voor gestabiliseerde CPU-intrinsics.
Geen downloads, C-toolchain, disk-/netwerkoperatie of extra scriptketen.
Dit is alleen voor de hostencoderclosure; niet in de default no_std-kern.
Runtime-CPUdetectie in std-upstreamcode is uitsluitend hostfunctionaliteit.
De daadwerkelijke featuretree selecteert flate2/miniz_oxide, **geen**
zlib-rs of C-backend. Cargo fetch haalt een ongebruikte optional zlib-rs
tarball op voor lock/resolution; dat is geen geselecteerd buildtarget.
